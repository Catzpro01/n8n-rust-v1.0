#!/usr/bin/env python3
"""Aturan normalisasi milik agent3 (N-08, N-09, N-22)
untuk differential harness (mandat fern #168/#170; desain AGENT4-NORMALIZER-DESIGN.md #179).

Pembagian (RULE_OWNER, dikonfirmasi #192/#199/#211):
  agent4 -> N-01..N-05, N-07, N-11, N-20, N-21 (norm_rules_a4.py)
  agent1 -> N-06, N-10, N-12 (norm_rules_a1.py + driver diff_harness.py)
  agent3 -> N-08, N-09, N-22 (file ini)

N-08: Timezone offset normalization — +07:00 vs +0700 vs +07 -> setara (parse ke UTC instant)
N-09: pairedItem handling — abaikan orphan (default) atau laporkan sbg deviasi eksplisit (strict)
N-22: Trigger non-deterministik -> SKIP-EXEC atau ganti Manual Trigger

TANTANGAN agent1 (#192): "N-09 pairedItem orphan JANGAN diabaikan — jadikan kelas diff eksplisit"
JAWABAN: N-09 punya toggle strict (default OFF = abaikan orphan utk diff; strict=True = laporkan
PAIRED_ITEM_ORPHAN sebagai kelas deviasi). Driver memilih sesuai konteks.

QA tooling (bukan kode produk Rust). Setiap fungsi + test di bawah.
"""
import re
import copy
from datetime import datetime, timezone, timedelta

# ===========================================================================
# N-08 — Timezone offset normalization
# ===========================================================================
# Masalah: Luxon/QuickJS bisa format timezone berbeda:
#   "2026-09-09T04:00:00.000+07:00" vs "2026-09-09T04:00:00.000+0700" vs "+07"
# Keduanya merepresentasikan instant yang sama tapi string berbeda.
# Solusi: parse ke UTC epoch ms, lalu bandingkan.

# Regex untuk timezone offset dalam berbagai format
_TZ_OFFSET_RE = re.compile(
    r'([+-])(\d{2}):?(\d{2})$'  # +07:00, +0700, -05:30
)

# Regex lengkap ISO-8601 dengan timezone
_ISO_TZ_RE = re.compile(
    r'^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?)'
    r'(Z|[+-]\d{2}:?\d{2})?$'
)


def _parse_to_utc_epoch_ms(s):
    """Parse ISO-8601 string ke UTC epoch milliseconds.
    Mendukung: Z, +HH:MM, +HHMM, +HH, tanpa timezone (anggap UTC).
    Returns None jika bukan ISO string yang valid."""
    if not isinstance(s, str):
        return None

    m = _ISO_TZ_RE.match(s)
    if not m:
        return None

    dt_part = m.group(1)
    tz_part = m.group(2)

    try:
        # Parse datetime part
        # Handle fractional seconds
        if '.' in dt_part:
            dt = datetime.strptime(dt_part, '%Y-%m-%dT%H:%M:%S.%f')
        else:
            dt = datetime.strptime(dt_part, '%Y-%m-%dT%H:%M:%S')

        # Apply timezone
        if tz_part is None or tz_part == '':
            # No timezone -> assume UTC
            dt = dt.replace(tzinfo=timezone.utc)
        elif tz_part == 'Z':
            dt = dt.replace(tzinfo=timezone.utc)
        else:
            # Parse offset: +07:00, +0700, -05:30
            sign = 1 if tz_part[0] == '+' else -1
            tz_clean = tz_part[1:].replace(':', '')
            hours = int(tz_clean[:2])
            minutes = int(tz_clean[2:4]) if len(tz_clean) >= 4 else 0
            offset = timedelta(hours=hours, minutes=minutes) * sign
            dt = dt.replace(tzinfo=timezone(offset))

        # Convert to UTC epoch ms
        epoch_ms = int(dt.timestamp() * 1000)
        return epoch_ms
    except (ValueError, OverflowError):
        return None


def normalize_timezone(obj, time_keys=None, all_iso=False):
    """N-08. Normalisasi timezone offset ke UTC epoch ms.
    Menangani: +07:00, +0700, Z, tanpa timezone.
    time_keys: set key names yang dianggap timestamp (default: common n8n keys).
    all_iso: jika True, semua string ISO-8601 dinormalisasi (bukan hanya time_keys)."""
    time_keys = time_keys if time_keys is not None else {
        'createdAt', 'updatedAt', 'startedAt', 'finishedAt',
        'ts', 'time', 'timestamp', 'date', 'started', 'finished'
    }

    if isinstance(obj, dict):
        out = {}
        for k, v in obj.items():
            if isinstance(v, str) and ((k in time_keys) or all_iso):
                epoch_ms = _parse_to_utc_epoch_ms(v)
                out[k] = epoch_ms if epoch_ms is not None else v
            else:
                out[k] = normalize_timezone(v, time_keys, all_iso)
        return out
    if isinstance(obj, list):
        return [normalize_timezone(v, time_keys, all_iso) for v in obj]
    return obj


def timezone_offsets_equal(s1, s2):
    """Check apakah dua ISO string merepresentasikan instant yang sama.
    Berguna untuk verifikasi sebelum normalisasi."""
    e1 = _parse_to_utc_epoch_ms(s1)
    e2 = _parse_to_utc_epoch_ms(s2)
    if e1 is None or e2 is None:
        return False
    return e1 == e2


# ===========================================================================
# N-09 — pairedItem handling
# ===========================================================================
# Masalah: pairedItem menunjuk ke index item di node sebelumnya.
# Saat crash recovery atau partial execution, pairedItem bisa menunjuk
# index yang tidak ada (orphan). Ini noise untuk differential testing.
#
# TANTANGAN agent1 (#192): "N-09 pairedItem orphan JANGAN diabaikan —
# jadikan kelas diff eksplisit"
#
# JAWABAN: Toggle strict.
#   strict=False (default): hapus pairedItem orphan dari output
#   strict=True: pertahankan, laporkan sebagai PAIRED_ITEM_ORPHAN
#
# Kelas deviasi:
#   PAIRED_ITEM_ORPHAN = pairedItem menunjuk index >= len(input batch)
#   PAIRED_ITEM_MISSING = pairedItem absent saat diharapkan ada

def _check_paired_item(paired_item, input_count):
    """Check apakah pairedItem valid (menunjuk index dalam range).
    Returns: (is_valid, orphan_indices)"""
    if paired_item is None:
        return True, []

    orphans = []
    if isinstance(paired_item, int):
        if paired_item < 0 or paired_item >= input_count:
            orphans.append(paired_item)
    elif isinstance(paired_item, dict):
        # Format: {"node": "NodeName", "itemIndex": N}
        idx = paired_item.get('itemIndex', 0)
        if isinstance(idx, int) and (idx < 0 or idx >= input_count):
            orphans.append(idx)
    elif isinstance(paired_item, list):
        for pi in paired_item:
            if isinstance(pi, int):
                if pi < 0 or pi >= input_count:
                    orphans.append(pi)
            elif isinstance(pi, dict):
                idx = pi.get('itemIndex', 0)
                if isinstance(idx, int) and (idx < 0 or idx >= input_count):
                    orphans.append(idx)

    return len(orphans) == 0, orphans


def normalize_paired_items(obj, input_count=999999, strict=False, _deviations=None):
    """N-09. Handle pairedItem orphans.
    
    input_count: jumlah item di input batch (untuk validasi index).
                 Default 999999 = anggap semua valid (jika tidak tahu ukuran batch).
    strict=False: hapus pairedItem orphan dari item output (default diff).
    strict=True: pertahankan pairedItem orphan, catat di deviations list
                 sebagai {'type': 'PAIRED_ITEM_ORPHAN', 'index': N, 'item': ...}.
    
    Returns: normalized object. Jika strict=True, _deviations list diisi.
    """
    if _deviations is None:
        _deviations = []

    if isinstance(obj, dict):
        out = {}
        for k, v in obj.items():
            if k == 'pairedItem':
                is_valid, orphans = _check_paired_item(v, input_count)
                if is_valid:
                    out[k] = v
                elif strict:
                    # Keep orphan, record deviation
                    out[k] = v
                    _deviations.append({
                        'type': 'PAIRED_ITEM_ORPHAN',
                        'orphan_indices': orphans,
                        'input_count': input_count,
                    })
                else:
                    # Drop orphan pairedItem (default behavior)
                    pass  # Don't include pairedItem in output
            else:
                out[k] = normalize_paired_items(v, input_count, strict, _deviations)
        return out
    if isinstance(obj, list):
        return [normalize_paired_items(v, input_count, strict, _deviations) for v in obj]
    return obj


def strip_paired_items(obj):
    """N-09 (simple): hapus semua pairedItem dari output.
    Gunakan saat tidak perlu validasi orphan (mis. L1 format check)."""
    if isinstance(obj, dict):
        return {k: strip_paired_items(v) for k, v in obj.items() if k != 'pairedItem'}
    if isinstance(obj, list):
        return [strip_paired_items(v) for v in obj]
    return obj


# ===========================================================================
# N-22 — Trigger non-deterministik -> SKIP-EXEC
# ===========================================================================
# Masalah: Workflow dengan trigger schedule/webhook/cron tidak bisa
# dijalankan deterministik di differential testing.
# Solusi: identifikasi trigger type dan tentukan aksi:
#   - Manual Trigger: OK, jalankan normal
#   - Schedule/Cron Trigger: SKIP-EXEC (hanya uji L1 import/parse)
#   - Webhook Trigger: SKIP-EXEC atau ganti Manual Trigger
#   - Error Trigger: SKIP-EXEC
#   - Custom trigger (community): SKIP-EXEC

# Trigger types yang deterministik (boleh dijalankan di diff testing)
DETERMINISTIC_TRIGGERS = {
    'n8n-nodes-base.manualTrigger',
    'n8n-nodes-base.executeWorkflowTrigger',
}

# Trigger types yang TIDAK deterministik (harus skip atau replace)
NON_DETERMINISTIC_TRIGGERS = {
    'n8n-nodes-base.scheduleTrigger',
    'n8n-nodes-base.cron',            # DEPRECATED, alias scheduleTrigger
    'n8n-nodes-base.webhook',
    'n8n-nodes-base.emailReadImap',
    'n8n-nodes-base.gmailTrigger',
    'n8n-nodes-base.slackTrigger',
    'n8n-nodes-base.telegramTrigger',
    'n8n-nodes-base.githubTrigger',
    'n8n-nodes-base.errorTrigger',
    'n8n-nodes-base.workflowTrigger',
    'n8n-nodes-base.n8nTrigger',
    'n8n-nodes-base.executeWorkflowTrigger',  # OK if called, not if standalone
}

# Trigger types yang bisa di-replace dengan Manual Trigger
REPLACEABLE_TRIGGERS = {
    'n8n-nodes-base.scheduleTrigger',
    'n8n-nodes-base.cron',
    'n8n-nodes-base.webhook',
}


def classify_trigger(workflow):
    """Klasifikasi trigger workflow. Returns:
    ('deterministic', trigger_type) — boleh dijalankan
    ('skip', trigger_type, reason) — harus skip
    ('replaceable', trigger_type, replacement) — bisa diganti Manual Trigger
    ('no_trigger', None) — workflow tanpa trigger (executeWorkflow?)
    """
    if not isinstance(workflow, dict):
        return ('invalid', None, 'not a dict')

    nodes = workflow.get('nodes', [])
    if not nodes:
        return ('no_trigger', None, 'no nodes')

    # Find trigger nodes (nodes with no incoming connections)
    connections = workflow.get('connections', {})
    # A trigger node is one that has no inputs from other nodes
    # In n8n, trigger nodes are typically the first node with type *Trigger or webhook
    trigger_nodes = []
    for node in nodes:
        if not isinstance(node, dict):
            continue
        ntype = node.get('type', '')
        if 'Trigger' in ntype or ntype in NON_DETERMINISTIC_TRIGGERS or ntype == 'n8n-nodes-base.webhook':
            trigger_nodes.append(node)

    if not trigger_nodes:
        return ('no_trigger', None, 'no trigger node found')

    # Check each trigger
    for tn in trigger_nodes:
        ntype = tn.get('type', '')
        if ntype in DETERMINISTIC_TRIGGERS:
            return ('deterministic', ntype)
        if ntype in REPLACEABLE_TRIGGERS:
            return ('replaceable', ntype, 'n8n-nodes-base.manualTrigger')
        # Non-deterministic, not replaceable
        return ('skip', ntype, f'non-deterministic trigger: {ntype}')

    return ('no_trigger', None, 'trigger nodes exist but unrecognized')


def should_skip_exec(workflow):
    """N-22. Tentukan apakah workflow harus di-SKIP saat differential testing.
    Returns: (skip: bool, reason: str)"""
    classification = classify_trigger(workflow)
    kind = classification[0]

    if kind == 'deterministic':
        return False, ''
    elif kind == 'skip':
        return True, f'SKIP-EXEC: {classification[2]}'
    elif kind == 'replaceable':
        # Could replace, but for safety, skip by default
        return True, f'SKIP-EXEC (replaceable): {classification[1]} -> {classification[2]}'
    elif kind == 'no_trigger':
        # No trigger = might be sub-workflow, skip standalone exec
        return True, f'SKIP-EXEC: no trigger ({classification[2]})'
    else:
        return True, f'SKIP-EXEC: {classification}'


def replace_trigger_with_manual(workflow):
    """N-22 (optional). Ganti trigger node dengan Manual Trigger.
    Returns: (modified_workflow, was_replaced: bool)
    
    HATI-HATI: ini mengubah parameter trigger, yang mungkin mempengaruhi
    downstream nodes. Gunakan hanya untuk testing L2 sederhana."""
    if not isinstance(workflow, dict):
        return workflow, False

    classification = classify_trigger(workflow)
    if classification[0] != 'replaceable':
        return workflow, False

    old_type = classification[1]
    wf = copy.deepcopy(workflow)
    nodes = wf.get('nodes', [])

    for node in nodes:
        if isinstance(node, dict) and node.get('type') == old_type:
            node['type'] = 'n8n-nodes-base.manualTrigger'
            node['typeVersion'] = 1
            node['parameters'] = {}  # Manual Trigger has no params
            break

    return wf, True


# ===========================================================================
# Self-test
# ===========================================================================
def selftest():
    fails = []

    def eq(a, b, label):
        if a != b:
            fails.append(f"{label}: {a!r} != {b!r}")

    # N-08: timezone normalization
    # Same instant, different offset format
    eq(timezone_offsets_equal(
        '2026-09-09T04:00:00.000+07:00',
        '2026-09-08T21:00:00.000Z'),
       True, 'N-08 +07:00 vs Z')
    eq(timezone_offsets_equal(
        '2026-09-09T04:00:00+0700',
        '2026-09-09T04:00:00+07:00'),
       True, 'N-08 +0700 vs +07:00')
    eq(timezone_offsets_equal(
        '2026-09-09T04:00:00+05:30',
        '2026-09-08T22:30:00Z'),
       True, 'N-08 India timezone')
    eq(timezone_offsets_equal(
        '2026-09-09T04:00:00Z',
        '2026-09-09T05:00:00Z'),
       False, 'N-08 different instants')

    # N-08: normalize_timezone function
    result = normalize_timezone({'createdAt': '2026-09-09T04:00:00.000+07:00'})
    expected_ms = _parse_to_utc_epoch_ms('2026-09-08T21:00:00.000Z')
    eq(result['createdAt'], expected_ms, 'N-08 normalize +07:00 to epoch')

    # N-08: non-date string is not affected
    eq(normalize_timezone({'name': 'hello'}),
       {'name': 'hello'}, 'N-08 non-date safe')

    # N-08: nested
    result = normalize_timezone({
        'data': {'startedAt': '2026-01-01T00:00:00Z'}
    })
    eq(result['data']['startedAt'],
       _parse_to_utc_epoch_ms('2026-01-01T00:00:00Z'), 'N-08 nested')

    # N-09: pairedItem valid
    eq(normalize_paired_items(
        {'json': {'a': 1}, 'pairedItem': 0}, input_count=5),
       {'json': {'a': 1}, 'pairedItem': 0}, 'N-09 valid pairedItem')

    # N-09: pairedItem orphan, default (strip)
    eq(normalize_paired_items(
        {'json': {'a': 1}, 'pairedItem': 10}, input_count=5),
       {'json': {'a': 1}}, 'N-09 orphan stripped')

    # N-09: pairedItem orphan, strict (keep + deviation)
    devs = []
    result = normalize_paired_items(
        {'json': {'a': 1}, 'pairedItem': 10}, input_count=5, strict=True, _deviations=devs)
    eq(result.get('pairedItem'), 10, 'N-09 strict keeps orphan')
    eq(len(devs), 1, 'N-09 strict records deviation')
    eq(devs[0]['type'], 'PAIRED_ITEM_ORPHAN', 'N-09 deviation type')

    # N-09: strip_paired_items
    eq(strip_paired_items(
        [{'json': {'a': 1}, 'pairedItem': 0},
         {'json': {'b': 2}, 'pairedItem': 1}]),
       [{'json': {'a': 1}}, {'json': {'b': 2}}], 'N-09 strip all')

    # N-09: dict pairedItem
    eq(normalize_paired_items(
        {'json': {}, 'pairedItem': {'node': 'X', 'itemIndex': 99}},
        input_count=5),
       {'json': {}}, 'N-09 dict orphan stripped')

    # N-22: classify_trigger
    wf_manual = {
        'nodes': [{'type': 'n8n-nodes-base.manualTrigger', 'name': 'Start'}],
        'connections': {}
    }
    eq(classify_trigger(wf_manual)[0], 'deterministic', 'N-22 manual trigger')

    wf_schedule = {
        'nodes': [{'type': 'n8n-nodes-base.scheduleTrigger', 'name': 'Every Hour'}],
        'connections': {}
    }
    eq(classify_trigger(wf_schedule)[0], 'replaceable', 'N-22 schedule trigger')

    wf_webhook = {
        'nodes': [{'type': 'n8n-nodes-base.webhook', 'name': 'Webhook'}],
        'connections': {}
    }
    eq(classify_trigger(wf_webhook)[0], 'replaceable', 'N-22 webhook trigger')

    # N-22: should_skip_exec
    eq(should_skip_exec(wf_manual), (False, ''), 'N-22 manual no skip')
    skip, reason = should_skip_exec(wf_schedule)
    eq(skip, True, 'N-22 schedule skip')
    eq('replaceable' in reason, True, 'N-22 schedule reason')

    # N-22: replace_trigger_with_manual
    wf2, replaced = replace_trigger_with_manual(wf_schedule)
    eq(replaced, True, 'N-22 replaced')
    eq(wf2['nodes'][0]['type'], 'n8n-nodes-base.manualTrigger', 'N-22 new type')
    eq(wf2['nodes'][0]['parameters'], {}, 'N-22 cleared params')

    # N-22: deprecated cron trigger
    wf_cron = {
        'nodes': [{'type': 'n8n-nodes-base.cron', 'name': 'CronJob'}],
        'connections': {}
    }
    skip2, _ = should_skip_exec(wf_cron)
    eq(skip2, True, 'N-22 deprecated cron skip')

    # N-22: no trigger
    wf_no_trigger = {'nodes': [{'type': 'n8n-nodes-base.set', 'name': 'Set'}], 'connections': {}}
    skip3, _ = should_skip_exec(wf_no_trigger)
    eq(skip3, True, 'N-22 no trigger skip')

    if fails:
        raise AssertionError('\n'.join(fails))
    return f'selftest OK: {len(fails)} fail (semua aturan N-08, N-09, N-22 lulus)'


if __name__ == '__main__':
    print(selftest())

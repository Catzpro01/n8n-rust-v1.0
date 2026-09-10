#!/usr/bin/env python3
"""Aturan normalisasi milik agent1 untuk differential harness (mandat fern #168/#170).

Pembagian (AGENT4-NORMALIZER-DESIGN.md #179, dikonfirmasi agent1):
  agent1 -> N-06 (orderless arrays), N-10 (binary metadata), N-12 (error compare)
  agent4 -> N-01..N-05, N-07, N-11 (modul terpisah, belum tersedia saat file ini ditulis)
  agent3 -> N-08, N-09, N-22 (modul terpisah)

Aturan milik pihak lain diimpor secara opsional; bila belum ada, driver
mendapat NormRuleMissing yang jelas (fail loudly, bukan diam-diam lolos).
"""

import hashlib

RULE_OWNER = {
    "N-01": "agent4", "N-02": "agent4", "N-03": "agent4", "N-04": "agent4",
    "N-05": "agent4", "N-06": "agent1", "N-07": "agent4", "N-08": "agent3",
    "N-09": "agent3", "N-10": "agent1", "N-11": "agent4", "N-12": "agent1",
    "N-20": "agent4", "N-21": "agent4", "N-22": "agent3",
}


class NormRuleMissing(NotImplementedError):
    """Aturan diminta tapi modul pemiliknya belum tersedia."""


def rule_owner(rule_id):
    return RULE_OWNER.get(rule_id, "UNKNOWN")


# ---------------------------------------------------------------- N-06
# Array item hanya disortir bila tipe node terdaftar eksplisit sebagai
# orderless. Default: urutan DIPERTAHANKAN (data-driven, bukan tebakan).
# Kunci sortir = representasi kanonis item (stabil, deterministik).
# ---------------------------------------------------------------------------
def sort_items_if_orderless(items, node_type, orderless_registry):
    """N-06. Kembalikan (items_baru, disortir_bool)."""
    if not isinstance(items, list):
        return items, False
    if node_type not in (orderless_registry or set()):
        return items, False
    import json as _json
    keyed = [(_json.dumps(it, sort_keys=True, ensure_ascii=True,
                          separators=(",", ":")), i, it)
             for i, it in enumerate(items)]
    keyed.sort(key=lambda t: (t[0], t[1]))
    return [t[2] for t in keyed], True


# ---------------------------------------------------------------- N-10
# Binary: jangan bawa byte mentah ke bentuk kanonis (bisa MB). Ganti dengan
# metadata + sha256 isi sehingga kesetaraan tetap terverifikasi.
# ---------------------------------------------------------------------------
def normalize_binary(value):
    """N-10. Normalisasi satu nilai binary n8n (dict dengan 'data' base64)."""
    if not isinstance(value, dict) or "data" not in value:
        return value
    raw = value["data"]
    if isinstance(raw, str):
        digest = hashlib.sha256(raw.encode("ascii", "ignore")).hexdigest()
    else:
        digest = hashlib.sha256(bytes(str(raw), "utf-8")).hexdigest()
    out = {"__binary_sha256": digest}
    for k in ("fileName", "mimeType", "fileSize", "fileType", "fileExtension"):
        if k in value:
            out[k] = value[k]
    return out


def normalize_item_binary(item):
    """N-10. Terapkan ke seluruh blok 'binary' satu INodeExecutionData."""
    if not isinstance(item, dict):
        return item
    binary = item.get("binary")
    if not isinstance(binary, dict):
        return item
    item = dict(item)
    item["binary"] = {k: normalize_binary(v) for k, v in binary.items()}
    return item


# ---------------------------------------------------------------- N-12
# Error: bandingkan message + code/name saja. Stack trace & field runtime
# (timestamp, execution id) dibuang karena tidak deterministik.
# ---------------------------------------------------------------------------
_ERROR_KEEP = ("message", "code", "name", "node", "httpCode", "statusCode")


def normalize_error(value):
    """N-12. Reduksi satu objek error ke field pembanding."""
    if not isinstance(value, dict):
        return value
    return {k: value[k] for k in _ERROR_KEEP if k in value}


def normalize_item_error(item):
    """N-12. Terapkan ke field 'error' satu INodeExecutionData."""
    if not isinstance(item, dict) or "error" not in item:
        return item
    item = dict(item)
    item["error"] = normalize_error(item["error"])
    return item

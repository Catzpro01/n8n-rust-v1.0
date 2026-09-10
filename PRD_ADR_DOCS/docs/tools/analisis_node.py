#!/usr/bin/env python3
"""
Analisis korpus template n8n -> penentuan daftar node berbasis bukti.

Sumber data : /opt/agent-workspace/docs/corpus/tpl-*.json (VPS)
              salinan lokal di ./tpl-*.json
Asal data   : api.n8n.io/api/templates/workflows/<id> (agent4, Fase 3)
Verifikasi  : 10/10 sampel byte-identik dgn API (VERIFIKASI-DELIVERABLE-TIM.md S5)

Cara pakai  : python3 analisis_node.py
Output      : stdout (tabel) + FREQ.json (data mentah)

Catatan metodologi:
  - Metrik utama adalah "workflow jalan penuh" = SEMUA tipe node dalam workflow
    ada di set yang didukung. Satu node tak didukung => seluruh workflow gagal
    impor. Ini metrik yang menentukan gate Fase 1, bukan metrik kecantikan.
  - stickyNote DIKECUALIKAN dari semua perhitungan: ia anotasi kanvas, bukan
    node eksekusi. Tapi ia TETAP harus bisa di-parse saat impor (lihat Temuan 1).
  - Greedy set-cover adalah HEURISTIK, bukan optimal (set cover NP-hard).
    Angka "node ke-N" adalah batas atas efisiensi, bukan janji.
"""
import json, glob, collections, io, sys

STICKY = 'stickyNote'

# Node LangChain / AI. Dipakai untuk memisahkan dua pasar (Temuan 5).
LANGCHAIN = {
    'agent', 'lmChatOpenAi', 'openAi', 'outputParserStructured', 'memoryBufferWindow',
    'chainLlm', 'documentDefaultDataLoader', 'lmChatAnthropic', 'vectorStorePinecone',
    'agentTool', 'chatTrigger', 'embeddingsOpenAi',
    'textSplitterRecursiveCharacterTextSplitter', 'toolWorkflow', 'vectorStoreInMemory',
    'rerankerCohere', 'memoryPostgresChat', 'llmChain', 'summarizationChain',
}

# Node deprecated yang masih muncul di template publik (Temuan 4).
DEPRECATED = {'function', 'cron', 'functionItem'}

# Node yang wajib ada berapapun gain marginalnya, karena tanpanya sebuah
# KAPABILITAS produk hilang total (bukan sekadar satu workflow gagal).
# Greedy murni melewatkan ini -- lihat Temuan 2.
CAPABILITY_CRITICAL = {
    'manualTrigger':     'entry point dasar',
    'scheduleTrigger':   'otomasi terjadwal -- tanpanya TIDAK ADA cron sama sekali',
    'webhook':           'otomasi event-driven',
    'httpRequest':       'pintu ke semua API eksternal',
    'set':               'transform dasar',
    'if':                'branching',
    'switch':            'multi-branch',
    'merge':             'gabung branch',
    'code':              'escape hatch -- tanpa ini pengguna buntu',
    'noOp':              'placeholder',
    'wait':              'delay / retry / rate-limit',
    'respondToWebhook':  'pasangan wajib webhook',
    'errorTrigger':      'error workflow',
    'executeWorkflow':   'sub-workflow',
    'filter':            'seleksi item',
    'splitOut':          'ekspansi array',
    'limit':             'paginasi',
    'cron':              'ALIAS deprecated -> scheduleTrigger',
    'function':          'ALIAS deprecated -> code',
    'functionItem':      'ALIAS deprecated -> code',
}


def load(pattern='tpl-*.json'):
    """Muat korpus. Menangani format flat maupun envelope API n8n.io."""
    wf = {}
    for f in sorted(glob.glob(pattern)):
        try:
            d = json.load(open(f, encoding='utf-8'))
        except Exception as e:
            print(f"  !! gagal parse {f}: {e}", file=sys.stderr)
            continue
        w = d.get('workflow', d)
        # envelope API: {workflow: {metadata..., workflow: {nodes, connections}}}
        if isinstance(w, dict) and 'workflow' in w:
            w = w['workflow']
        nodes = w.get('nodes') or []
        if not nodes:
            continue
        types = set()
        for n in nodes:
            t = (n.get('type') or '?').split('.')[-1]   # buang prefix n8n-nodes-base.
            if t != STICKY:
                types.add(t)
        if types:
            wf[f] = types
    return wf


def greedy(target, k, start=None):
    """
    Pilih k node yang memaksimumkan jumlah workflow jalan penuh.

    DETERMINISTIK: iterasi sorted(pool), bukan set. Set Python tidak punya
    urutan stabil antar-proses karena PYTHONHASHSEED, jadi greedy yang
    mengiterasi set menghasilkan angka berbeda tiap run (audit agent1 #309
    TEMUAN A: 147/161/153/149/161 untuk "node ke-90%"). Tie-break juga
    eksplisit: gain tertinggi, lalu urutan alfabet.

    `start` = himpunan awal. Bila None, mulai dari KOSONG -- artinya angka
    "node ke-N" adalah TOTAL node yang harus diimplementasi, dan TIDAK BOLEH
    di-offset lagi. (Bug lama: hit = i+26 padahal greedy mulai kosong dan
    pool sudah mencakup 26 node itu -> 187, melebihi universe 185.)
    """
    cur = set(start) if start else set()
    hist = []
    pool = {t for ns in target.values() for t in ns} - cur
    while len(hist) < k and pool:
        best, bg = None, -1
        for t in sorted(pool):                    # deterministik
            g = sum(1 for ns in target.values() if ns <= (cur | {t}))
            if g > bg:                            # strict >: tie-break alfabet
                bg, best = g, t
        if best is None:
            break
        cur.add(best)
        pool.discard(best)
        hist.append((best, bg))
    return cur, hist


def build_recommended(freq, byfreq, k):
    """
    Rekomendasi final: capability-first + frekuensi + ambang anti-overfit.

    Greedy murni overfit (memilih bitwarden/bubble/copper yang masing-masing
    hanya menutup 1 workflow). Frekuensi murni underperform (melewatkan node
    yang jarang tapi selalu jadi satu-satunya penghalang). Gabungan keduanya.
    """
    s = [t for t in CAPABILITY_CRITICAL if t in freq]
    for t in byfreq:                       # isi sisa dgn frekuensi tertinggi
        if len(s) >= k:
            break
        if t in s or t in LANGCHAIN:
            continue
        if freq[t] <= 2 and t not in CAPABILITY_CRITICAL:
            continue                       # anti-overfit
        s.append(t)
    for t in byfreq:                       # longgarkan bila masih kurang
        if len(s) >= k:
            break
        if t not in s and t not in LANGCHAIN:
            s.append(t)
    return s[:k]


def main():
    wf = load()
    N = len(wf)
    if not N:
        print("Korpus kosong. Jalankan dari direktori berisi tpl-*.json", file=sys.stderr)
        return 1

    filec = collections.Counter()      # berapa FILE memuat tipe ini
    instc = collections.Counter()      # berapa INSTANCE tipe ini
    for ns in wf.values():
        for t in ns:
            filec[t] += 1
    # instance butuh hitung ulang dari file mentah (set sudah dedup)
    for f in sorted(glob.glob('tpl-*.json')):
        try:
            d = json.load(open(f, encoding='utf-8'))
        except Exception:
            continue
        w = d.get('workflow', d)
        if isinstance(w, dict) and 'workflow' in w:
            w = w['workflow']
        for n in (w.get('nodes') or []):
            t = (n.get('type') or '?').split('.')[-1]
            if t != STICKY:
                instc[t] += 1

    instances = sum(instc.values())
    byfreq = [t for t, _ in filec.most_common()]

    print("=" * 68)
    print("DATA DASAR KORPUS")
    print("=" * 68)
    print(f"  template valid            : {N}")
    print(f"  instance node (non-sticky): {instances}")
    print(f"  tipe node unik            : {len(filec)}")
    sticky_files = sum(1 for f in glob.glob('tpl-*.json')
                       for _ in [0] if STICKY in _raw(f))
    print(f"  file memuat stickyNote    : {sticky_files} "
          f"({100 * sticky_files / N:.0f}%)")
    ai = {f: ns for f, ns in wf.items() if ns & LANGCHAIN}
    nonai = {f: ns for f, ns in wf.items() if not (ns & LANGCHAIN)}
    print(f"  file memuat node AI/LC    : {len(ai)} ({100 * len(ai) / N:.0f}%)")
    tail2 = [t for t, c in filec.items() if c <= 2]
    tail1 = [t for t, c in filec.items() if c == 1]
    print(f"  tipe muncul di <=2 file   : {len(tail2)} ({100 * len(tail2) / len(filec):.0f}%)")
    print(f"  tipe muncul di TEPAT 1    : {len(tail1)} ({100 * len(tail1) / len(filec):.0f}%)")

    print()
    print("=" * 68)
    print("TOP 35 TIPE NODE (berdasarkan jumlah FILE yang memuat)")
    print("=" * 68)
    print(f"{'#':>3} {'tipe node':<32}{'file':>6}{'inst':>6}{'%file':>7}")
    print("-" * 60)
    for i, t in enumerate(byfreq[:35], 1):
        c = filec[t]
        print(f"{i:>3} {t:<32}{c:>6}{instc[t]:>6}{100 * c / N:>6.0f}%")

    # ---- rekomendasi 26 node
    rec = build_recommended(filec, byfreq, 26)
    rec_set = set(rec)
    full = sum(1 for ns in wf.values() if ns <= rec_set)
    # instance SEJATI (bukan slot unik). wf[f] adalah SET, jadi len(ns & rec_set)
    # menghitung tipe unik -- membandingkannya ke `instances` (true count) adalah
    # apel vs jeruk. Pakai instc yang dihitung dari file mentah.
    inst_cov = sum(c for tname, c in instc.items() if tname in rec_set)
    slot_cov = sum(len(ns & rec_set) for ns in wf.values())
    slots = sum(len(ns) for ns in wf.values())

    print()
    print("=" * 68)
    print("TEMUAN 3: KESENJANGAN instance-coverage vs workflow-coverage")
    print("=" * 68)
    print(f"  instance node SEJATI tercakup : {inst_cov}/{instances}"
          f" = {100 * inst_cov / instances:.0f}%")
    print(f"  slot unik per workflow        : {slot_cov}/{slots}"
          f" = {100 * slot_cov / slots:.0f}%")
    print(f"  tipe unik tercakup            : {len(rec_set & set(filec))}/{len(filec)}"
          f" = {100 * len(rec_set & set(filec)) / len(filec):.0f}%")
    print(f"  WORKFLOW JALAN PENUH          : {full}/{N} = {100 * full / N:.0f}%")
    print(f"  -> selisih instance({100 * inst_cov / instances:.0f}%) vs workflow"
          f"({100 * full / N:.0f}%) = {100 * inst_cov / instances - 100 * full / N:.0f} poin"
          f" = harga long-tail")

    # ---- distribusi gap
    print()
    print("=" * 68)
    print("TEMUAN 2: berapa node tambahan agar tiap workflow jalan penuh")
    print("=" * 68)
    gap = collections.Counter()
    blocking = collections.Counter()
    for ns in wf.values():
        g = ns - rec_set
        gap[len(g)] += 1
        if len(g) == 1:
            blocking[next(iter(g))] += 1
    cum = 0
    for g in range(0, 9):
        cum += gap.get(g, 0)
        print(f"  butuh +{g} node : {gap.get(g, 0):>3} wf   kumulatif {cum:>3} "
              f"({100 * cum / N:>3.0f}%)  {'#' * gap.get(g, 0)}")
    print(f"  butuh +9 / lebih : {sum(v for k, v in gap.items() if k >= 9)} wf")
    print(f"\n  node penghalang bagi kelompok '+1': {len(blocking)} tipe BERBEDA")
    only1 = sum(1 for c in blocking.values() if c == 1)
    print(f"  yang hanya menutup 1 workflow     : {only1}/{len(blocking)} "
          f"({100 * only1 / len(blocking):.0f}%)")
    print("  -> KURVA MARGINAL DATAR. Tidak ada shortcut.")

    # ---- deprecated
    print()
    print("=" * 68)
    print("TEMUAN 4: node deprecated (bukti untuk T-12)")
    print("=" * 68)
    for t in sorted(DEPRECATED):
        n = sum(1 for ns in wf.values() if t in ns)
        print(f"  {t:<14} {n:>3} workflow ({100 * n / N:.0f}%) gagal impor bila ditolak")
    nd = sum(1 for ns in wf.values() if ns & DEPRECATED)
    print(f"  TOTAL memuat >=1 deprecated: {nd}/{N} ({100 * nd / N:.0f}%)")

    # ---- dua pasar
    print()
    print("=" * 68)
    print("TEMUAN 5: dua pasar berbeda (AI vs non-AI)")
    print("=" * 68)
    print(f"  {'pasar':<10}{'workflow':>9}{'node unik':>11}{'node u/ 90%':>13}")
    for name, tgt in (('AI', ai), ('non-AI', nonai)):
        _, h = greedy(tgt, 200)
        at90 = next((i for i, (t, g) in enumerate(h, 1)
                     if 100 * g / len(tgt) >= 90), None)
        print(f"  {name:<10}{len(tgt):>9}"
              f"{len({t for ns in tgt.values() for t in ns}):>11}"
              f"{str(at90) if at90 else '>200':>13}")
    only_ai = ({t for ns in ai.values() for t in ns}
               - {t for ns in nonai.values() for t in ns})
    print(f"  node HANYA dibutuhkan pasar AI: {len(only_ai)}")

    # ---- tabel biaya coverage
    print()
    print("=" * 68)
    print("TEMUAN 6: biaya mencapai target coverage (semua workflow)")
    print("=" * 68)
    # greedy mulai KOSONG -> i+1 adalah TOTAL node. Tanpa offset.
    _, hist = greedy(wf, 185)
    universe = len(filec)
    print(f"  (greedy mulai dari himpunan kosong; universe = {universe} tipe)")
    print(f"  {'target':>8}{'total node':>12}{'node terakhir':>22}")
    for target in (20, 30, 45, 60, 75, 90):
        hit = next(((i + 1, t) for i, (t, g) in enumerate(hist)
                    if 100 * g / N >= target), None)
        if hit:
            assert hit[0] <= universe, f"overcount: {hit[0]} > {universe}"
            print(f"  {target:>7}%{hit[0]:>12}{hit[1]:>22}")
        else:
            print(f"  {target:>7}%{'tak tercapai':>12}")

    print()
    print("=" * 68)
    print(f"REKOMENDASI {len(rec)} NODE (capability-first + frekuensi + anti-overfit)")
    print("=" * 68)
    for t in sorted(rec):
        tag = ' [ALIAS]' if t in DEPRECATED else ''
        why = CAPABILITY_CRITICAL.get(t, f'frekuensi {filec[t]} file')
        print(f"  {t:<24}{why}{tag}")

    io.open('FREQ.json', 'w', encoding='utf-8').write(json.dumps({
        'n_templates': N, 'n_instances': instances, 'n_unique': len(filec),
        'sticky_files': sticky_files, 'ai_files': len(ai),
        'filec': dict(filec.most_common()), 'inst': dict(instc.most_common()),
        'recommended_26': sorted(rec), 'full_coverage_26': full,
    }, indent=1, ensure_ascii=False))
    print("\n  -> FREQ.json ditulis")
    return 0


def _raw(f):
    """Bantu hitung stickyNote dari file mentah."""
    try:
        d = json.load(open(f, encoding='utf-8'))
    except Exception:
        return set()
    w = d.get('workflow', d)
    if isinstance(w, dict) and 'workflow' in w:
        w = w['workflow']
    return {(n.get('type') or '?').split('.')[-1] for n in (w.get('nodes') or [])}


if __name__ == '__main__':
    sys.exit(main())

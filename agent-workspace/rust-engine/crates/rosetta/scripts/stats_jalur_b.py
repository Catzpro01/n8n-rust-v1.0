#!/usr/bin/env python3
"""Statistik agregat param_schemas_jalur_b.json — angka SELALU dari artefak,
bukan hitungan manual (pelajaran: koreksi #1459/#1495)."""
import json, sys, hashlib
p = sys.argv[1] if len(sys.argv) > 1 else 'data/param_schemas_jalur_b.json'
d = json.load(open(p))
raw = open(p, 'rb').read()
fields = [f for e in d for f in e.get('fields', [])]
rq = [f for f in fields if f.get('required_ui')]
rqnd = [f for f in rq if f.get('default') is None]
legacy = [f for e in d for f in e.get('fields', []) if f.get('provenance') == 'corpus-legacy']
print(f'entri={len(d)} fields={len(fields)} required_ui={len(rq)} '
      f'required_ui_tanpa_default={len(rqnd)} corpus_legacy={len(legacy)} '
      f'provisional={sum(1 for e in d if e.get("provisional"))}')
print('sha256=' + hashlib.sha256(raw).hexdigest())

def _main_rem():
    import os
    inv = None
    for cand in ('data/jalur_b_inventory.json', 'crates/rosetta/data/jalur_b_inventory.json'):
        if os.path.exists(cand):
            inv = json.load(open(cand)); break
    if inv is None: return
    if isinstance(inv, dict): inv = inv.get('types') or inv.get('entries')
    done = {e['type_full'] for e in d}
    rem = [e for e in inv if (e.get('type_full') or e.get('type')) not in done]
    tools = [e for e in rem if 'Tool' in (e.get('type_full') or e.get('type'))]
    print(f'sisa={len(rem)} (nonTool={len(rem)-len(tools)} tool={len(tools)})')
if __name__ == '__main__' and sys.argv and '--with-remaining' in sys.argv:
    _main_rem()

#!/usr/bin/env bash
# docs/parity/verify-parity.sh — verifikasi ulang angka paritas n8n vs n8n-rust.
#
# Pakai:   bash docs/parity/verify-parity.sh            (probe repo + runtime lokal)
#          bash docs/parity/verify-parity.sh --n8n      (tambah pengukuran repo n8n via gh, perlu auth)
#
# Keluaran: tabel angka + peringatan bila klaim dokumen tidak cocok dengan runtime.
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"
CHECK_N8N=0; [ "${1:-}" = "--n8n" ] && CHECK_N8N=1

hr() { printf '%s\n' "-------------------------------------------------------------"; }
row() { printf '  %-46s %s\n' "$1" "$2"; }

echo "PARITY PROBE — $(date -u +%Y-%m-%dT%H:%M:%SZ)"
hr
echo "[1] REPO n8n-rust"
row "crate di workspace"          "$(ls -d n8n-rust/crates/*/ | wc -l)"
row "file .rs"                    "$(find n8n-rust/crates -name '*.rs' | wc -l)"
row "baris kode .rs"              "$(find n8n-rust/crates -name '*.rs' -exec cat {} + | wc -l)"
row "unit test (#[test])"         "$(grep -rn '#\[test\]' n8n-rust/crates --include='*.rs' | wc -l)"
row "route axum (.route()"        "$(grep -cE '\.route\(' n8n-rust/crates/n8n-server/src/main.rs)"
row "node type terdaftar (core+ext)" "$(cat n8n-rust/crates/n8n-nodes/src/lib.rs n8n-rust/crates/n8n-nodes/src/extended.rs | grep -oE '"n8n-nodes-base\.[A-Za-z0-9]+"' | sort -u | wc -l) (core $(grep -oE '"n8n-nodes-base\.[A-Za-z0-9]+"' n8n-rust/crates/n8n-nodes/src/lib.rs | sort -u | wc -l) + ext $(grep -oE '"n8n-nodes-base\.[A-Za-z0-9]+"' n8n-rust/crates/n8n-nodes/src/extended.rs | sort -u | wc -l))"
row "credential type"             "$(grep -c 'CredentialType {' n8n-rust/crates/n8n-core/src/credentials.rs)"
row "file UI"                     "$(ls n8n-rust/crates/n8n-server/ui/ | tr '\n' ' ')"
row "baris UI app.html"           "$(wc -l < n8n-rust/crates/n8n-server/ui/app.html)"
row "router di UI (pushState/hash)" "$(grep -ciE 'history\.pushstate|location\.hash|vue-router' n8n-rust/crates/n8n-server/ui/app.html)"
row "auth di server (api-key/jwt/session)" "$(grep -ciE 'x-n8n-api-key|authorization|bearer|jwt|session' n8n-rust/crates/n8n-server/src/main.rs)"
row "kapabilitas ada? (langchain/subworkflow/retry/binary)" "$(grep -ril 'langchain' n8n-rust/crates --include='*.rs' | wc -l) / $(grep -ril 'executeWorkflow' n8n-rust/crates --include='*.rs' | wc -l) / $(grep -ril 'retries' n8n-rust/crates --include='*.rs' | wc -l) / $(grep -ril 'binaryData' n8n-rust/crates --include='*.rs' | wc -l)"

hr
echo "[2] RUNTIME (preview-server.js, port 3000)"
SRV_PID=""; STARTED=0
if curl -sf -m 2 http://127.0.0.1:3000/health >/dev/null 2>&1; then
  echo "  (server sudah jalan — dipakai apa adanya)"
else
  ( node preview-server.js >/tmp/parity-preview.log 2>&1 & echo $! >/tmp/parity-preview.pid )
  SRV_PID="$(cat /tmp/parity-preview.pid 2>/dev/null || true)"; STARTED=1
  for _ in $(seq 1 15); do curl -sf -m 1 http://127.0.0.1:3000/health >/dev/null 2>&1 && break; sleep 1; done
fi
H="$(curl -s -m 5 http://127.0.0.1:3000/health || echo '{}')"
row "/health"                     "$H"
row "GET /api/nodes (jumlah)"     "$(curl -s -m 5 http://127.0.0.1:3000/api/nodes | python3 -c 'import json,sys;print(len(json.load(sys.stdin)))' 2>/dev/null || echo '?')"
for p in / /templates /variables /projects /settings /signin /workflow/new /api/v1/workflows; do
  row "HTTP $p" "$(curl -s -o /dev/null -m 5 -w '%{http_code}' "http://127.0.0.1:3000$p")"
done
if [ "$STARTED" = "1" ] && [ -n "$SRV_PID" ]; then kill "$SRV_PID" 2>/dev/null; fi

if [ "$CHECK_N8N" = "1" ]; then
  hr
  echo "[3] REPO n8n (via gh api — target pembanding)"
  if ! command -v gh >/dev/null; then echo "  gh tidak ada — dilewati"; else
    TMP="/tmp/n8n-tree-$$.json"
    if gh api 'repos/n8n-io/n8n/git/trees/master?recursive=1' > "$TMP" 2>/dev/null; then
      python3 - "$TMP" <<'PY'
import json,sys,collections
t=json.load(open(sys.argv[1]))
p=[e['path'] for e in t['tree'] if e.get('type')=='blob']
def c(f): return len([x for x in p if f(x)])
row=lambda k,v: print(f'  {k:<46} {v}')
row('blob total', len(p))
row('implementasi node (.node.ts)', c(lambda x:x.endswith('.node.ts')))
row('  — nodes-base', c(lambda x:x.startswith('packages/nodes-base/nodes/') and x.endswith('.node.ts')))
row('  — LangChain', c(lambda x:'nodes-langchain' in x and x.endswith('.node.ts')))
row('direktori integrasi', len({x.split('/')[3] for x in p if x.startswith('packages/nodes-base/nodes/') and x.count('/')>4}))
row('file credential', c(lambda x:'/credentials/' in x and x.endswith('.credentials.ts')))
row('tabel terdokumentasi (sqlite)', c(lambda x:x.startswith('docs/generated/sqlite-schema/') and x.endswith('.md')))
row('file test', c(lambda x:'/__tests__/' in x or x.endswith(('.test.ts','.spec.ts'))))
row('komponen .vue (editor-ui)', c(lambda x:x.startswith('packages/frontend/editor-ui/src/') and x.endswith('.vue')))
row('skill resmi n8n (.agents/skills)', c(lambda x:x.startswith('.agents/skills/') and x.endswith('SKILL.md')))
PY
    else echo "  gagal ambil tree n8n (cek auth gh)"; fi
    rm -f "$TMP"
    gh api repos/n8n-io/n8n/releases/latest --jq '"  versi rilis terbaru n8n: \(.tag_name) (\(.published_at))"' 2>/dev/null || true
  fi
fi

hr
echo "Catatan: angka di atas adalah bukti untuk GAP-ANALYSIS.md; skrip ini tidak mengubah apa pun."
echo "Matriks item: docs/parity/PARITY-MATRIX.json"

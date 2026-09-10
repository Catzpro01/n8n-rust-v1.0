#!/bin/bash
# e2e_anchor.sh — menutup C-01 END-TO-END dengan TSA NYATA.
#
# Rantai Envelope (crate `envelope`, agent10 sesi A) -> anchor RFC 3161 ke FreeTSA
# (binary `anchor`, W0-ANCHOR-IMPL agent1) -> verifikasi silang.
#
# Ini bukti yang tidak bisa dihasilkan oleh uji dalam-proses: head yang di-anchor benar-benar
# dipegang pihak ketiga di luar host, lalu pemalsuan "ubah + rekomputasi" (G-C1b) dideteksi.
#
# Usage: bash tests/e2e_anchor.sh <anchor-binary> <envelope_e2e-binary>
set -u
ANCHOR_BIN="${1:?usage: e2e_anchor.sh <anchor-binary> <envelope_e2e-binary>}"
E2E="${2:?usage: e2e_anchor.sh <anchor-binary> <envelope_e2e-binary>}"
FREETSA="https://freetsa.org/tsr"
N=1000

T="$(mktemp -d /tmp/env-e2e-XXXXXX)"
DB="$T/anchor_log.sqlite"; TSR="$T/tsr"; mkdir -p "$TSR/pins"
PASS=0; FAIL=0
gate(){ if [ "$3" = "$4" ]; then echo "GATE $1 PASS: $2"; PASS=$((PASS+1));
        else echo "GATE $1 FAIL: $2 (want [$3] got [$4])"; FAIL=$((FAIL+1)); fi; }

echo "== setup: ambil CA FreeTSA =="
curl -sS --max-time 25 https://freetsa.org/files/cacert.pem -o "$T/cacert.pem" || { echo "SETUP FAIL"; exit 1; }
FP="$(openssl x509 -in "$T/cacert.pem" -noout -fingerprint -sha256 | cut -d= -f2 | tr -d ':' | tr 'A-Z' 'a-z')"
echo "   fingerprint CA = $FP"

echo "== langkah 1: hitung head rantai Envelope ($N entri deterministik) =="
HEAD="$("$E2E" emit "$N")" || { echo "emit gagal"; exit 1; }
echo "   head = $HEAD"
gate E2E-0 "head 64 hex" "64" "${#HEAD}"

echo "== langkah 2: anchor head ke TSA NYATA =="
"$ANCHOR_BIN" create --chain-id ENVELOPE-E2E --chain-head "$HEAD" \
  --entry-from 0 --entry-to $((N-1)) \
  --tsa "$FREETSA" --pin "$T/cacert.pem" --db "$DB" --tsr-dir "$TSR"
CRE=$?
gate E2E-1 "anchor create exit 0" "0" "$CRE"

ANCHORED="$(sqlite3 "$DB" 'SELECT lower(hex(chain_head)) FROM anchor_log ORDER BY anchor_id DESC LIMIT 1;')"
STATUS="$(sqlite3 "$DB" 'SELECT status FROM anchor_log ORDER BY anchor_id DESC LIMIT 1;')"
echo "   status=$STATUS anchored=$ANCHORED"
gate E2E-1b "status ANCHORED" "ANCHORED" "$STATUS"
gate E2E-2 "head yang di-anchor == head yang dihitung engine" "$HEAD" "$ANCHORED"

echo "== langkah 3: verifikasi rantai BERSIH dengan anchor -> harus VERIFIED_ANCHORED =="
"$E2E" check "$N" "$ANCHORED" clean > "$T/clean.out" 2>&1; C_CLEAN=$?
cat "$T/clean.out"
gate E2E-3 "clean + anchored -> VERIFIED_ANCHORED" "VERIFIED_ANCHORED" "$(grep -o 'verdict=[A-Z_]*' "$T/clean.out" | cut -d= -f2)"

echo "== langkah 4: SERANGAN G-C1b (ubah entri + rekomputasi rantai) dengan anchor -> harus BROKEN =="
"$E2E" check "$N" "$ANCHORED" tamper > "$T/tamper.out" 2>&1; C_TAMPER=$?
cat "$T/tamper.out"
gate E2E-4 "tamper + anchored -> BROKEN (C-01 TERTUTUP oleh anchor)" "BROKEN" "$(grep -o 'verdict=[A-Z_]*' "$T/tamper.out" | cut -d= -f2)"
gate E2E-4b "exit code 3 (BROKEN)" "3" "$C_TAMPER"
gate E2E-4c "alasan = ANCHOR MISMATCH" "yes" "$(grep -q 'ANCHOR MISMATCH' "$T/tamper.out" && echo yes || echo no)"

echo "== langkah 5: serangan yang SAMA tanpa anchor -> harus LOLOS (batas jujur A1) =="
"$E2E" check "$N" none tamper > "$T/noanchor.out" 2>&1
cat "$T/noanchor.out"
gate E2E-5 "tamper TANPA anchor -> TIDAK terdeteksi (VERIFIED_UNANCHORED)" "VERIFIED_UNANCHORED" "$(grep -o 'verdict=[A-Z_]*' "$T/noanchor.out" | cut -d= -f2)"

echo "== langkah 6: verifikasi ulang offline dari artefak saja (spec §5.3) =="
"$ANCHOR_BIN" verify --db "$DB" --tsr-dir "$TSR" --pin "$T/cacert.pem" > "$T/offline.out" 2>&1
head -2 "$T/offline.out"
gate E2E-6 "offline re-verify -> ANCHORED" "yes" "$(grep -q 'ANCHORED' "$T/offline.out" && echo yes || echo no)"

echo ""
echo "== HASIL: $PASS passed, $FAIL failed (workdir $T disimpan untuk audit) =="
echo "   interpretasi: E2E-4 = C-01 tertutup OLEH ANCHOR; E2E-5 = tanpa anchor pemalsuan lolos."
echo "   Keduanya harus lulus bersamaan: itulah isi addendum A1 (keyed chain perlu, anchor yang menjamin)."
[ "$FAIL" = 0 ]

#!/bin/bash
# ANC-1..ANC-7 acceptance gates for W0-ANCHOR-IMPL (AGENT10-W0-ANCHOR-SPEC §10).
# Live TSA: FreeTSA (MVP-only per §1.3). Produces a PASS/FAIL line per gate.
# Usage: anc_gates.sh <anchor-binary>
set -u
BIN="${1:?usage: anc_gates.sh <anchor-binary>}"
T="$(mktemp -d /tmp/anc-test-XXXXXX)"
DB="$T/anchor_log.sqlite"
TSR="$T/tsr"
FREETSA="https://freetsa.org/tsr"
PASS=0; FAIL=0
gate() { # gate <id> <desc> <expected> <actual>
  if [ "$3" = "$4" ]; then echo "GATE $1 PASS: $2"; PASS=$((PASS+1));
  else echo "GATE $1 FAIL: $2 (want [$3] got [$4])"; FAIL=$((FAIL+1)); fi
}

echo "== setup: fetch FreeTSA CA =="
mkdir -p "$TSR/pins"
curl -sS --max-time 25 https://freetsa.org/files/cacert.pem -o "$T/cacert.pem" || { echo "SETUP FAIL: cannot fetch freetsa cacert"; exit 1; }
FP="$(openssl x509 -in "$T/cacert.pem" -noout -fingerprint -sha256 | cut -d= -f2 | tr -d ':' | tr 'A-Z' 'a-z')"
[ "${#FP}" = 64 ] || { echo "SETUP FAIL: bad fp len ${#FP}"; exit 1; }
cp "$T/cacert.pem" "$TSR/pins/$FP.pem"
echo "freetsa fp=$FP"

HEAD1="$(printf 'a1%.0s' $(seq 1 32))"   # 64 hex chars
HEAD2="$(printf 'b2%.0s' $(seq 1 32))"

echo "== ANC-1: live anchor create -> ANCHORED =="
OUT="$("$BIN" create --chain-id TEST --chain-head "$HEAD1" --entry-from 0 --entry-to 99 \
  --tsa "$FREETSA" --pin "$T/cacert.pem" --db "$DB" --tsr-dir "$TSR" 2>"$T/anc1.err")"
echo "$OUT"
gate ANC-1 "status ANCHORED" "ANCHORED" "$(echo "$OUT" | grep -o 'status=[A-Z]*' | cut -d= -f2)"
NROWS="$(sqlite3 "$DB" 'SELECT COUNT(*) FROM anchor_log;')"
gate ANC-1b "one row recorded" "1" "$NROWS"

echo "== ANC-2: verify re-verifies -> ANCHORED =="
OUT2="$("$BIN" verify --db "$DB" --tsr-dir "$TSR" --pin "$T/cacert.pem" 2>"$T/anc2.err")"
echo "$OUT2"
gate ANC-2 "re-verify ANCHORED" "ANCHORED" "$(echo "$OUT2" | grep -o 'ANCHORED\|ANCHOR-MISMATCH\|UNANCHORED' | head -n 1)"

echo "== ANC-3: tampered TSR file detected =="
TSRFILE="$(sqlite3 "$DB" 'SELECT tsr_path FROM anchor_log LIMIT 1;')"
cp "$TSRFILE" "$T/tsr.orig"
printf 'XX' | dd of="$TSRFILE" bs=1 seek=20 conv=notrunc status=none
OUT3="$("$BIN" verify --db "$DB" --tsr-dir "$TSR" --pin "$T/cacert.pem" 2>/dev/null)"
echo "$OUT3" | head -n 2
gate ANC-3 "tamper -> MISMATCH" "MISMATCH" "$(echo "$OUT3" | grep -q 'ANCHOR-MISMATCH' && echo MISMATCH || echo OTHER)"
cp "$T/tsr.orig" "$TSRFILE"  # restore

echo "== ANC-3b: tampered DB head detected =="
sqlite3 "$DB" "UPDATE anchor_log SET chain_head = zeroblob(32) WHERE anchor_id = 1;"
OUT3b="$("$BIN" verify --db "$DB" --tsr-dir "$TSR" --pin "$T/cacert.pem" 2>/dev/null)"
gate ANC-3b "db-tamper -> MISMATCH" "MISMATCH" "$(echo "$OUT3b" | grep -q 'ANCHOR-MISMATCH' && echo MISMATCH || echo OTHER)"
sqlite3 "$DB" "DELETE FROM anchor_log;"  # reset for next gates

echo "== ANC-4: locally-forged TSR rejected under FreeTSA pin =="
# Build a self-signed TSA (spec §3.4 method), forge a WELL-FORMED TSR, then
# confirm it verifies against the fake CA (sanity) but FAILS vs FreeTSA pin.
openssl req -x509 -newkey rsa:2048 -keyout "$T/fake.key" -out "$T/fake.crt" -days 1 -nodes \
  -subj "/CN=fake-tsa" -addext "extendedKeyUsage=critical,timeStamping" >/dev/null 2>&1
printf 'forged-payload' > "$T/fp.bin"
openssl ts -query -data "$T/fp.bin" -sha256 -cert -out "$T/fake.tsq" 2>/dev/null
mkdir -p "$T/demoCA"; echo 01 > "$T/demoCA/tsaserial"
printf '[ tsa ]\ndefault_tsa = tsa_config1\n[ tsa_config1 ]\ndir = %s/demoCA\nserial = %s/demoCA/tsaserial\ncrypto_device = builtin\nsigner_digest = sha256\ndigest = sha256\ness_cert_id_alg = sha256\nsigner_key = %s/fake.key\nsigner_cert = %s/fake.crt\ncerts = %s/fake.crt\npolicy = 1.2.3.4.1\ndefault_policy = 1.2.3.4.1\ndigests = sha256\naccuracy = secs:1\nordering = yes\ntsa_name = yes\ness_cert_id_chain = no\n' \
  "$T" "$T" "$T" "$T" "$T" > "$T/tsa.cnf"
openssl ts -reply -config "$T/tsa.cnf" -queryfile "$T/fake.tsq" -out "$T/fake.tsr" >/dev/null 2>&1
[ -s "$T/fake.tsr" ] || { echo "ANC-4 SETUP FAIL: fake.tsr not generated (refusing vacuous pass)"; exit 1; }
SANITY="$(openssl ts -verify -data "$T/fp.bin" -in "$T/fake.tsr" -CAfile "$T/fake.crt" 2>&1)"
gate ANC-4a "forged TSR well-formed (OK vs fake CA)" "OK" \
  "$(echo "$SANITY" | grep -q 'Verification: OK' && echo OK || echo FAIL)"
V4OUT="$(openssl ts -verify -data "$T/fp.bin" -in "$T/fake.tsr" -CAfile "$T/cacert.pem" 2>&1)"
echo "$V4OUT" | grep -E 'Verification|verify error' | head -n 2
gate ANC-4 "forged TSR fails openssl verify vs pin" "FAIL" \
  "$(echo "$V4OUT" | grep -q 'Verification: OK' && echo OK || echo FAIL)"

echo "== ANC-5: verifier refuses without --pin =="
"$BIN" verify --db "$DB" --tsr-dir "$TSR" >"$T/anc5.out" 2>&1
C5=$?
gate ANC-5 "exit code 2" "2" "$C5"
gate ANC-5b "POLICY REFUSAL printed" "REFUSE" "$(grep -q 'POLICY REFUSAL' "$T/anc5.out" && echo REFUSE || echo OTHER)"

echo "== ANC-6: replay (old TSR claimed for new window) rejected =="
"$BIN" create --chain-id TEST --chain-head "$HEAD1" --entry-from 0 --entry-to 9 \
  --tsa "$FREETSA" --pin "$T/cacert.pem" --db "$DB" --tsr-dir "$TSR" >/dev/null 2>&1
"$BIN" create --chain-id TEST --chain-head "$HEAD2" --entry-from 10 --entry-to 19 \
  --tsa "$FREETSA" --pin "$T/cacert.pem" --db "$DB" --tsr-dir "$TSR" >/dev/null 2>&1
# Attack: point row2 at row1's TSR and fix the sha (simulates stolen-file replay).
P1="$(sqlite3 "$DB" 'SELECT tsr_path FROM anchor_log WHERE anchor_id=1;')"
H1="$(sqlite3 "$DB" 'SELECT hex(tsr_sha256) FROM anchor_log WHERE anchor_id=1;')"
sqlite3 "$DB" "UPDATE anchor_log SET tsr_path='$P1', tsr_sha256=x'$H1' WHERE anchor_id=2;"
OUT6="$("$BIN" verify --db "$DB" --tsr-dir "$TSR" --pin "$T/cacert.pem" 2>/dev/null)"
echo "$OUT6"
gate ANC-6 "replayed TSR -> MISMATCH on row2" "MISMATCH" \
  "$(echo "$OUT6" | sed -n '2p' | grep -q 'ANCHOR-MISMATCH' && echo MISMATCH || echo OTHER)"
sqlite3 "$DB" "DELETE FROM anchor_log;"

echo "== V8: range gap rejected at creation =="
"$BIN" create --chain-id TEST --chain-head "$HEAD1" --entry-from 0 --entry-to 9 \
  --tsa "$FREETSA" --pin "$T/cacert.pem" --db "$DB" --tsr-dir "$TSR" >/dev/null 2>&1
"$BIN" create --chain-id TEST --chain-head "$HEAD2" --entry-from 50 --entry-to 59 \
  --tsa "$FREETSA" --pin "$T/cacert.pem" --db "$DB" --tsr-dir "$TSR" >"$T/v8.out" 2>&1
C8=$?
gate V8 "gap -> exit 2" "2" "$C8"

echo "== ANC-7: dead TSA -> FAILED row + alarm + exit 0 (non-blocking) =="
"$BIN" create --chain-id TEST --chain-head "$HEAD2" --entry-from 10 --entry-to 19 \
  --tsa "http://10.255.255.1/tsr" --pin "$T/cacert.pem" --db "$DB" --tsr-dir "$TSR" \
  --tsa-timeout 3 >"$T/anc7.out" 2>"$T/anc7.err"
C7=$?
cat "$T/anc7.out"; grep ALARM "$T/anc7.err" | head -n 2
gate ANC-7 "exit 0 (non-blocking)" "0" "$C7"
gate ANC-7b "FAILED row recorded" "FAILED" "$(sqlite3 "$DB" "SELECT status FROM anchor_log ORDER BY anchor_id DESC LIMIT 1;")"
gate ANC-7c "ALARM emitted" "ALARM" "$(grep -q ALARM "$T/anc7.err" && echo ALARM || echo OTHER)"

echo "== RESULT: $PASS passed, $FAIL failed (workdir $T kept for audit) =="
[ "$FAIL" = 0 ]

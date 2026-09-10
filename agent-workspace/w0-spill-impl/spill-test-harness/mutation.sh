#!/bin/bash
# W0-SPILL-TEST mutation runner: plants bugs in a scratch copy of the
# implementation and proves the gates go RED exactly where expected.
# Usage: mutation.sh (runs MU1..MU5, prints verdict per mutant)
set -u
MUT=/home/agent1/spill-mut
HTEST=/home/agent1/spill-test-mut
export CARGO_TARGET_DIR=/mnt/extra-storage/cargo-target-agent1-mut
export CARGO_HOME=/home/agent1/.cargo

plant() { # plant triples of: <old> <new> <want-count>
  rm -rf "$MUT" "$HTEST"
  cp -r /home/agent1/spill-work "$MUT"
  cp -r /home/agent1/spill-test "$HTEST"
  sed -i 's|/home/agent1/spill-work|/home/agent1/spill-mut|' "$HTEST/Cargo.toml"
  python3 - "$MUT/crates/data-plane/src/spill.rs" "$@" <<'EOF'
import sys
p = sys.argv[1]
src = open(p).read()
specs = sys.argv[2:]
assert len(specs) % 3 == 0, "triples only"
for i in range(0, len(specs), 3):
    old, new, want = specs[i], specs[i+1], int(specs[i+2])
    n = src.count(old)
    assert n == want, "anchor found %dx want %sx: %r" % (n, want, old[:60])
    src = src.replace(old, new)
open(p, "w").write(src)
print("planted OK")
EOF
}

run_mut() { # run_mut <id> <expected-failing csv> <triples...>
  local id="$1" expect="$2"; shift 2
  echo "===== $id (expect RED: $expect) ====="
  plant "$@"
  local out failed
  out=$(cd "$HTEST" && cargo test --test gates 2>&1 | grep -E 'test t[0-9][0-9]_.* \.\.\. (ok|FAILED)')
  failed=$(echo "$out" | grep FAILED | cut -d_ -f1 | cut -d' ' -f2 | sort | tr '\n' ',' | sed 's/,$//')
  echo "failed: {${failed:-none}}"
  local e_sorted
  e_sorted=$(echo "$expect" | tr ',' '\n' | sort | tr '\n' ',' | sed 's/,$//')
  if [ "$failed" = "$e_sorted" ]; then echo "MUTANT $id: KILLED (exact)"; return 0;
  else echo "MUTANT $id: VERDICT-MISMATCH want={$e_sorted} got={${failed:-none}}"; return 1; fi
}

NL=$'\n'
fails=0
run_mut MU1-skip-magic "t04" \
  "        check_magic(&mut entry.file, handle.path.as_str())?;${NL}" "" "2" \
  "        check_magic(&mut file, handle.path.as_str())?;${NL}" "" "1" \
  || fails=$((fails+1))
run_mut MU2-skip-compare "t03" \
  "        if actual != expected {" "        if false {" "1" \
  || fails=$((fails+1))
run_mut MU3-no-0600 "t06" \
  "        opts.mode(0o600);${NL}" "" "1" \
  || fails=$((fails+1))
# MU4 blast radius: dup offsets break roundtrip (t01), pushed() count (t09),
# indexed read (t10) and API-OOB boundary (t12) — all four are correct kills.
run_mut MU4-offset-dup "t01,t09,t10,t12" \
  "            self.offsets.push(self.pos);" "            self.offsets.push(self.pos); self.offsets.push(self.pos);" "1" \
  || fails=$((fails+1))
run_mut MU5-verify-true "t03,t04,t07" \
  "    async fn verify_integrity(&self, handle: &SpilledList) -> Result<bool, KernelError> {${NL}        let expected" \
  "    async fn verify_integrity(&self, handle: &SpilledList) -> Result<bool, KernelError> {${NL}        let _ = handle;${NL}        return Ok(true);${NL}        let expected" "1" \
  || fails=$((fails+1))
echo "===== MUTATION RESULT: $((5-fails))/5 killed ====="
exit "$fails"

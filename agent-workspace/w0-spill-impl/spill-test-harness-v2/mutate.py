#!/usr/bin/env python3
"""Apply one mutant (MU-A..MU-F) to a COPY of agent2's spill.rs. Never touches shared code."""
import sys

MUT = sys.argv[1]
P = sys.argv[2]
src = open(P).read()


def sub(old: str, new: str, count: int):
    global src
    assert src.count(old) == count, f"{MUT}: pattern x{src.count(old)}, want x{count}: {old[:70]!r}"
    src = src.replace(old, new)


if MUT == "MU-A":  # checksum compare bypassed (both read sites)
    sub("if computed_checksum != header.checksum {", "if false {", 2)
elif MUT == "MU-B":  # magic/version validation skipped (both read sites)
    sub("        header.validate()?;\n", "", 2)
elif MUT == "MU-C":  # 0600 enforcement removed
    sub("            fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600)).map_err(io_to_kernel)?;\n", "", 1)
elif MUT == "MU-D":  # disk_footprint always 0
    sub("        fs::metadata(Path::new(handle.path.as_str())).map(|m| m.len()).unwrap_or(0)", "        0", 1)
elif MUT == "MU-E":  # delete is a no-op
    sub("""        let path = Path::new(handle.path.as_str());
        if path.exists() {
            tokio_fs::remove_file(path).await.map_err(io_to_kernel)?;
        }
        Ok(())""", """        let _ = handle;
        Ok(())""", 1)
elif MUT == "MU-F":  # delete errors on missing file (idempotency broken)
    sub("""        let path = Path::new(handle.path.as_str());
        if path.exists() {
            tokio_fs::remove_file(path).await.map_err(io_to_kernel)?;
        }
        Ok(())""", """        let path = Path::new(handle.path.as_str());
        tokio_fs::remove_file(path).await.map_err(io_to_kernel)?;
        Ok(())""", 1)
else:
    raise SystemExit(f"unknown mutant {MUT}")

open(P, "w").write(src)
print(f"{MUT}-APPLIED")

//! Pemetaan manifest -> spec sandbox (MURNI; tanpa runtime). Lapis host (wasmtime)
//! mengonsumsi spec ini. Tidak mengimpor kernel — aman dikerjakan sblm W0-KERNEL-UNIFY.

use crate::manifest::Manifest;

pub const WASM_PAGE_BYTES: u64 = 65_536; // 64 KiB
pub const MAX_LINEAR_MEMORY_BYTES: u64 = 32 * 1024 * 1024; // 32 MiB sandbox (mandat #1074)

/// Spec sandbox yg diturunkan dari manifest ABI + guards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxSpec {
    /// Halaman memori linear maks (<= 2048 = 32 MiB).
    pub memory_pages_max: u32,
    pub memory_max_bytes: u64,
    /// Batas fuel (instruksi) per instance; None = fuel tak diwajibkan manifest.
    pub fuel: Option<u64>,
    /// body_window -> spill + truncated (MV-6).
    pub body_window_bytes: u64,
    /// polite_delay_ms [min,max] utk egress (guards).
    pub polite_delay_ms: (u64, u64),
    /// sandbox 32MiB linear (mandat #1074) — TRUE selalu utk node WCB.
    pub linear_32mib: bool,
}

impl SandboxSpec {
    pub fn from_manifest(m: &Manifest) -> SandboxSpec {
        let pages = m.abi.memory_max_pages;
        let bytes = u64::from(pages) * WASM_PAGE_BYTES;
        SandboxSpec {
            memory_pages_max: pages,
            memory_max_bytes: bytes,
            fuel: None, // fuel dikonfigurasi lapis host pasca-load; default None di spec
            body_window_bytes: m.guards.body_window_bytes,
            polite_delay_ms: m.guards.polite_delay_ms,
            linear_32mib: bytes <= MAX_LINEAR_MEMORY_BYTES,
        }
    }

    /// Guard keras: node WCB dilarang melebihi 32 MiB linear (pooling static_memory).
    pub fn assert_within_32mib(&self) -> bool {
        self.memory_max_bytes <= MAX_LINEAR_MEMORY_BYTES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_fixture_yields_32mib_linear() {
        let m = crate::manifest::tests::example_manifest();
        let s = SandboxSpec::from_manifest(&m);
        assert_eq!(s.memory_pages_max, crate::manifest::MAX_MEMORY_PAGES); // 512
        assert_eq!(s.memory_max_bytes, MAX_LINEAR_MEMORY_BYTES);
        assert!(s.linear_32mib && s.assert_within_32mib());
        assert_eq!(s.body_window_bytes, 262_144);
        assert_eq!(s.polite_delay_ms, (2000, 5000));
    }

    #[test]
    fn page_math_512pages_eq_32mib_and_over_limit_rejected() {
        let ok = 512u32; // 512 * 64KiB = 32 MiB persis
        let at_limit = SandboxSpec {
            memory_pages_max: ok,
            memory_max_bytes: u64::from(ok) * WASM_PAGE_BYTES,
            fuel: None,
            body_window_bytes: 262_144,
            polite_delay_ms: (2000, 5000),
            linear_32mib: true,
        };
        assert_eq!(at_limit.memory_max_bytes, MAX_LINEAR_MEMORY_BYTES);
        assert!(at_limit.assert_within_32mib());

        let over = 513u32;
        let too_big = SandboxSpec {
            memory_pages_max: over,
            memory_max_bytes: u64::from(over) * WASM_PAGE_BYTES,
            fuel: None,
            body_window_bytes: 262_144,
            polite_delay_ms: (2000, 5000),
            linear_32mib: false,
        };
        assert!(!too_big.assert_within_32mib());
        assert!(too_big.memory_max_bytes > MAX_LINEAR_MEMORY_BYTES);
    }
}

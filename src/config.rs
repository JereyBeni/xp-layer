//! Configuration and host -> XP environment mapping.

use sysinfo::System;

/// Returns the amount of RAM (in mebibytes) that should be reported
/// to Windows XP-era applications, based on the host's physical RAM.
///
/// Mapping (as defined for this project):
/// - >= 16 GB host -> 2 GB
/// - >=  8 GB host -> 1 GB
/// - >=  4 GB host -> 768 MB
/// - >=  2 GB host -> 512 MB
/// - otherwise     -> 128 MB
pub fn reported_xp_memory_mb(host_ram_mb: u64) -> u64 {
    match host_ram_mb {
        x if x >= 16 * 1024 => 2 * 1024, // 16 GB -> 2 GB
        x if x >= 8 * 1024 => 1024,      //  8 GB -> 1 GB
        x if x >= 4 * 1024 => 768,       //  4 GB -> 768 MB
        x if x >= 2 * 1024 => 512,       //  2 GB -> 512 MB
        _ => 128,                        //  1 GB or less -> 128 MB
    }
}

/// Detects the host's total physical memory in mebibytes.
pub fn detect_host_memory_mb() -> u64 {
    let mut sys = System::new();
    sys.refresh_memory();
    // sysinfo returns bytes
    sys.total_memory() / (1024 * 1024)
}

/// Holds the runtime configuration used by the translation layer.
#[derive(Debug, Clone)]
pub struct LayerConfig {
    pub host_memory_mb: u64,
    pub reported_memory_mb: u64,
}

impl LayerConfig {
    pub fn detect() -> Self {
        let host_memory_mb = detect_host_memory_mb();
        let reported_memory_mb = reported_xp_memory_mb(host_memory_mb);
        Self {
            host_memory_mb,
            reported_memory_mb,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_mapping() {
        assert_eq!(reported_xp_memory_mb(16 * 1024), 2 * 1024);
        assert_eq!(reported_xp_memory_mb(8 * 1024), 1024);
        assert_eq!(reported_xp_memory_mb(4 * 1024), 768);
        assert_eq!(reported_xp_memory_mb(2 * 1024), 512);
        assert_eq!(reported_xp_memory_mb(1024), 128);
        assert_eq!(reported_xp_memory_mb(512), 128);
    }
}

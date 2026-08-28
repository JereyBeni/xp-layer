//! Configuration and host -> XP environment mapping.
//!
//! Currently holds the memory-reporting policy. Host memory detection
//! will be added later (e.g. via the `sysinfo` crate).

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

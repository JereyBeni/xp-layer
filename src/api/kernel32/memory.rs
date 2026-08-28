//! Memory-status APIs that report the constrained XP memory size.

use crate::config::LayerConfig;

/// MEMORYSTATUS structure (Windows XP).
/// https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-memorystatus
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryStatus {
    pub length: u32,
    pub memory_load: u32,
    pub total_phys: usize,
    pub avail_phys: usize,
    pub total_page_file: usize,
    pub avail_page_file: usize,
    pub total_virtual: usize,
    pub avail_virtual: usize,
}

/// MEMORYSTATUSEX structure (Windows XP and later).
/// https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/ns-sysinfoapi-memorystatusex
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryStatusEx {
    pub length: u32,
    pub memory_load: u32,
    pub total_phys: u64,
    pub avail_phys: u64,
    pub total_page_file: u64,
    pub avail_page_file: u64,
    pub total_virtual: u64,
    pub avail_virtual: u64,
    pub avail_extended_virtual: u64,
}

/// GlobalMemoryStatus equivalent.
/// Fills the structure with the constrained XP-visible memory values.
pub fn global_memory_status(config: &LayerConfig) -> MemoryStatus {
    let total = (config.reported_memory_mb as usize).saturating_mul(1024 * 1024);
    // Present a modest amount as "in use" so memory_load is non-zero but realistic.
    let used = total / 8;
    let avail = total.saturating_sub(used);

    MemoryStatus {
        length: std::mem::size_of::<MemoryStatus>() as u32,
        memory_load: ((used as u64 * 100) / total.max(1) as u64) as u32,
        total_phys: total,
        avail_phys: avail,
        total_page_file: total * 2,
        avail_page_file: total * 2 - used,
        total_virtual: 0x7FFE_FFFF, // typical 32-bit user-mode limit approx
        avail_virtual: 0x7FFE_FFFF - used,
    }
}

/// GlobalMemoryStatusEx equivalent.
pub fn global_memory_status_ex(config: &LayerConfig) -> MemoryStatusEx {
    let total = (config.reported_memory_mb as u64).saturating_mul(1024 * 1024);
    let used = total / 8;
    let avail = total.saturating_sub(used);

    MemoryStatusEx {
        length: std::mem::size_of::<MemoryStatusEx>() as u32,
        memory_load: ((used * 100) / total.max(1)) as u32,
        total_phys: total,
        avail_phys: avail,
        total_page_file: total * 2,
        avail_page_file: total * 2 - used,
        total_virtual: 0x7FFE_FFFF,
        avail_virtual: 0x7FFE_FFFF - used,
        avail_extended_virtual: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LayerConfig;

    #[test]
    fn status_respects_reported_limit() {
        let config = LayerConfig {
            host_memory_mb: 16 * 1024,
            reported_memory_mb: 2 * 1024,
        };
        let status = global_memory_status_ex(&config);
        assert_eq!(status.total_phys, 2 * 1024 * 1024 * 1024);
        assert!(status.avail_phys <= status.total_phys);
        assert!(status.memory_load <= 100);
    }
}

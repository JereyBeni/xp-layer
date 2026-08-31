//! Host CPU detection via CPUID.

use raw_cpuid::CpuId;

/// Result of the host CPU policy check.
#[derive(Debug, Clone)]
pub struct HostCpuInfo {
    pub vendor: String,
    pub brand: String,
    pub family: u32,
    pub model: u32,
    pub stepping: u32,
    pub status: HostCpuStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostCpuStatus {
    /// Intel, accepted for this build.
    SupportedIntel,
    /// AMD — explicitly not supported yet.
    UnsupportedAmd,
    /// Other vendor (VIA, Hygon, virtual weirdness, etc.).
    UnsupportedVendor,
    /// Intel but too old for the current policy (~pre-2006).
    UnsupportedOldIntel,
    /// Could not read CPUID usefully.
    Unknown,
}

impl HostCpuStatus {
    pub fn is_ok(self) -> bool {
        matches!(self, HostCpuStatus::SupportedIntel)
    }

    pub fn message(self) -> &'static str {
        match self {
            HostCpuStatus::SupportedIntel => "Intel CPU accepted (2006–present policy).",
            HostCpuStatus::UnsupportedAmd => {
                "AMD CPUs are not supported yet. Intel only for now."
            }
            HostCpuStatus::UnsupportedVendor => {
                "Only GenuineIntel hosts are supported in this build."
            }
            HostCpuStatus::UnsupportedOldIntel => {
                "This Intel CPU is older than the 2006+ policy allows."
            }
            HostCpuStatus::Unknown => "Could not identify the host CPU."
        }
    }
}

/// Probe the host with CPUID and apply the project policy.
pub fn check_host_cpu() -> HostCpuInfo {
    let cpuid = CpuId::new();

    let vendor = cpuid
        .get_vendor_info()
        .map(|v| v.as_str().to_string())
        .unwrap_or_else(|| "unknown".into());

    let brand = cpuid
        .get_processor_brand_string()
        .map(|b| b.as_str().trim().to_string())
        .unwrap_or_else(|| "(unknown brand)".into());

    let (family, model, stepping) = cpuid
        .get_feature_info()
        .map(|f| {
            (
                f.family_id() as u32,
                f.model_id() as u32,
                f.stepping_id() as u32,
            )
        })
        .unwrap_or((0, 0, 0));

    let status = classify(&vendor, family, model);

    HostCpuInfo {
        vendor,
        brand,
        family,
        model,
        stepping,
        status,
    }
}

fn classify(vendor: &str, family: u32, model: u32) -> HostCpuStatus {
    let v = vendor.to_ascii_lowercase();
    if v.contains("amd") || v.contains("authenticamd") {
        return HostCpuStatus::UnsupportedAmd;
    }
    if !v.contains("intel") && !v.contains("genuineintel") {
        // raw-cpuid returns "GenuineIntel" etc.
        if vendor != "GenuineIntel" {
            return HostCpuStatus::UnsupportedVendor;
        }
    }
    if vendor != "GenuineIntel" && !v.contains("intel") {
        return HostCpuStatus::UnsupportedVendor;
    }
    // GenuineIntel path
    if vendor != "GenuineIntel" {
        // brand string path already filtered; treat odd strings carefully
        if !v.contains("intel") {
            return HostCpuStatus::UnsupportedVendor;
        }
    }

    // Intel family 6 = P6 / Core lineage used from Pentium Pro through modern Core.
    // Reject clearly ancient families (4, 5) and require family 6+ with a
    // model floor that roughly matches Core 2 era (2006).
    //
    // Core 2 (Conroe) is family 6, model 15 (0xF) and later models in family 6.
    // We accept family 6 with model >= 15, or any newer display family (>= 15
    // extended encodings already folded into family_id by raw-cpuid).
    if family < 6 {
        return HostCpuStatus::UnsupportedOldIntel;
    }
    if family == 6 && model < 15 {
        // e.g. Pentium III / early Pentium M era
        return HostCpuStatus::UnsupportedOldIntel;
    }

    HostCpuStatus::SupportedIntel
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amd_rejected() {
        assert_eq!(
            classify("AuthenticAMD", 25, 1),
            HostCpuStatus::UnsupportedAmd
        );
    }

    #[test]
    fn old_intel_rejected() {
        assert_eq!(classify("GenuineIntel", 6, 8), HostCpuStatus::UnsupportedOldIntel);
        assert_eq!(classify("GenuineIntel", 5, 4), HostCpuStatus::UnsupportedOldIntel);
    }

    #[test]
    fn core2_era_accepted() {
        assert_eq!(classify("GenuineIntel", 6, 15), HostCpuStatus::SupportedIntel);
        assert_eq!(classify("GenuineIntel", 6, 60), HostCpuStatus::SupportedIntel);
    }
}

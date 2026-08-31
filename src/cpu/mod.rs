//! Host CPU checks and guest x86 execution skeleton.
//!
//! Host policy (current):
//! - Only **GenuineIntel** is accepted.
//! - AMD and other vendors are rejected ("not included yet").
//! - Intel CPUs from roughly Core 2 era (2006) onward are accepted
//!   via family/model heuristics; very old Pentium-era chips are rejected.

mod check;
mod emu;

pub use check::{check_host_cpu, HostCpuInfo, HostCpuStatus};
pub use emu::{CpuState, Emulator, EmulatorReport};

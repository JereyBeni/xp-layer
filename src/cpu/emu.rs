//! Minimal 32-bit x86 CPU state and a tiny interpreter skeleton.
//!
//! This is not a full emulator yet. It holds registers, points at the
//! loaded PE image, and can single-step a very small set of opcodes so
//! the pipeline (load → check CPU → enter emulator) is in place.

use crate::pe::LoadedImage;

/// General-purpose and special registers for a 32-bit guest.
#[derive(Debug, Clone)]
pub struct CpuState {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
    pub esi: u32,
    pub edi: u32,
    pub ebp: u32,
    pub esp: u32,
    pub eip: u32,
    pub eflags: u32,
    pub cs: u16,
    pub ds: u16,
    pub es: u16,
    pub ss: u16,
    pub fs: u16,
    pub gs: u16,
}

impl Default for CpuState {
    fn default() -> Self {
        Self {
            eax: 0,
            ebx: 0,
            ecx: 0,
            edx: 0,
            esi: 0,
            edi: 0,
            ebp: 0,
            esp: 0,
            eip: 0,
            eflags: 0x202, // reserved bit + IF often set at process start
            cs: 0x23,
            ds: 0x2B,
            es: 0x2B,
            ss: 0x2B,
            fs: 0,
            gs: 0,
        }
    }
}

/// Result of a short emulation burst.
#[derive(Debug, Clone)]
pub struct EmulatorReport {
    pub steps: u32,
    pub stopped_reason: String,
    pub eip: u32,
    pub eax: u32,
}

/// Guest CPU bound to a loaded PE image.
pub struct Emulator {
    pub state: CpuState,
    pub image_base: u32,
    /// Guest linear memory (PE image at preferred base for now).
    memory: Vec<u8>,
}

impl Emulator {
    /// Create an emulator positioned at the PE entry point.
    pub fn from_image(image: &LoadedImage) -> Self {
        let mut state = CpuState::default();
        // Flat model: EIP = image_base + entry RVA
        state.eip = image.image_base.wrapping_add(image.entry_point_rva);
        // Simple stack near top of a 1 MiB stack region after the image (placeholder)
        state.esp = image.image_base.wrapping_add(image.memory.len() as u32).wrapping_add(0x10000);
        state.ebp = state.esp;

        Self {
            state,
            image_base: image.image_base,
            memory: image.memory.clone(),
        }
    }

    fn read_u8(&self, linear: u32) -> Option<u8> {
        let off = linear.wrapping_sub(self.image_base) as usize;
        self.memory.get(off).copied()
    }

    fn read_u32(&self, linear: u32) -> Option<u32> {
        let b0 = self.read_u8(linear)? as u32;
        let b1 = self.read_u8(linear.wrapping_add(1))? as u32;
        let b2 = self.read_u8(linear.wrapping_add(2))? as u32;
        let b3 = self.read_u8(linear.wrapping_add(3))? as u32;
        Some(b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    }

    /// Run up to `max_steps` instructions. Stops on unknown opcode or limit.
    pub fn run(&mut self, max_steps: u32) -> EmulatorReport {
        let mut steps = 0u32;
        let mut reason = String::from("step limit reached");

        while steps < max_steps {
            let eip = self.state.eip;
            let op = match self.read_u8(eip) {
                Some(b) => b,
                None => {
                    reason = format!("fetch failed at EIP=0x{:08X} (outside image)", eip);
                    break;
                }
            };

            match op {
                // NOP
                0x90 => {
                    self.state.eip = self.state.eip.wrapping_add(1);
                }
                // MOV EAX, imm32
                0xB8 => {
                    if let Some(imm) = self.read_u32(eip.wrapping_add(1)) {
                        self.state.eax = imm;
                        self.state.eip = self.state.eip.wrapping_add(5);
                    } else {
                        reason = format!("truncated MOV EAX,imm32 at 0x{:08X}", eip);
                        break;
                    }
                }
                // RET (near)
                0xC3 => {
                    reason = format!("RET at EIP=0x{:08X}", eip);
                    break;
                }
                // INT3
                0xCC => {
                    reason = format!("INT3 at EIP=0x{:08X}", eip);
                    break;
                }
                _ => {
                    reason = format!(
                        "unimplemented opcode 0x{:02X} at EIP=0x{:08X}",
                        op, eip
                    );
                    break;
                }
            }

            steps += 1;
        }

        EmulatorReport {
            steps,
            stopped_reason: reason,
            eip: self.state.eip,
            eax: self.state.eax,
        }
    }
}

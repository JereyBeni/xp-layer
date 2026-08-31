//! Loads a 32-bit PE image into a flat memory buffer.

use super::header::PeInfo;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// A loaded PE image ready for further processing by the translation layer.
#[derive(Debug)]
pub struct LoadedImage {
    pub info: PeInfo,
    /// Flat memory image (SizeOfImage bytes), sections copied to their RVAs.
    pub memory: Vec<u8>,
    pub entry_point_rva: u32,
    pub image_base: u32,
}

impl LoadedImage {
    /// Load a PE file from disk and map its sections.
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut file = File::open(path)?;
        let info = PeInfo::parse(&mut file)?;

        let size = info.optional.size_of_image as usize;
        if size == 0 || size > 512 * 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unreasonable SizeOfImage: {}", size),
            ));
        }

        let mut memory = vec![0u8; size];

        // Copy headers
        let header_size = info.optional.size_of_headers as usize;
        file.seek(SeekFrom::Start(0))?;
        let mut header_buf = vec![0u8; header_size.min(size)];
        file.read_exact(&mut header_buf)?;
        memory[..header_buf.len()].copy_from_slice(&header_buf);

        // Map each section
        for section in &info.sections {
            if section.size_of_raw_data == 0 || section.pointer_to_raw_data == 0 {
                continue;
            }
            let raw_size = section.size_of_raw_data as usize;
            let dest_rva = section.virtual_address as usize;
            if dest_rva >= size {
                continue;
            }
            let copy_len = raw_size.min(size - dest_rva);

            file.seek(SeekFrom::Start(section.pointer_to_raw_data as u64))?;
            let mut section_data = vec![0u8; copy_len];
            match file.read_exact(&mut section_data) {
                Ok(()) => memory[dest_rva..dest_rva + copy_len].copy_from_slice(&section_data),
                Err(_) => {
                    // partial read is acceptable for the last section sometimes
                    let n = file.read(&mut section_data)?;
                    memory[dest_rva..dest_rva + n].copy_from_slice(&section_data[..n]);
                }
            }
        }

        Ok(Self {
            entry_point_rva: info.optional.address_of_entry_point,
            image_base: info.optional.image_base,
            info,
            memory,
        })
    }

    /// Human-readable summary for the UI / debug status.
    pub fn summary(&self) -> String {
        let mut s = format!(
            "PE loaded successfully\n\
  DOS magic   : 0x{:04X}\n\
  Machine     : 0x{:04X} (i386)\n\
  Sections    : {}\n\
  ImageBase   : 0x{:08X}\n\
  EntryPoint  : RVA 0x{:08X}\n\
  SizeOfImage : {} bytes\n\
  Subsystem   : {}\n",
            self.info.dos.e_magic,
            self.info.file.machine,
            self.info.file.number_of_sections,
            self.image_base,
            self.entry_point_rva,
            self.memory.len(),
            self.info.optional.subsystem
        );
        for sec in &self.info.sections {
            s.push_str(&format!(
                "    [{:8}] VA=0x{:08X}  RawSize={}\n",
                sec.name, sec.virtual_address, sec.size_of_raw_data
            ));
        }
        s
    }
}

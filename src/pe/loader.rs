//! Loads a 32-bit PE image into a flat memory buffer and parses imports.

use super::header::PeInfo;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// A single imported function (by name or by ordinal).
#[derive(Debug, Clone)]
pub enum ImportedSymbol {
    Name(String),
    Ordinal(u16),
}

/// Imports from one DLL.
#[derive(Debug, Clone)]
pub struct ImportDll {
    pub name: String,
    pub symbols: Vec<ImportedSymbol>,
}

/// A loaded PE image ready for further processing by the translation layer.
#[derive(Debug)]
pub struct LoadedImage {
    pub info: PeInfo,
    /// Flat memory image (SizeOfImage bytes), sections copied to their RVAs.
    pub memory: Vec<u8>,
    pub entry_point_rva: u32,
    pub image_base: u32,
    pub imports: Vec<ImportDll>,
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
                    let n = file.read(&mut section_data)?;
                    memory[dest_rva..dest_rva + n].copy_from_slice(&section_data[..n]);
                }
            }
        }

        let imports = parse_imports(&memory, &info);

        Ok(Self {
            entry_point_rva: info.optional.address_of_entry_point,
            image_base: info.optional.image_base,
            info,
            memory,
            imports,
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

        if self.imports.is_empty() {
            s.push_str("\nImports: (none found)\n");
        } else {
            s.push_str(&format!("\nImports ({} DLL(s)):\n", self.imports.len()));
            for dll in &self.imports {
                s.push_str(&format!("  {}\n", dll.name));
                for sym in &dll.symbols {
                    match sym {
                        ImportedSymbol::Name(n) => s.push_str(&format!("    - {}\n", n)),
                        ImportedSymbol::Ordinal(o) => {
                            s.push_str(&format!("    - ordinal {}\n", o))
                        }
                    }
                }
            }
        }

        s
    }
}

fn read_u32(mem: &[u8], rva: usize) -> Option<u32> {
    let end = rva.checked_add(4)?;
    let bytes = mem.get(rva..end)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_cstring(mem: &[u8], rva: usize) -> Option<String> {
    if rva >= mem.len() {
        return None;
    }
    let end = mem[rva..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| rva + p)
        .unwrap_or(mem.len());
    let slice = mem.get(rva..end)?;
    Some(String::from_utf8_lossy(slice).into_owned())
}

/// Parse the import directory from the already-mapped image.
fn parse_imports(memory: &[u8], info: &PeInfo) -> Vec<ImportDll> {
    let import_rva = info.optional.import_directory.virtual_address as usize;
    if import_rva == 0 || import_rva >= memory.len() {
        return Vec::new();
    }

    let mut imports = Vec::new();
    let mut desc_offset = import_rva;

    // IMAGE_IMPORT_DESCRIPTOR is 20 bytes; null descriptor terminates the list
    loop {
        if desc_offset + 20 > memory.len() {
            break;
        }

        let original_first_thunk = match read_u32(memory, desc_offset) {
            Some(v) => v,
            None => break,
        };
        // Skip TimeDateStamp (4), ForwarderChain (4)
        let name_rva = match read_u32(memory, desc_offset + 12) {
            Some(v) => v,
            None => break,
        };
        let first_thunk = match read_u32(memory, desc_offset + 16) {
            Some(v) => v,
            None => break,
        };

        // Null descriptor ends the array
        if original_first_thunk == 0 && name_rva == 0 && first_thunk == 0 {
            break;
        }

        let dll_name = match read_cstring(memory, name_rva as usize) {
            Some(n) if !n.is_empty() => n,
            _ => {
                desc_offset += 20;
                continue;
            }
        };

        // Prefer the ILT (OriginalFirstThunk); fall back to IAT (FirstThunk)
        let mut thunk_rva = if original_first_thunk != 0 {
            original_first_thunk as usize
        } else {
            first_thunk as usize
        };

        let mut symbols = Vec::new();
        loop {
            let thunk = match read_u32(memory, thunk_rva) {
                Some(v) => v,
                None => break,
            };
            if thunk == 0 {
                break;
            }

            // High bit set => import by ordinal
            if thunk & 0x8000_0000 != 0 {
                let ordinal = (thunk & 0xFFFF) as u16;
                symbols.push(ImportedSymbol::Ordinal(ordinal));
            } else {
                // Hint/Name table: 2-byte hint + null-terminated name
                let name_rva = (thunk as usize).saturating_add(2);
                if let Some(name) = read_cstring(memory, name_rva) {
                    if !name.is_empty() {
                        symbols.push(ImportedSymbol::Name(name));
                    }
                }
            }

            thunk_rva += 4; // 32-bit thunks
            if symbols.len() > 4096 {
                // safety cap
                break;
            }
        }

        imports.push(ImportDll {
            name: dll_name,
            symbols,
        });

        desc_offset += 20;
        if imports.len() > 256 {
            // safety cap
            break;
        }
    }

    imports
}

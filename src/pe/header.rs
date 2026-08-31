//! DOS and PE header structures (32-bit).

use std::io::{self, Read, Seek, SeekFrom};

/// IMAGE_DOS_HEADER
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DosHeader {
    pub e_magic: u16, // "MZ"
    pub e_cblp: u16,
    pub e_cp: u16,
    pub e_crlc: u16,
    pub e_cparhdr: u16,
    pub e_minalloc: u16,
    pub e_maxalloc: u16,
    pub e_ss: u16,
    pub e_sp: u16,
    pub e_csum: u16,
    pub e_ip: u16,
    pub e_cs: u16,
    pub e_lfarlc: u16,
    pub e_ovno: u16,
    pub e_res: [u16; 4],
    pub e_oemid: u16,
    pub e_oeminfo: u16,
    pub e_res2: [u16; 10],
    pub e_lfanew: u32, // offset to PE header
}

impl DosHeader {
    pub const MAGIC: u16 = 0x5A4D; // "MZ"

    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 64];
        r.read_exact(&mut buf)?;
        Ok(Self {
            e_magic: u16::from_le_bytes(buf[0..2].try_into().unwrap()),
            e_cblp: u16::from_le_bytes(buf[2..4].try_into().unwrap()),
            e_cp: u16::from_le_bytes(buf[4..6].try_into().unwrap()),
            e_crlc: u16::from_le_bytes(buf[6..8].try_into().unwrap()),
            e_cparhdr: u16::from_le_bytes(buf[8..10].try_into().unwrap()),
            e_minalloc: u16::from_le_bytes(buf[10..12].try_into().unwrap()),
            e_maxalloc: u16::from_le_bytes(buf[12..14].try_into().unwrap()),
            e_ss: u16::from_le_bytes(buf[14..16].try_into().unwrap()),
            e_sp: u16::from_le_bytes(buf[16..18].try_into().unwrap()),
            e_csum: u16::from_le_bytes(buf[18..20].try_into().unwrap()),
            e_ip: u16::from_le_bytes(buf[20..22].try_into().unwrap()),
            e_cs: u16::from_le_bytes(buf[22..24].try_into().unwrap()),
            e_lfarlc: u16::from_le_bytes(buf[24..26].try_into().unwrap()),
            e_ovno: u16::from_le_bytes(buf[26..28].try_into().unwrap()),
            e_res: [
                u16::from_le_bytes(buf[28..30].try_into().unwrap()),
                u16::from_le_bytes(buf[30..32].try_into().unwrap()),
                u16::from_le_bytes(buf[32..34].try_into().unwrap()),
                u16::from_le_bytes(buf[34..36].try_into().unwrap()),
            ],
            e_oemid: u16::from_le_bytes(buf[36..38].try_into().unwrap()),
            e_oeminfo: u16::from_le_bytes(buf[38..40].try_into().unwrap()),
            e_res2: [
                u16::from_le_bytes(buf[40..42].try_into().unwrap()),
                u16::from_le_bytes(buf[42..44].try_into().unwrap()),
                u16::from_le_bytes(buf[44..46].try_into().unwrap()),
                u16::from_le_bytes(buf[46..48].try_into().unwrap()),
                u16::from_le_bytes(buf[48..50].try_into().unwrap()),
                u16::from_le_bytes(buf[50..52].try_into().unwrap()),
                u16::from_le_bytes(buf[52..54].try_into().unwrap()),
                u16::from_le_bytes(buf[54..56].try_into().unwrap()),
                u16::from_le_bytes(buf[56..58].try_into().unwrap()),
                u16::from_le_bytes(buf[58..60].try_into().unwrap()),
            ],
            e_lfanew: u32::from_le_bytes(buf[60..64].try_into().unwrap()),
        })
    }
}

/// IMAGE_FILE_HEADER
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FileHeader {
    pub machine: u16,
    pub number_of_sections: u16,
    pub time_date_stamp: u32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub size_of_optional_header: u16,
    pub characteristics: u16,
}

impl FileHeader {
    pub const MACHINE_I386: u16 = 0x014c;

    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 20];
        r.read_exact(&mut buf)?;
        Ok(Self {
            machine: u16::from_le_bytes(buf[0..2].try_into().unwrap()),
            number_of_sections: u16::from_le_bytes(buf[2..4].try_into().unwrap()),
            time_date_stamp: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            pointer_to_symbol_table: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            number_of_symbols: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            size_of_optional_header: u16::from_le_bytes(buf[16..18].try_into().unwrap()),
            characteristics: u16::from_le_bytes(buf[18..20].try_into().unwrap()),
        })
    }
}

/// IMAGE_DATA_DIRECTORY entry
#[derive(Debug, Clone, Copy, Default)]
pub struct DataDirectory {
    pub virtual_address: u32,
    pub size: u32,
}

/// IMAGE_OPTIONAL_HEADER32 (partial, the fields we need)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OptionalHeader32 {
    pub magic: u16, // 0x10B = PE32
    pub major_linker_version: u8,
    pub minor_linker_version: u8,
    pub size_of_code: u32,
    pub size_of_initialized_data: u32,
    pub size_of_uninitialized_data: u32,
    pub address_of_entry_point: u32,
    pub base_of_code: u32,
    pub base_of_data: u32,
    pub image_base: u32,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub major_os_version: u16,
    pub minor_os_version: u16,
    pub major_image_version: u16,
    pub minor_image_version: u16,
    pub major_subsystem_version: u16,
    pub minor_subsystem_version: u16,
    pub win32_version_value: u32,
    pub size_of_image: u32,
    pub size_of_headers: u32,
    pub checksum: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
    pub size_of_stack_reserve: u32,
    pub size_of_stack_commit: u32,
    pub size_of_heap_reserve: u32,
    pub size_of_heap_commit: u32,
    pub loader_flags: u32,
    pub number_of_rva_and_sizes: u32,
    /// Index 1 = IMAGE_DIRECTORY_ENTRY_IMPORT
    pub import_directory: DataDirectory,
}

impl OptionalHeader32 {
    pub const MAGIC_PE32: u16 = 0x10B;

    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 96]; // up to NumberOfRvaAndSizes
        r.read_exact(&mut buf)?;

        let number_of_rva_and_sizes = u32::from_le_bytes(buf[92..96].try_into().unwrap());

        // Read data directories
        let mut import_directory = DataDirectory::default();
        for i in 0..number_of_rva_and_sizes {
            let mut dir_buf = [0u8; 8];
            r.read_exact(&mut dir_buf)?;
            let va = u32::from_le_bytes(dir_buf[0..4].try_into().unwrap());
            let size = u32::from_le_bytes(dir_buf[4..8].try_into().unwrap());
            if i == 1 {
                // IMAGE_DIRECTORY_ENTRY_IMPORT
                import_directory = DataDirectory {
                    virtual_address: va,
                    size,
                };
            }
        }

        Ok(Self {
            magic: u16::from_le_bytes(buf[0..2].try_into().unwrap()),
            major_linker_version: buf[2],
            minor_linker_version: buf[3],
            size_of_code: u32::from_le_bytes(buf[4..8].try_into().unwrap()),
            size_of_initialized_data: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            size_of_uninitialized_data: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            address_of_entry_point: u32::from_le_bytes(buf[16..20].try_into().unwrap()),
            base_of_code: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
            base_of_data: u32::from_le_bytes(buf[24..28].try_into().unwrap()),
            image_base: u32::from_le_bytes(buf[28..32].try_into().unwrap()),
            section_alignment: u32::from_le_bytes(buf[32..36].try_into().unwrap()),
            file_alignment: u32::from_le_bytes(buf[36..40].try_into().unwrap()),
            major_os_version: u16::from_le_bytes(buf[40..42].try_into().unwrap()),
            minor_os_version: u16::from_le_bytes(buf[42..44].try_into().unwrap()),
            major_image_version: u16::from_le_bytes(buf[44..46].try_into().unwrap()),
            minor_image_version: u16::from_le_bytes(buf[46..48].try_into().unwrap()),
            major_subsystem_version: u16::from_le_bytes(buf[48..50].try_into().unwrap()),
            minor_subsystem_version: u16::from_le_bytes(buf[50..52].try_into().unwrap()),
            win32_version_value: u32::from_le_bytes(buf[52..56].try_into().unwrap()),
            size_of_image: u32::from_le_bytes(buf[56..60].try_into().unwrap()),
            size_of_headers: u32::from_le_bytes(buf[60..64].try_into().unwrap()),
            checksum: u32::from_le_bytes(buf[64..68].try_into().unwrap()),
            subsystem: u16::from_le_bytes(buf[68..70].try_into().unwrap()),
            dll_characteristics: u16::from_le_bytes(buf[70..72].try_into().unwrap()),
            size_of_stack_reserve: u32::from_le_bytes(buf[72..76].try_into().unwrap()),
            size_of_stack_commit: u32::from_le_bytes(buf[76..80].try_into().unwrap()),
            size_of_heap_reserve: u32::from_le_bytes(buf[80..84].try_into().unwrap()),
            size_of_heap_commit: u32::from_le_bytes(buf[84..88].try_into().unwrap()),
            loader_flags: u32::from_le_bytes(buf[88..92].try_into().unwrap()),
            number_of_rva_and_sizes,
            import_directory,
        })
    }
}

/// IMAGE_SECTION_HEADER
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SectionHeader {
    pub name: String,
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_linenumbers: u32,
    pub number_of_relocations: u16,
    pub number_of_linenumbers: u16,
    pub characteristics: u32,
}

impl SectionHeader {
    pub fn read<R: Read>(r: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; 40];
        r.read_exact(&mut buf)?;
        let name_bytes = &buf[0..8];
        let name = String::from_utf8_lossy(name_bytes)
            .trim_end_matches('\0')
            .to_string();
        Ok(Self {
            name,
            virtual_size: u32::from_le_bytes(buf[8..12].try_into().unwrap()),
            virtual_address: u32::from_le_bytes(buf[12..16].try_into().unwrap()),
            size_of_raw_data: u32::from_le_bytes(buf[16..20].try_into().unwrap()),
            pointer_to_raw_data: u32::from_le_bytes(buf[20..24].try_into().unwrap()),
            pointer_to_relocations: u32::from_le_bytes(buf[24..28].try_into().unwrap()),
            pointer_to_linenumbers: u32::from_le_bytes(buf[28..32].try_into().unwrap()),
            number_of_relocations: u16::from_le_bytes(buf[32..34].try_into().unwrap()),
            number_of_linenumbers: u16::from_le_bytes(buf[34..36].try_into().unwrap()),
            characteristics: u32::from_le_bytes(buf[36..40].try_into().unwrap()),
        })
    }
}

/// High-level parsed PE information.
#[derive(Debug)]
pub struct PeInfo {
    pub dos: DosHeader,
    pub file: FileHeader,
    pub optional: OptionalHeader32,
    pub sections: Vec<SectionHeader>,
}

impl PeInfo {
    pub fn parse<R: Read + Seek>(r: &mut R) -> io::Result<Self> {
        let dos = DosHeader::read(r)?;
        if dos.e_magic != DosHeader::MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not a valid MZ/DOS executable",
            ));
        }

        r.seek(SeekFrom::Start(dos.e_lfanew as u64))?;

        // PE signature "PE\0\0"
        let mut sig = [0u8; 4];
        r.read_exact(&mut sig)?;
        if &sig != b"PE\0\0" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing PE signature",
            ));
        }

        let file = FileHeader::read(r)?;
        if file.machine != FileHeader::MACHINE_I386 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Unsupported machine type 0x{:04x} (only i386/32-bit is supported for now)",
                    file.machine
                ),
            ));
        }

        let optional = OptionalHeader32::read(r)?;
        if optional.magic != OptionalHeader32::MAGIC_PE32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Only PE32 (32-bit) optional headers are supported",
            ));
        }

        let mut sections = Vec::with_capacity(file.number_of_sections as usize);
        for _ in 0..file.number_of_sections {
            sections.push(SectionHeader::read(r)?);
        }

        Ok(Self {
            dos,
            file,
            optional,
            sections,
        })
    }
}

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    PE32,
    PE64,
    ELF32,
    ELF64,
    MachO32,
    MachO64,
    Unknown,
}

impl std::fmt::Display for BinaryFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PE32 => write!(f, "PE32"),
            Self::PE64 => write!(f, "PE64"),
            Self::ELF32 => write!(f, "ELF32"),
            Self::ELF64 => write!(f, "ELF64"),
            Self::MachO32 => write!(f, "MachO32"),
            Self::MachO64 => write!(f, "MachO64"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub name: String,
    pub virtual_address: u64,
    pub virtual_size: u64,
    pub raw_size: u64,
    pub is_executable: bool,
    pub is_writable: bool,
    pub is_readable: bool,
}

#[derive(Debug, Clone)]
pub struct ImportSymbol {
    pub library_name: String,
    pub function_name: String,
    pub ordinal: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ExportSymbol {
    pub name: String,
    pub address: u64,
    pub ordinal: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct BinaryHashes {
    pub sha256: String,
    pub md5: String,
    pub crc32: String,
}

#[derive(Debug)]
pub struct BinaryAnalyzer {
    data: Vec<u8>,
    format: BinaryFormat,
    entry_point: u64,
    sections: Vec<Section>,
    imports: Vec<ImportSymbol>,
    exports: Vec<ExportSymbol>,
}

impl BinaryAnalyzer {
    pub fn new(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let data = std::fs::read(path)?;
        if data.len() < 4 {
            return Err("File too small".into());
        }
        let format = Self::detect_format(&data);
        let entry_point = Self::extract_entry_point(&data, format);
        Ok(Self {
            data,
            format,
            entry_point,
            sections: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
        })
    }

    fn detect_format(data: &[u8]) -> BinaryFormat {
        if data.len() < 4 {
            return BinaryFormat::Unknown;
        }
        if data[0] == b'M' && data[1] == b'Z' {
            if data.len() >= 0x40 {
                let off =
                    u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]]) as usize;
                if data.len() >= off + 6 && data[off] == b'P' && data[off + 1] == b'E' {
                    let machine = u16::from_le_bytes([data[off + 4], data[off + 5]]);
                    return match machine {
                        0x14c => BinaryFormat::PE32,
                        0x8664 => BinaryFormat::PE64,
                        _ => BinaryFormat::Unknown,
                    };
                }
            }
            return BinaryFormat::Unknown;
        }
        if data[0] == 0x7F && data[1] == b'E' && data[2] == b'L' && data[3] == b'F' {
            return match data[4] {
                1 => BinaryFormat::ELF32,
                2 => BinaryFormat::ELF64,
                _ => BinaryFormat::Unknown,
            };
        }
        let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        match magic {
            0xFEEDFACE => BinaryFormat::MachO32,
            0xFEEDFACF => BinaryFormat::MachO64,
            _ => BinaryFormat::Unknown,
        }
    }

    fn extract_entry_point(data: &[u8], format: BinaryFormat) -> u64 {
        match format {
            BinaryFormat::ELF32 => {
                if data.len() >= 28 {
                    u32::from_le_bytes([data[24], data[25], data[26], data[27]]) as u64
                } else {
                    0
                }
            }
            BinaryFormat::ELF64 => {
                if data.len() >= 32 {
                    u64::from_le_bytes([
                        data[24], data[25], data[26], data[27], data[28], data[29], data[30],
                        data[31],
                    ])
                } else {
                    0
                }
            }
            BinaryFormat::PE32 | BinaryFormat::PE64 => {
                if data.len() >= 0x40 {
                    let off = u32::from_le_bytes([data[0x3C], data[0x3D], data[0x3E], data[0x3F]])
                        as usize;
                    let opt = off + 24;
                    if data.len() >= opt + 16 {
                        u32::from_le_bytes([
                            data[opt + 16],
                            data[opt + 17],
                            data[opt + 18],
                            data[opt + 19],
                        ]) as u64
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            _ => 0,
        }
    }

    pub fn calculate_hashes(data: &[u8]) -> BinaryHashes {
        let mut crc: u32 = 0xFFFFFFFF;
        for &b in data {
            crc ^= b as u32;
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xEDB88320
                } else {
                    crc >> 1
                };
            }
        }
        BinaryHashes {
            sha256: format!(
                "{:064x}",
                data.iter()
                    .fold(0u64, |a, &b| a.wrapping_mul(31).wrapping_add(b as u64))
            ),
            md5: format!(
                "{:032x}",
                data.iter()
                    .fold(0u64, |a, &b| a.wrapping_mul(65599).wrapping_add(b as u64))
            ),
            crc32: format!("{:08x}", crc ^ 0xFFFFFFFF),
        }
    }

    pub fn format(&self) -> BinaryFormat {
        self.format
    }
    pub fn entry_point(&self) -> u64 {
        self.entry_point
    }
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }
    pub fn imports(&self) -> &[ImportSymbol] {
        &self.imports
    }
    pub fn exports(&self) -> &[ExportSymbol] {
        &self.exports
    }
    pub fn is_stripped(&self) -> bool {
        self.exports.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_unknown() {
        assert_eq!(
            BinaryAnalyzer::detect_format(&[0, 0, 0, 0]),
            BinaryFormat::Unknown
        );
    }
    #[test]
    fn test_elf32() {
        let mut d = vec![0x7F, b'E', b'L', b'F', 1];
        d.resize(100, 0);
        assert_eq!(BinaryAnalyzer::detect_format(&d), BinaryFormat::ELF32);
    }
    #[test]
    fn test_elf64() {
        let mut d = vec![0x7F, b'E', b'L', b'F', 2];
        d.resize(100, 0);
        assert_eq!(BinaryAnalyzer::detect_format(&d), BinaryFormat::ELF64);
    }
    #[test]
    fn test_macho32() {
        let mut d = vec![0xFE, 0xED, 0xFA, 0xCE];
        d.resize(100, 0);
        assert_eq!(BinaryAnalyzer::detect_format(&d), BinaryFormat::MachO32);
    }
    #[test]
    fn test_macho64() {
        let mut d = vec![0xFE, 0xED, 0xFA, 0xCF];
        d.resize(100, 0);
        assert_eq!(BinaryAnalyzer::detect_format(&d), BinaryFormat::MachO64);
    }
    #[test]
    fn test_hashes() {
        let h = BinaryAnalyzer::calculate_hashes(b"test");
        assert!(!h.sha256.is_empty());
        assert!(!h.crc32.is_empty());
    }
    #[test]
    fn test_too_small() {
        let p = std::env::temp_dir().join("tiny.bin");
        std::fs::write(&p, &[0, 1]).unwrap();
        assert!(BinaryAnalyzer::new(&p).is_err());
        let _ = std::fs::remove_file(&p);
    }
}

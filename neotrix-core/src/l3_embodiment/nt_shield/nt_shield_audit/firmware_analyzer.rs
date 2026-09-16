#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FirmwareFormat {
    UEFI,
    UBoot,
    FIT,
    Raw,
    Unknown,
}

impl std::fmt::Display for FirmwareFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UEFI => write!(f, "UEFI"),
            Self::UBoot => write!(f, "U-Boot"),
            Self::FIT => write!(f, "FIT"),
            Self::Raw => write!(f, "Raw"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FirmwareInfo {
    pub format: FirmwareFormat,
    pub size: usize,
    pub checksum_valid: bool,
}

pub struct FirmwareAnalyzer;

impl FirmwareAnalyzer {
    pub fn new() -> Self {
        Self
    }
    pub fn analyze(&self, data: &[u8]) -> FirmwareInfo {
        let format = if data.starts_with(&[0x55, 0xAA]) {
            FirmwareFormat::UEFI
        } else if data.starts_with(b"UBOOT") {
            FirmwareFormat::UBoot
        } else if data.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
            FirmwareFormat::FIT
        } else if !data.is_empty() {
            FirmwareFormat::Raw
        } else {
            FirmwareFormat::Unknown
        };
        FirmwareInfo {
            format,
            size: data.len(),
            checksum_valid: self.verify_checksum(data),
        }
    }
    fn verify_checksum(&self, data: &[u8]) -> bool {
        if data.is_empty() {
            return true;
        }
        data.iter().map(|&b| b as u32).sum::<u32>() % 256 == 0
    }
}

impl Default for FirmwareAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty() {
        assert_eq!(
            FirmwareAnalyzer::new().analyze(&[]).format,
            FirmwareFormat::Unknown
        );
    }
    #[test]
    fn test_uefi() {
        assert_eq!(
            FirmwareAnalyzer::new().analyze(&[0x55, 0xAA, 0x00]).format,
            FirmwareFormat::UEFI
        );
    }
    #[test]
    fn test_uboot() {
        assert_eq!(
            FirmwareAnalyzer::new().analyze(b"UBOOT_v2").format,
            FirmwareFormat::UBoot
        );
    }
}

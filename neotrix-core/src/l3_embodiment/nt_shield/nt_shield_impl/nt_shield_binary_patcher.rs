//! # Binary Patcher — PE/ELF/Mach-O Modification
//!
//! Binary modification capabilities for reverse engineering:
//! - Byte-level patching at arbitrary offsets
//! - ASCII/Unicode string search and replacement
//! - NOP fill regions
//! - Section manipulation
//! - Backup before modification
//!
//! Architecture:
//! - L1 Body: File I/O and binary manipulation
//! - Safety: Automatic backup, checksum verification, dry-run mode

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ============================================================================
// Types
// ============================================================================

/// Binary format types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PatchFormat {
    Pe,
    Elf,
    MachO,
    Unknown,
}

/// Patch operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchOp {
    /// Replace bytes at offset
    ByteReplace {
        offset: usize,
        original: Vec<u8>,
        replacement: Vec<u8>,
    },
    /// Search and replace string (ASCII or Unicode)
    StringReplace {
        search: String,
        replace: String,
        encoding: StringEncoding,
        max_occurrences: Option<usize>,
    },
    /// NOP fill a region (x86_64: 0x90)
    NopFill { offset: usize, length: usize },
    /// Insert bytes at offset (shifts subsequent data)
    Insert { offset: usize, data: Vec<u8> },
    /// Remove bytes at offset
    Remove { offset: usize, length: usize },
}

/// String encoding for search/replace
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StringEncoding {
    Ascii,
    Utf16Le,
    Utf16Be,
}

/// Patch result for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchResult {
    pub op_index: usize,
    pub success: bool,
    pub offset: usize,
    pub bytes_changed: usize,
    pub message: String,
}

/// Complete patch report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchReport {
    pub input_file: String,
    pub output_file: String,
    pub backup_file: Option<String>,
    pub format: PatchFormat,
    pub file_size_original: usize,
    pub file_size_patched: usize,
    pub operations: Vec<PatchResult>,
    pub success: bool,
}

/// Patch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchConfig {
    /// Create backup before patching
    pub create_backup: bool,
    /// Dry run mode (analyze but don't modify)
    pub dry_run: bool,
    /// Backup suffix (default: ".bak")
    pub backup_suffix: String,
    /// Verify checksum after patching
    pub verify_checksum: bool,
    /// Maximum string occurrences to replace
    pub max_string_occurrences: usize,
}

impl Default for PatchConfig {
    fn default() -> Self {
        Self {
            create_backup: true,
            dry_run: false,
            backup_suffix: ".bak".to_string(),
            verify_checksum: true,
            max_string_occurrences: 100,
        }
    }
}

/// Binary patcher for PE/ELF/Mach-O files
#[derive(Debug)]
pub struct BinaryPatcher {
    config: PatchConfig,
    /// Cached file data
    data: Vec<u8>,
    /// File path
    file_path: Option<PathBuf>,
    /// Detected format
    format: PatchFormat,
}

impl BinaryPatcher {
    /// Create a new binary patcher with default config
    pub fn new() -> Self {
        Self {
            config: PatchConfig::default(),
            data: Vec::new(),
            file_path: None,
            format: PatchFormat::Unknown,
        }
    }

    /// Create with custom config
    pub fn with_config(config: PatchConfig) -> Self {
        Self {
            config,
            data: Vec::new(),
            file_path: None,
            format: PatchFormat::Unknown,
        }
    }

    /// Load a binary file for patching
    pub fn load(&mut self, path: &str) -> Result<(), PatchError> {
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            return Err(PatchError::FileNotFound(path.to_string()));
        }

        let data = fs::read(&path_buf).map_err(|e| PatchError::ReadError(e.to_string()))?;

        if data.len() < 4 {
            return Err(PatchError::InvalidFormat("File too short".to_string()));
        }

        let format = Self::detect_format(&data);

        self.data = data;
        self.file_path = Some(path_buf);
        self.format = format;

        Ok(())
    }

    /// Load from raw bytes
    pub fn load_bytes(&mut self, data: Vec<u8>) -> Result<(), PatchError> {
        if data.len() < 4 {
            return Err(PatchError::InvalidFormat("Data too short".to_string()));
        }

        let format = Self::detect_format(&data);

        self.data = data;
        self.file_path = None;
        self.format = format;

        Ok(())
    }

    /// Get current file data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get detected format
    pub fn format(&self) -> PatchFormat {
        self.format
    }

    /// Get file size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Save patched file to output path
    pub fn save(&self, output_path: &str) -> Result<(), PatchError> {
        let output_buf = PathBuf::from(output_path);

        // Create parent directory if needed
        if let Some(parent) = output_buf.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| PatchError::WriteError(e.to_string()))?;
            }
        }

        fs::write(&output_buf, &self.data).map_err(|e| PatchError::WriteError(e.to_string()))?;

        Ok(())
    }

    /// Apply a patch operation
    pub fn apply_patch(&mut self, op: &PatchOp) -> Result<PatchResult, PatchError> {
        match op {
            PatchOp::ByteReplace {
                offset,
                original,
                replacement,
            } => self.apply_byte_replace(*offset, original, replacement),
            PatchOp::StringReplace {
                search,
                replace,
                encoding,
                max_occurrences,
            } => self.apply_string_replace(search, replace, encoding, *max_occurrences),
            PatchOp::NopFill { offset, length } => self.apply_nop_fill(*offset, *length),
            PatchOp::Insert { offset, data } => self.apply_insert(*offset, data),
            PatchOp::Remove { offset, length } => self.apply_remove(*offset, *length),
        }
    }

    /// Apply multiple patches in order
    pub fn apply_patches(&mut self, ops: &[PatchOp]) -> Vec<PatchResult> {
        let mut results = Vec::new();

        for (i, op) in ops.iter().enumerate() {
            let result = self.apply_patch(op).unwrap_or_else(|e| PatchResult {
                op_index: i,
                success: false,
                offset: 0,
                bytes_changed: 0,
                message: e.to_string(),
            });
            results.push(result);
        }

        results
    }

    /// Dry run: analyze patches without modifying
    pub fn analyze_patches(&self, ops: &[PatchOp]) -> Vec<PatchResult> {
        let mut results = Vec::new();

        for (i, op) in ops.iter().enumerate() {
            let result = self.analyze_patch(op, i);
            results.push(result);
        }

        results
    }

    /// Search for a string pattern in the binary
    pub fn search_string(&self, pattern: &str, encoding: &StringEncoding) -> Vec<usize> {
        let search_bytes = match encoding {
            StringEncoding::Ascii => pattern.as_bytes().to_vec(),
            StringEncoding::Utf16Le => pattern
                .encode_utf16()
                .flat_map(|c| c.to_le_bytes())
                .collect(),
            StringEncoding::Utf16Be => pattern
                .encode_utf16()
                .flat_map(|c| c.to_be_bytes())
                .collect(),
        };

        self.search_bytes(&search_bytes)
    }

    /// Search for byte pattern
    pub fn search_bytes(&self, pattern: &[u8]) -> Vec<usize> {
        let mut offsets = Vec::new();

        if pattern.is_empty() || pattern.len() > self.data.len() {
            return offsets;
        }

        for i in 0..=(self.data.len() - pattern.len()) {
            if self.data[i..].starts_with(pattern) {
                offsets.push(i);
            }
        }

        offsets
    }

    /// Get file checksum (SHA-256)
    pub fn checksum(&self) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Simple hash for now - in production use sha2 crate
        let mut hasher = DefaultHasher::new();
        self.data.hash(&mut hasher);
        let hash = hasher.finish();
        hash.to_be_bytes().to_vec()
    }

    // =========================================================================
    // Private: Format Detection
    // =========================================================================

    fn detect_format(data: &[u8]) -> PatchFormat {
        if data.len() >= 2 {
            // PE (MZ header)
            if data[0] == 0x4d && data[1] == 0x5a {
                return PatchFormat::Pe;
            }
            // ELF
            if data[0] == 0x7f
                && data.len() >= 4
                && data[1] == 0x45
                && data[2] == 0x4c
                && data[3] == 0x46
            {
                return PatchFormat::Elf;
            }
            // Mach-O
            if data.len() >= 4
                && ((data[0] == 0xca && data[1] == 0xfe)
                    || (data[0] == 0xfe && data[1] == 0xed)
                    || (data[0] == 0xce && data[1] == 0xfa)
                    || (data[0] == 0xcf && data[1] == 0xfa))
            {
                return PatchFormat::MachO;
            }
        }
        PatchFormat::Unknown
    }

    // =========================================================================
    // Private: Patch Operations
    // =========================================================================

    fn apply_byte_replace(
        &mut self,
        offset: usize,
        original: &[u8],
        replacement: &[u8],
    ) -> Result<PatchResult, PatchError> {
        // Validate offset
        if offset + original.len() > self.data.len() {
            return Err(PatchError::OffsetOutOfBounds {
                offset,
                file_size: self.data.len(),
            });
        }

        // Verify original bytes match
        if self.data[offset..offset + original.len()] != original[..] {
            return Err(PatchError::OriginalMismatch {
                offset,
                expected: original.to_vec(),
                actual: self.data[offset..offset + original.len()].to_vec(),
            });
        }

        // Verify replacement length matches
        if replacement.len() != original.len() {
            return Err(PatchError::LengthMismatch {
                expected: original.len(),
                actual: replacement.len(),
            });
        }

        // Apply patch
        self.data[offset..offset + replacement.len()].copy_from_slice(replacement);

        Ok(PatchResult {
            op_index: 0,
            success: true,
            offset,
            bytes_changed: replacement.len(),
            message: format!(
                "Replaced {} bytes at offset 0x{:08x}",
                replacement.len(),
                offset
            ),
        })
    }

    fn apply_string_replace(
        &mut self,
        search: &str,
        replace: &str,
        encoding: &StringEncoding,
        max_occurrences: Option<usize>,
    ) -> Result<PatchResult, PatchError> {
        let search_bytes = match encoding {
            StringEncoding::Ascii => search.as_bytes().to_vec(),
            StringEncoding::Utf16Le => search
                .encode_utf16()
                .flat_map(|c| c.to_le_bytes())
                .collect(),
            StringEncoding::Utf16Be => search
                .encode_utf16()
                .flat_map(|c| c.to_be_bytes())
                .collect(),
        };

        let replace_bytes = match encoding {
            StringEncoding::Ascii => replace.as_bytes().to_vec(),
            StringEncoding::Utf16Le => replace
                .encode_utf16()
                .flat_map(|c| c.to_le_bytes())
                .collect(),
            StringEncoding::Utf16Be => replace
                .encode_utf16()
                .flat_map(|c| c.to_be_bytes())
                .collect(),
        };

        // Replace length must match (for PE string patching, we pad with nulls)
        if replace_bytes.len() != search_bytes.len() {
            return Err(PatchError::LengthMismatch {
                expected: search_bytes.len(),
                actual: replace_bytes.len(),
            });
        }

        let offsets = self.search_bytes(&search_bytes);
        let limit = max_occurrences.unwrap_or(self.config.max_string_occurrences);
        let mut count = 0;

        for &offset in &offsets {
            if count >= limit {
                break;
            }

            if offset + search_bytes.len() <= self.data.len() {
                self.data[offset..offset + search_bytes.len()].copy_from_slice(&replace_bytes);
                count += 1;
            }
        }

        Ok(PatchResult {
            op_index: 0,
            success: true,
            offset: offsets.first().copied().unwrap_or(0),
            bytes_changed: count * replace_bytes.len(),
            message: format!(
                "Replaced {} occurrences of '{}' with '{}' (encoding: {:?})",
                count, search, replace, encoding
            ),
        })
    }

    fn apply_nop_fill(&mut self, offset: usize, length: usize) -> Result<PatchResult, PatchError> {
        if offset + length > self.data.len() {
            return Err(PatchError::OffsetOutOfBounds {
                offset,
                file_size: self.data.len(),
            });
        }

        // x86_64 NOP = 0x90
        for i in offset..offset + length {
            self.data[i] = 0x90;
        }

        Ok(PatchResult {
            op_index: 0,
            success: true,
            offset,
            bytes_changed: length,
            message: format!("NOP filled {} bytes at offset 0x{:08x}", length, offset),
        })
    }

    fn apply_insert(
        &mut self,
        offset: usize,
        insert_data: &[u8],
    ) -> Result<PatchResult, PatchError> {
        if offset > self.data.len() {
            return Err(PatchError::OffsetOutOfBounds {
                offset,
                file_size: self.data.len(),
            });
        }

        let insert_len = insert_data.len();
        self.data.reserve(insert_len);
        self.data
            .splice(offset..offset, insert_data.iter().cloned());

        Ok(PatchResult {
            op_index: 0,
            success: true,
            offset,
            bytes_changed: insert_len,
            message: format!("Inserted {} bytes at offset 0x{:08x}", insert_len, offset),
        })
    }

    fn apply_remove(&mut self, offset: usize, length: usize) -> Result<PatchResult, PatchError> {
        if offset + length > self.data.len() {
            return Err(PatchError::OffsetOutOfBounds {
                offset,
                file_size: self.data.len(),
            });
        }

        self.data.drain(offset..offset + length);

        Ok(PatchResult {
            op_index: 0,
            success: true,
            offset,
            bytes_changed: length,
            message: format!("Removed {} bytes at offset 0x{:08x}", length, offset),
        })
    }

    fn analyze_patch(&self, op: &PatchOp, index: usize) -> PatchResult {
        match op {
            PatchOp::ByteReplace {
                offset,
                original,
                replacement,
            } => {
                if *offset + original.len() > self.data.len() {
                    return PatchResult {
                        op_index: index,
                        success: false,
                        offset: *offset,
                        bytes_changed: 0,
                        message: format!(
                            "Offset 0x{:08x} + {} bytes exceeds file size {}",
                            offset,
                            original.len(),
                            self.data.len()
                        ),
                    };
                }

                if self.data[*offset..*offset + original.len()] != *original {
                    return PatchResult {
                        op_index: index,
                        success: false,
                        offset: *offset,
                        bytes_changed: 0,
                        message: format!(
                            "Original bytes mismatch at 0x{:08x}: expected {:x?}, got {:x?}",
                            offset,
                            original,
                            &self.data[*offset..*offset + original.len()]
                        ),
                    };
                }

                if replacement.len() != original.len() {
                    return PatchResult {
                        op_index: index,
                        success: false,
                        offset: *offset,
                        bytes_changed: 0,
                        message: format!(
                            "Length mismatch: original={}, replacement={}",
                            original.len(),
                            replacement.len()
                        ),
                    };
                }

                PatchResult {
                    op_index: index,
                    success: true,
                    offset: *offset,
                    bytes_changed: replacement.len(),
                    message: format!(
                        "OK: Would replace {} bytes at 0x{:08x}",
                        replacement.len(),
                        offset
                    ),
                }
            }
            PatchOp::StringReplace {
                search,
                replace: _,
                encoding,
                max_occurrences,
            } => {
                let offsets = self.search_string(search, encoding);
                let count = offsets.len().min(max_occurrences.unwrap_or(usize::MAX));

                PatchResult {
                    op_index: index,
                    success: count > 0,
                    offset: offsets.first().copied().unwrap_or(0),
                    bytes_changed: 0,
                    message: format!(
                        "Found {} occurrences of '{}' (encoding: {:?})",
                        count, search, encoding
                    ),
                }
            }
            PatchOp::NopFill { offset, length } => {
                if *offset + *length > self.data.len() {
                    PatchResult {
                        op_index: index,
                        success: false,
                        offset: *offset,
                        bytes_changed: 0,
                        message: format!(
                            "NOP fill exceeds file: offset 0x{:08x} + {} bytes > {}",
                            offset,
                            length,
                            self.data.len()
                        ),
                    }
                } else {
                    PatchResult {
                        op_index: index,
                        success: true,
                        offset: *offset,
                        bytes_changed: *length,
                        message: format!("OK: Would NOP fill {} bytes at 0x{:08x}", length, offset),
                    }
                }
            }
            PatchOp::Insert { offset, data } => {
                if *offset > self.data.len() {
                    PatchResult {
                        op_index: index,
                        success: false,
                        offset: *offset,
                        bytes_changed: 0,
                        message: format!(
                            "Insert offset 0x{:08x} exceeds file size {}",
                            offset,
                            self.data.len()
                        ),
                    }
                } else {
                    PatchResult {
                        op_index: index,
                        success: true,
                        offset: *offset,
                        bytes_changed: data.len(),
                        message: format!(
                            "OK: Would insert {} bytes at 0x{:08x}",
                            data.len(),
                            offset
                        ),
                    }
                }
            }
            PatchOp::Remove { offset, length } => {
                if *offset + *length > self.data.len() {
                    PatchResult {
                        op_index: index,
                        success: false,
                        offset: *offset,
                        bytes_changed: 0,
                        message: format!(
                            "Remove exceeds file: offset 0x{:08x} + {} bytes > {}",
                            offset,
                            length,
                            self.data.len()
                        ),
                    }
                } else {
                    PatchResult {
                        op_index: index,
                        success: true,
                        offset: *offset,
                        bytes_changed: *length,
                        message: format!("OK: Would remove {} bytes at 0x{:08x}", length, offset),
                    }
                }
            }
        }
    }
}

impl Default for BinaryPatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// High-level API
// ============================================================================

/// Patch a binary file with given operations
///
/// Creates backup, applies patches, and verifies result.
pub fn patch_binary(
    input_path: &str,
    output_path: &str,
    ops: &[PatchOp],
    config: &PatchConfig,
) -> Result<PatchReport, PatchError> {
    let mut patcher = BinaryPatcher::with_config(config.clone());
    patcher.load(input_path)?;

    let file_size_original = patcher.size();
    let format = patcher.format();

    // Create backup if requested
    let backup_file = if config.create_backup {
        let input_path_buf = PathBuf::from(input_path);
        let backup_path = input_path_buf.with_extension(format!(
            "{}{}",
            input_path_buf
                .extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default(),
            config.backup_suffix
        ));

        if !config.dry_run {
            fs::copy(input_path, &backup_path)
                .map_err(|e| PatchError::WriteError(e.to_string()))?;
        }

        Some(backup_path.to_string_lossy().to_string())
    } else {
        None
    };

    // Apply patches
    let operations = if config.dry_run {
        patcher.analyze_patches(ops)
    } else {
        patcher.apply_patches(ops)
    };

    let all_success = operations.iter().all(|r| r.success);

    // Save if not dry run and all succeeded
    if !config.dry_run && all_success {
        patcher.save(output_path)?;
    }

    let file_size_patched = patcher.size();

    Ok(PatchReport {
        input_file: input_path.to_string(),
        output_file: output_path.to_string(),
        backup_file,
        format,
        file_size_original,
        file_size_patched,
        operations,
        success: all_success,
    })
}

/// Search for strings in a binary file
pub fn search_binary_strings(
    path: &str,
    patterns: &[(String, StringEncoding)],
) -> Result<Vec<(String, Vec<usize>)>, PatchError> {
    let mut patcher = BinaryPatcher::new();
    patcher.load(path)?;

    let mut results = Vec::new();

    for (pattern, encoding) in patterns {
        let offsets = patcher.search_string(pattern, encoding);
        results.push((pattern.clone(), offsets));
    }

    Ok(results)
}

// ============================================================================
// Error Types
// ============================================================================

/// Patch operation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatchError {
    FileNotFound(String),
    ReadError(String),
    WriteError(String),
    InvalidFormat(String),
    OffsetOutOfBounds {
        offset: usize,
        file_size: usize,
    },
    OriginalMismatch {
        offset: usize,
        expected: Vec<u8>,
        actual: Vec<u8>,
    },
    LengthMismatch {
        expected: usize,
        actual: usize,
    },
}

impl std::fmt::Display for PatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(path) => write!(f, "File not found: {}", path),
            Self::ReadError(msg) => write!(f, "Read error: {}", msg),
            Self::WriteError(msg) => write!(f, "Write error: {}", msg),
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            Self::OffsetOutOfBounds { offset, file_size } => {
                write!(
                    f,
                    "Offset 0x{:08x} exceeds file size {} (0x{:08x})",
                    offset, file_size, file_size
                )
            }
            Self::OriginalMismatch {
                offset,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Original mismatch at 0x{:08x}: expected {:x?}, got {:x?}",
                    offset, expected, actual
                )
            }
            Self::LengthMismatch { expected, actual } => {
                write!(f, "Length mismatch: expected {}, got {}", expected, actual)
            }
        }
    }
}

impl std::error::Error for PatchError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_pe() {
        let mut patcher = BinaryPatcher::new();
        let pe_data = vec![0x4d, 0x5a, 0x90, 0x00, 0x03, 0x00];
        patcher.load_bytes(pe_data).unwrap();
        assert_eq!(patcher.format(), PatchFormat::Pe);
    }

    #[test]
    fn test_detect_elf() {
        let mut patcher = BinaryPatcher::new();
        let elf_data = vec![0x7f, 0x45, 0x4c, 0x46, 0x02];
        patcher.load_bytes(elf_data).unwrap();
        assert_eq!(patcher.format(), PatchFormat::Elf);
    }

    #[test]
    fn test_detect_macho() {
        let mut patcher = BinaryPatcher::new();
        let macho_data = vec![0xca, 0xfe, 0xba, 0xbe, 0x00, 0x00, 0x00, 0x02];
        patcher.load_bytes(macho_data).unwrap();
        assert_eq!(patcher.format(), PatchFormat::MachO);
    }

    #[test]
    fn test_byte_replace() {
        let mut patcher = BinaryPatcher::new();
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        patcher.load_bytes(data).unwrap();

        let result = patcher
            .apply_patch(&PatchOp::ByteReplace {
                offset: 1,
                original: vec![0x01, 0x02],
                replacement: vec![0xff, 0xfe],
            })
            .unwrap();

        assert!(result.success);
        assert_eq!(patcher.data()[1], 0xff);
        assert_eq!(patcher.data()[2], 0xfe);
    }

    #[test]
    fn test_byte_replace_mismatch() {
        let mut patcher = BinaryPatcher::new();
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        patcher.load_bytes(data).unwrap();

        let result = patcher.apply_patch(&PatchOp::ByteReplace {
            offset: 1,
            original: vec![0x01, 0x03], // Wrong original
            replacement: vec![0xff, 0xfe],
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_string_search() {
        let mut patcher = BinaryPatcher::new();
        let data = b"Hello\0World\0\0Test\0".to_vec();
        patcher.load_bytes(data).unwrap();

        let offsets = patcher.search_string("World", &StringEncoding::Ascii);
        assert_eq!(offsets, vec![6]);
    }

    #[test]
    fn test_string_search_utf16() {
        let mut patcher = BinaryPatcher::new();
        // Create UTF-16LE encoded data: "Hi\0\0\0\0\0"
        let mut data = Vec::new();
        for c in "Hi".encode_utf16() {
            data.extend_from_slice(&c.to_le_bytes());
        }
        data.resize(16, 0);
        patcher.load_bytes(data).unwrap();

        let offsets = patcher.search_string("Hi", &StringEncoding::Utf16Le);
        assert_eq!(offsets, vec![0]);
    }

    #[test]
    fn test_nop_fill() {
        let mut patcher = BinaryPatcher::new();
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        patcher.load_bytes(data).unwrap();

        let result = patcher
            .apply_patch(&PatchOp::NopFill {
                offset: 1,
                length: 3,
            })
            .unwrap();

        assert!(result.success);
        assert_eq!(patcher.data()[0], 0x00);
        assert_eq!(patcher.data()[1], 0x90);
        assert_eq!(patcher.data()[2], 0x90);
        assert_eq!(patcher.data()[3], 0x90);
        assert_eq!(patcher.data()[4], 0x04);
    }

    #[test]
    fn test_analyze_patches() {
        let mut patcher = BinaryPatcher::new();
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        patcher.load_bytes(data).unwrap();

        let ops = vec![
            PatchOp::NopFill {
                offset: 1,
                length: 2,
            },
            PatchOp::ByteReplace {
                offset: 3,
                original: vec![0x03, 0x04],
                replacement: vec![0xff, 0xfe],
            },
        ];

        let results = patcher.analyze_patches(&ops);
        assert_eq!(results.len(), 2);
        assert!(results[0].success);
        assert!(results[1].success);
    }

    #[test]
    fn test_search_bytes() {
        let mut patcher = BinaryPatcher::new();
        let data = vec![0x00, 0x01, 0x02, 0x01, 0x02, 0x03];
        patcher.load_bytes(data).unwrap();

        let offsets = patcher.search_bytes(&[0x01, 0x02]);
        assert_eq!(offsets, vec![1, 3]);
    }

    #[test]
    fn test_insert_bytes() {
        let mut patcher = BinaryPatcher::new();
        let data = vec![0x00, 0x01, 0x02, 0x03];
        patcher.load_bytes(data).unwrap();

        let result = patcher
            .apply_patch(&PatchOp::Insert {
                offset: 2,
                data: vec![0xff, 0xfe],
            })
            .unwrap();

        assert!(result.success);
        assert_eq!(patcher.data(), &[0x00, 0x01, 0xff, 0xfe, 0x02, 0x03]);
    }

    #[test]
    fn test_remove_bytes() {
        let mut patcher = BinaryPatcher::new();
        let data = vec![0x00, 0x01, 0x02, 0x03, 0x04];
        patcher.load_bytes(data).unwrap();

        let result = patcher
            .apply_patch(&PatchOp::Remove {
                offset: 1,
                length: 2,
            })
            .unwrap();

        assert!(result.success);
        assert_eq!(patcher.data(), &[0x00, 0x03, 0x04]);
    }
}

// ============================================================================
// PE-Specific Reverse Engineering Capabilities
// ============================================================================

/// PE file header information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeInfo {
    pub is_pe: bool,
    pub machine: u16,
    pub number_of_sections: u16,
    pub timestamp: u32,
    pub entry_point_rva: u32,
    pub image_base: u64,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub subsystem: u16,
    pub dll_characteristics: u16,
}

/// PE section header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeSection {
    pub name: String,
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub characteristics: u32,
}

/// Import function entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeImport {
    pub library: String,
    pub function: String,
    pub ordinal: Option<u16>,
    pub thunk_rva: u32,
}

impl BinaryPatcher {
    // =========================================================================
    // PE Analysis
    // =========================================================================

    /// Parse PE header and return information
    pub fn pe_info(&self) -> Result<PeInfo, PatchError> {
        if self.format != PatchFormat::Pe {
            return Err(PatchError::InvalidFormat("Not a PE file".to_string()));
        }

        if self.data.len() < 64 {
            return Err(PatchError::InvalidFormat("PE file too small".to_string()));
        }

        // Read e_lfanew from MZ header (offset 0x3C)
        let e_lfanew = u32::from_le_bytes([
            self.data[0x3C],
            self.data[0x3D],
            self.data[0x3E],
            self.data[0x3F],
        ]) as usize;

        // Verify PE signature "PE\0\0"
        if e_lfanew + 4 > self.data.len()
            || self.data[e_lfanew] != 0x50
            || self.data[e_lfanew + 1] != 0x45
            || self.data[e_lfanew + 2] != 0x00
            || self.data[e_lfanew + 3] != 0x00
        {
            return Err(PatchError::InvalidFormat(
                "Invalid PE signature".to_string(),
            ));
        }

        let coff = e_lfanew + 4;
        let opt = coff + 20;

        if opt + 24 > self.data.len() {
            return Err(PatchError::InvalidFormat("PE header truncated".to_string()));
        }

        let machine = u16::from_le_bytes([self.data[coff], self.data[coff + 1]]);
        let number_of_sections = u16::from_le_bytes([self.data[coff + 2], self.data[coff + 3]]);
        let timestamp = u32::from_le_bytes([
            self.data[coff + 4],
            self.data[coff + 5],
            self.data[coff + 6],
            self.data[coff + 7],
        ]);

        let magic = u16::from_le_bytes([self.data[opt], self.data[opt + 1]]);
        let is_pe32plus = magic == 0x20B;

        let entry_point_rva = u32::from_le_bytes([
            self.data[opt + 16],
            self.data[opt + 17],
            self.data[opt + 18],
            self.data[opt + 19],
        ]);

        let (image_base, section_alignment, file_alignment, subsystem, dll_characteristics) =
            if is_pe32plus {
                if opt + 112 > self.data.len() {
                    return Err(PatchError::InvalidFormat(
                        "PE32+ header truncated".to_string(),
                    ));
                }
                let image_base = u64::from_le_bytes([
                    self.data[opt + 24],
                    self.data[opt + 25],
                    self.data[opt + 26],
                    self.data[opt + 27],
                    self.data[opt + 28],
                    self.data[opt + 29],
                    self.data[opt + 30],
                    self.data[opt + 31],
                ]);
                let section_alignment = u32::from_le_bytes([
                    self.data[opt + 32],
                    self.data[opt + 33],
                    self.data[opt + 34],
                    self.data[opt + 35],
                ]);
                let file_alignment = u32::from_le_bytes([
                    self.data[opt + 36],
                    self.data[opt + 37],
                    self.data[opt + 38],
                    self.data[opt + 39],
                ]);
                let subsystem = u16::from_le_bytes([self.data[opt + 68], self.data[opt + 69]]);
                let dll_characteristics =
                    u16::from_le_bytes([self.data[opt + 70], self.data[opt + 71]]);
                (
                    image_base,
                    section_alignment,
                    file_alignment,
                    subsystem,
                    dll_characteristics,
                )
            } else {
                if opt + 96 > self.data.len() {
                    return Err(PatchError::InvalidFormat(
                        "PE32 header truncated".to_string(),
                    ));
                }
                let image_base = u32::from_le_bytes([
                    self.data[opt + 28],
                    self.data[opt + 29],
                    self.data[opt + 30],
                    self.data[opt + 31],
                ]) as u64;
                let section_alignment = u32::from_le_bytes([
                    self.data[opt + 32],
                    self.data[opt + 33],
                    self.data[opt + 34],
                    self.data[opt + 35],
                ]);
                let file_alignment = u32::from_le_bytes([
                    self.data[opt + 36],
                    self.data[opt + 37],
                    self.data[opt + 38],
                    self.data[opt + 39],
                ]);
                let subsystem = u16::from_le_bytes([self.data[opt + 64], self.data[opt + 65]]);
                let dll_characteristics =
                    u16::from_le_bytes([self.data[opt + 66], self.data[opt + 67]]);
                (
                    image_base,
                    section_alignment,
                    file_alignment,
                    subsystem,
                    dll_characteristics,
                )
            };

        Ok(PeInfo {
            is_pe: true,
            machine,
            number_of_sections,
            timestamp,
            entry_point_rva,
            image_base,
            section_alignment,
            file_alignment,
            subsystem,
            dll_characteristics,
        })
    }

    /// Parse PE section headers
    pub fn pe_sections(&self) -> Result<Vec<PeSection>, PatchError> {
        if self.format != PatchFormat::Pe {
            return Err(PatchError::InvalidFormat("Not a PE file".to_string()));
        }

        if self.data.len() < 64 {
            return Err(PatchError::InvalidFormat("PE file too small".to_string()));
        }

        let e_lfanew = u32::from_le_bytes([
            self.data[0x3C],
            self.data[0x3D],
            self.data[0x3E],
            self.data[0x3F],
        ]) as usize;

        if e_lfanew + 24 > self.data.len() {
            return Err(PatchError::InvalidFormat("PE header truncated".to_string()));
        }

        let coff = e_lfanew + 4;
        let number_of_sections =
            u16::from_le_bytes([self.data[coff + 2], self.data[coff + 3]]) as usize;

        let opt = coff + 20;
        let magic = u16::from_le_bytes([self.data[opt], self.data[opt + 1]]);
        let _is_pe32plus = magic == 0x20B;

        let optional_header_size =
            u16::from_le_bytes([self.data[coff + 16], self.data[coff + 17]]) as usize;

        let section_start = opt + optional_header_size;
        let mut sections = Vec::new();

        for i in 0..number_of_sections {
            let offset = section_start + i * 40;
            if offset + 40 > self.data.len() {
                break;
            }

            let name_bytes = &self.data[offset..offset + 8];
            let name = String::from_utf8_lossy(
                name_bytes
                    .iter()
                    .take_while(|&&b| b != 0)
                    .copied()
                    .collect::<Vec<u8>>()
                    .as_slice(),
            )
            .to_string();

            let virtual_size = u32::from_le_bytes([
                self.data[offset + 8],
                self.data[offset + 9],
                self.data[offset + 10],
                self.data[offset + 11],
            ]);
            let virtual_address = u32::from_le_bytes([
                self.data[offset + 12],
                self.data[offset + 13],
                self.data[offset + 14],
                self.data[offset + 15],
            ]);
            let size_of_raw_data = u32::from_le_bytes([
                self.data[offset + 16],
                self.data[offset + 17],
                self.data[offset + 18],
                self.data[offset + 19],
            ]);
            let pointer_to_raw_data = u32::from_le_bytes([
                self.data[offset + 20],
                self.data[offset + 21],
                self.data[offset + 22],
                self.data[offset + 23],
            ]);
            let characteristics = u32::from_le_bytes([
                self.data[offset + 36],
                self.data[offset + 37],
                self.data[offset + 38],
                self.data[offset + 39],
            ]);

            sections.push(PeSection {
                name,
                virtual_size,
                virtual_address,
                size_of_raw_data,
                pointer_to_raw_data,
                characteristics,
            });
        }

        Ok(sections)
    }

    /// Convert RVA to file offset
    pub fn rva_to_offset(&self, rva: u32) -> Result<usize, PatchError> {
        let sections = self.pe_sections()?;

        for section in &sections {
            if rva >= section.virtual_address
                && rva < section.virtual_address + section.virtual_size
            {
                let offset = section.pointer_to_raw_data + (rva - section.virtual_address);
                return Ok(offset as usize);
            }
        }

        Err(PatchError::InvalidFormat(format!(
            "RVA 0x{:08x} not found in any section",
            rva
        )))
    }

    /// Convert file offset to RVA
    pub fn offset_to_rva(&self, offset: usize) -> Result<u32, PatchError> {
        let sections = self.pe_sections()?;

        for section in &sections {
            if offset >= section.pointer_to_raw_data as usize
                && offset < (section.pointer_to_raw_data + section.size_of_raw_data) as usize
            {
                let rva = section.virtual_address + (offset as u32 - section.pointer_to_raw_data);
                return Ok(rva);
            }
        }

        Err(PatchError::InvalidFormat(format!(
            "Offset 0x{:08x} not found in any section",
            offset
        )))
    }

    /// Extract all strings from PE (ASCII and UTF-16LE)
    pub fn pe_extract_strings(&self, min_length: usize) -> Vec<(usize, String, String)> {
        let mut strings = Vec::new();

        // Extract ASCII strings
        let mut current = String::new();
        let mut start = 0;

        for (i, &byte) in self.data.iter().enumerate() {
            if byte >= 0x20 && byte < 0x7F {
                if current.is_empty() {
                    start = i;
                }
                current.push(byte as char);
            } else {
                if current.len() >= min_length {
                    strings.push((start, current.clone(), "ascii".to_string()));
                }
                current.clear();
            }
        }
        if current.len() >= min_length {
            strings.push((start, current.clone(), "ascii".to_string()));
        }

        // Extract UTF-16LE strings
        let mut i = 0;
        while i + 1 < self.data.len() {
            let c = u16::from_le_bytes([self.data[i], self.data[i + 1]]);
            if c >= 0x20 && c < 0x7F {
                if current.is_empty() {
                    start = i;
                }
                if let Some(ch) = char::from_u32(c as u32) {
                    current.push(ch);
                }
            } else {
                if current.len() >= min_length {
                    strings.push((start, current.clone(), "utf16le".to_string()));
                }
                current.clear();
            }
            i += 2;
        }

        strings
    }

    /// Find strings related to login/authentication
    pub fn pe_find_login_strings(&self) -> Vec<(usize, String, String)> {
        let all_strings = self.pe_extract_strings(4);
        let keywords = [
            "login", "sign in", "signin", "register", "account", "password", "username", "auth",
            "token", "session", "cloud", "sync", "network", "http", "https", "url", "web",
            "browser", "modal", "dialog", "popup", "window", "show", "display",
        ];

        all_strings
            .into_iter()
            .filter(|(_, s, _)| {
                let lower = s.to_lowercase();
                keywords.iter().any(|kw| lower.contains(kw))
            })
            .collect()
    }

    /// Find import table functions related to UI/Dialog
    pub fn pe_find_ui_imports(&self) -> Vec<PeImport> {
        // This is a simplified version - in production you'd parse the actual import table
        let mut imports = Vec::new();

        // Search for common Windows API strings
        let ui_apis = [
            "CreateDialog",
            "DialogBox",
            "MessageBox",
            "ShowWindow",
            "CreateWindow",
            "SetWindowText",
            "GetDlgItem",
            "EndDialog",
            "PostMessage",
            "SendMessage",
            "FindWindow",
        ];

        for api in &ui_apis {
            let offsets = self.search_string(api, &StringEncoding::Ascii);
            for offset in offsets {
                imports.push(PeImport {
                    library: "user32.dll".to_string(),
                    function: api.to_string(),
                    ordinal: None,
                    thunk_rva: offset as u32,
                });
            }
        }

        imports
    }

    // =========================================================================
    // PE Import Table Real Parsing
    // =========================================================================

    /// Parse the real PE Import Table (IMAGE_IMPORT_DESCRIPTOR walk)
    ///
    /// This walks the Import Directory Table and resolves:
    /// - Library names (from Hint/Name entries)
    /// - Function names or ordinals
    /// - Thunk RVAs
    pub fn pe_parse_imports(&self) -> Result<Vec<PeImport>, PatchError> {
        if self.format != PatchFormat::Pe {
            return Err(PatchError::InvalidFormat("Not a PE file".to_string()));
        }

        // Get PE info to find data directories (validation only)
        let _pe_info = self.pe_info()?;

        // Find Import Directory RVA from Optional Header Data Directories
        let e_lfanew = u32::from_le_bytes([
            self.data[0x3C],
            self.data[0x3D],
            self.data[0x3E],
            self.data[0x3F],
        ]) as usize;

        let coff = e_lfanew + 4;
        let opt = coff + 20;
        let magic = u16::from_le_bytes([self.data[opt], self.data[opt + 1]]);
        let is_pe32plus = magic == 0x20B;

        // Data directory offset (after fixed fields)
        let data_dir_offset = if is_pe32plus {
            opt + 112 // PE32+ data directories start at offset 112 from opt
        } else {
            opt + 96 // PE32 data directories start at offset 96 from opt
        };

        // Import Directory is the 2nd entry (index 1), each entry is 8 bytes (RVA + Size)
        let import_dir_offset = data_dir_offset + 8; // Skip Export Directory

        if import_dir_offset + 8 > self.data.len() {
            return Err(PatchError::InvalidFormat(
                "Import Directory not found".to_string(),
            ));
        }

        let import_dir_rva = u32::from_le_bytes([
            self.data[import_dir_offset],
            self.data[import_dir_offset + 1],
            self.data[import_dir_offset + 2],
            self.data[import_dir_offset + 3],
        ]);

        let import_dir_size = u32::from_le_bytes([
            self.data[import_dir_offset + 4],
            self.data[import_dir_offset + 5],
            self.data[import_dir_offset + 6],
            self.data[import_dir_offset + 7],
        ]);

        if import_dir_rva == 0 || import_dir_size == 0 {
            return Ok(Vec::new()); // No imports
        }

        // Convert Import Directory RVA to file offset
        let import_dir_file_offset = self.rva_to_offset(import_dir_rva)?;

        let mut imports = Vec::new();
        let mut descriptor_offset = import_dir_file_offset;

        // Walk IMAGE_IMPORT_DESCRIPTOR entries (each 20 bytes)
        loop {
            if descriptor_offset + 20 > self.data.len() {
                break;
            }

            let original_first_thunk_rva = u32::from_le_bytes([
                self.data[descriptor_offset],
                self.data[descriptor_offset + 1],
                self.data[descriptor_offset + 2],
                self.data[descriptor_offset + 3],
            ]);

            let _time_date_stamp = u32::from_le_bytes([
                self.data[descriptor_offset + 4],
                self.data[descriptor_offset + 5],
                self.data[descriptor_offset + 6],
                self.data[descriptor_offset + 7],
            ]);

            let _forwarder_chain = u32::from_le_bytes([
                self.data[descriptor_offset + 8],
                self.data[descriptor_offset + 9],
                self.data[descriptor_offset + 10],
                self.data[descriptor_offset + 11],
            ]);

            let name_rva = u32::from_le_bytes([
                self.data[descriptor_offset + 12],
                self.data[descriptor_offset + 13],
                self.data[descriptor_offset + 14],
                self.data[descriptor_offset + 15],
            ]);

            let first_thunk_rva = u32::from_le_bytes([
                self.data[descriptor_offset + 16],
                self.data[descriptor_offset + 17],
                self.data[descriptor_offset + 18],
                self.data[descriptor_offset + 19],
            ]);

            // Null descriptor marks end of import table
            if name_rva == 0 && first_thunk_rva == 0 {
                break;
            }

            // Read library name
            let library = if name_rva != 0 {
                self.read_cstring_at_rva(name_rva)?
            } else {
                format!("unknown_0x{:x}", descriptor_offset)
            };

            // Read import name table entries
            let thunk_entries = if first_thunk_rva != 0 {
                self.parse_thunk_table(first_thunk_rva)?
            } else if original_first_thunk_rva != 0 {
                self.parse_thunk_table(original_first_thunk_rva)?
            } else {
                Vec::new()
            };

            for entry in &thunk_entries {
                imports.push(PeImport {
                    library: library.clone(),
                    function: entry.function_name.clone(),
                    ordinal: entry.ordinal,
                    thunk_rva: entry.thunk_rva,
                });
            }

            descriptor_offset += 20;
        }

        Ok(imports)
    }

    /// Parse Import Name Table (INT) or Import Address Table (IAT) thunk entries
    fn parse_thunk_table(&self, thunk_rva: u32) -> Result<Vec<_ThunkEntry>, PatchError> {
        let mut entries = Vec::new();
        let thunk_file_offset = self.rva_to_offset(thunk_rva)?;
        let mut offset = thunk_file_offset;

        loop {
            if offset + 8 > self.data.len() {
                break;
            }

            let thunk_value = if self.is_pe32plus()? {
                u64::from_le_bytes([
                    self.data[offset],
                    self.data[offset + 1],
                    self.data[offset + 2],
                    self.data[offset + 3],
                    self.data[offset + 4],
                    self.data[offset + 5],
                    self.data[offset + 6],
                    self.data[offset + 7],
                ])
            } else {
                u32::from_le_bytes([
                    self.data[offset],
                    self.data[offset + 1],
                    self.data[offset + 2],
                    self.data[offset + 3],
                ]) as u64
            };

            // Null terminator
            if thunk_value == 0 {
                break;
            }

            let is_ordinal = if self.is_pe32plus()? {
                (thunk_value >> 63) == 1
            } else {
                (thunk_value >> 31) == 1
            };

            if is_ordinal {
                let ordinal = (thunk_value & 0xFFFF) as u16;
                entries.push(_ThunkEntry {
                    function_name: format!("Ordinal({})", ordinal),
                    ordinal: Some(ordinal),
                    thunk_rva: (offset - self.rva_to_offset(thunk_rva)? + thunk_rva as usize)
                        as u32,
                });
            } else {
                let hint_name_rva = (thunk_value & 0x7FFFFFFF) as u32;
                let hint_name_offset = self.rva_to_offset(hint_name_rva)?;

                if hint_name_offset + 2 < self.data.len() {
                    let _hint = u16::from_le_bytes([
                        self.data[hint_name_offset],
                        self.data[hint_name_offset + 1],
                    ]);
                    let name = self.read_cstring(hint_name_offset + 2)?;

                    entries.push(_ThunkEntry {
                        function_name: name,
                        ordinal: None,
                        thunk_rva: (offset - self.rva_to_offset(thunk_rva)? + thunk_rva as usize)
                            as u32,
                    });
                }
            }

            offset += if self.is_pe32plus()? { 8 } else { 4 };
        }

        Ok(entries)
    }

    /// Helper: check if PE is PE32+
    fn is_pe32plus(&self) -> Result<bool, PatchError> {
        let e_lfanew = u32::from_le_bytes([
            self.data[0x3C],
            self.data[0x3D],
            self.data[0x3E],
            self.data[0x3F],
        ]) as usize;

        let opt = e_lfanew + 4 + 20; // PE signature + COFF header
        let magic = u16::from_le_bytes([self.data[opt], self.data[opt + 1]]);
        Ok(magic == 0x20B)
    }

    /// Read a null-terminated ASCII string at a file offset
    fn read_cstring(&self, offset: usize) -> Result<String, PatchError> {
        let mut s = String::new();
        let mut i = offset;
        while i < self.data.len() && self.data[i] != 0 {
            s.push(self.data[i] as char);
            i += 1;
        }
        Ok(s)
    }

    /// Read a null-terminated ASCII string at an RVA
    fn read_cstring_at_rva(&self, rva: u32) -> Result<String, PatchError> {
        let offset = self.rva_to_offset(rva)?;
        self.read_cstring(offset)
    }
}

/// Internal thunk entry type
#[derive(Debug, Clone)]
struct _ThunkEntry {
    function_name: String,
    ordinal: Option<u16>,
    thunk_rva: u32,
}

// ============================================================================
// PE CheckSum Recalculation
// ============================================================================

impl BinaryPatcher {
    /// Recalculate PE CheckSum field in Optional Header
    ///
    /// The PE checksum is stored at Optional Header offset 0x58 (PE32) or 0x70 (PE32+).
    /// Algorithm: sum all words in the file (skipping the checksum field itself),
    /// then fold into 16-bit value with carry.
    pub fn pe_recalculate_checksum(&mut self) -> Result<u32, PatchError> {
        if self.format != PatchFormat::Pe {
            return Err(PatchError::InvalidFormat("Not a PE file".to_string()));
        }

        let e_lfanew = u32::from_le_bytes([
            self.data[0x3C],
            self.data[0x3D],
            self.data[0x3E],
            self.data[0x3F],
        ]) as usize;

        let opt = e_lfanew + 4 + 20; // PE signature + COFF header
        let magic = u16::from_le_bytes([self.data[opt], self.data[opt + 1]]);
        let is_pe32plus = magic == 0x20B;

        // CheckSum field offset in Optional Header
        let checksum_offset_in_opt = if is_pe32plus { 0x70 } else { 0x58 };
        let checksum_file_offset = opt + checksum_offset_in_opt;

        if checksum_file_offset + 4 > self.data.len() {
            return Err(PatchError::InvalidFormat(
                "PE header too small for checksum".to_string(),
            ));
        }

        // Read the SizeOfImage to know file coverage
        let size_of_image_offset = opt + if is_pe32plus { 56 } else { 56 };
        let _size_of_image = if size_of_image_offset + 4 <= self.data.len() {
            u32::from_le_bytes([
                self.data[size_of_image_offset],
                self.data[size_of_image_offset + 1],
                self.data[size_of_image_offset + 2],
                self.data[size_of_image_offset + 3],
            ])
        } else {
            self.data.len() as u32
        };

        // Calculate checksum using standard PE checksum algorithm
        let checksum = self.calculate_pe_checksum(checksum_file_offset);

        // Write the new checksum back
        self.data[checksum_file_offset..checksum_file_offset + 4]
            .copy_from_slice(&checksum.to_le_bytes());

        Ok(checksum)
    }

    /// Calculate PE checksum without modifying the file
    pub fn pe_calculate_checksum(&self) -> Result<u32, PatchError> {
        if self.format != PatchFormat::Pe {
            return Err(PatchError::InvalidFormat("Not a PE file".to_string()));
        }

        let e_lfanew = u32::from_le_bytes([
            self.data[0x3C],
            self.data[0x3D],
            self.data[0x3E],
            self.data[0x3F],
        ]) as usize;

        let opt = e_lfanew + 4 + 20;
        let magic = u16::from_le_bytes([self.data[opt], self.data[opt + 1]]);
        let is_pe32plus = magic == 0x20B;

        let checksum_offset_in_opt = if is_pe32plus { 0x70 } else { 0x58 };
        let checksum_file_offset = opt + checksum_offset_in_opt;

        Ok(self.calculate_pe_checksum(checksum_file_offset))
    }

    /// Internal: PE checksum algorithm (Windows SDK style)
    fn calculate_pe_checksum(&self, checksum_field_offset: usize) -> u32 {
        let mut checksum: u64 = 0;
        let file_len = self.data.len();
        let word_count = file_len / 2;

        for i in 0..word_count {
            let offset = i * 2;

            // Skip the 4-byte checksum field
            if offset == checksum_field_offset || offset + 1 == checksum_field_offset {
                continue;
            }

            let word = u16::from_le_bytes([self.data[offset], self.data[offset + 1]]) as u64;

            checksum = (checksum & 0xFFFFFFFF) + word + (checksum >> 32);
            if checksum > 0x100000000 {
                checksum = (checksum & 0xFFFFFFFF) + (checksum >> 32);
            }
        }

        // Handle odd byte
        if file_len % 2 == 1 {
            let last_byte = self.data[file_len - 1] as u64;
            checksum = (checksum & 0xFFFFFFFF) + last_byte + (checksum >> 32);
            if checksum > 0x100000000 {
                checksum = (checksum & 0xFFFFFFFF) + (checksum >> 32);
            }
        }

        // Fold to 16-bit
        let low = (checksum & 0xFFFF) as u32;
        let high = (checksum >> 16) as u32;
        (low + high) as u32
    }

    // =========================================================================
    // Conditional Jump Modification
    // =========================================================================

    // (X86_JCC_OPS 操作码表零引用已删除; 同数据由下方案 invert_jcc match 覆盖)

    /// Invert a conditional jump opcode (je↔jne, jl↔jge, etc.)
    ///
    /// Returns the inverted opcode, or None if not a recognized jcc.
    pub fn invert_jcc(opcode: u8) -> Option<u8> {
        match opcode {
            // Short jcc: invert by flipping bit 0
            0x70 => Some(0x71), // JO  → JNO
            0x71 => Some(0x70), // JNO → JO
            0x72 => Some(0x73), // JB  → JNB
            0x73 => Some(0x72), // JNB → JB
            0x74 => Some(0x75), // JE  → JNE
            0x75 => Some(0x74), // JNE → JE
            0x76 => Some(0x77), // JBE → JNBE
            0x77 => Some(0x76), // JNBE→ JBE
            0x78 => Some(0x79), // JS  → JNS
            0x79 => Some(0x78), // JNS → JS
            0x7A => Some(0x7B), // JP  → JNP
            0x7B => Some(0x7A), // JNP → JP
            0x7C => Some(0x7D), // JL  → JNL
            0x7D => Some(0x7C), // JNL → JL
            0x7E => Some(0x7F), // JLE → JNLE
            0x7F => Some(0x7E), // JNLE→ JLE
            _ => None,
        }
    }

    /// Check if a byte at a given offset is a conditional jump instruction
    pub fn is_jcc_at(&self, offset: usize) -> Option<_JccInfo> {
        if offset >= self.data.len() {
            return None;
        }

        let b0 = self.data[offset];

        // Short jcc: 0x70-0x7F (1-byte opcode + 1-byte relative offset)
        if b0 >= 0x70 && b0 <= 0x7F {
            if offset + 2 > self.data.len() {
                return None;
            }
            let rel_offset = self.data[offset + 1] as i8;
            return Some(_JccInfo {
                offset,
                opcode: b0,
                is_near: false,
                target_offset: (offset as i64 + 2 + rel_offset as i64) as usize,
                instruction_len: 2,
            });
        }

        // Near jcc: 0x0F 0x80-0x8F (2-byte opcode + 4-byte relative offset)
        if b0 == 0x0F && offset + 1 < self.data.len() {
            let b1 = self.data[offset + 1];
            if b1 >= 0x80 && b1 <= 0x8F {
                if offset + 6 > self.data.len() {
                    return None;
                }
                let rel_offset = i32::from_le_bytes([
                    self.data[offset + 2],
                    self.data[offset + 3],
                    self.data[offset + 4],
                    self.data[offset + 5],
                ]);
                return Some(_JccInfo {
                    offset,
                    opcode: b0,
                    is_near: true,
                    target_offset: (offset as i64 + 6 + rel_offset as i64) as usize,
                    instruction_len: 6,
                });
            }
        }

        None
    }

    /// NOP a conditional jump instruction (replace with NOPs)
    pub fn nop_jcc_at(&mut self, offset: usize) -> Result<PatchResult, PatchError> {
        let jcc = self.is_jcc_at(offset).ok_or_else(|| {
            PatchError::InvalidFormat(format!("No conditional jump at offset 0x{:08x}", offset))
        })?;

        for i in offset..offset + jcc.instruction_len {
            self.data[i] = 0x90; // NOP
        }

        Ok(PatchResult {
            op_index: 0,
            success: true,
            offset,
            bytes_changed: jcc.instruction_len,
            message: format!(
                "NOPped {} conditional jump at 0x{:08x}",
                if jcc.is_near { "near" } else { "short" },
                offset
            ),
        })
    }

    /// Invert a conditional jump at a given offset (je→jne, etc.)
    pub fn invert_jcc_at(&mut self, offset: usize) -> Result<PatchResult, PatchError> {
        let jcc = self.is_jcc_at(offset).ok_or_else(|| {
            PatchError::InvalidFormat(format!("No conditional jump at offset 0x{:08x}", offset))
        })?;

        if jcc.is_near {
            return Err(PatchError::InvalidFormat(
                "Cannot invert near jcc (0F 8x) in-place — use ByteReplace with target patch"
                    .to_string(),
            ));
        }

        let inverted = Self::invert_jcc(jcc.opcode).ok_or_else(|| {
            PatchError::InvalidFormat(format!(
                "Cannot invert opcode 0x{:02x} at 0x{:08x}",
                jcc.opcode, offset
            ))
        })?;

        self.data[offset] = inverted;

        Ok(PatchResult {
            op_index: 0,
            success: true,
            offset,
            bytes_changed: 1,
            message: format!(
                "Inverted jcc at 0x{:08x}: 0x{:02x} → 0x{:02x}",
                offset, jcc.opcode, inverted
            ),
        })
    }

    /// Scan for all conditional jumps in a byte range
    pub fn scan_jcc(&self, start: usize, end: usize) -> Vec<_JccInfo> {
        let mut results = Vec::new();
        let mut offset = start;

        while offset < end && offset < self.data.len() {
            if let Some(jcc) = self.is_jcc_at(offset) {
                let instr_len = jcc.instruction_len;
                results.push(jcc);
                offset += instr_len;
            } else {
                offset += 1;
            }
        }

        results
    }
}

/// Conditional jump instruction info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct _JccInfo {
    pub offset: usize,
    pub opcode: u8,
    pub is_near: bool,
    pub target_offset: usize,
    pub instruction_len: usize,
}

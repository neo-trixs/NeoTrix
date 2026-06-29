#![allow(dead_code)]

use flate2::read::ZlibDecoder;
use std::collections::HashMap;
use std::io::Read;

// ---------------------------------------------------------------------------
// ZIP container reader (no external zip crate)
// ---------------------------------------------------------------------------

/// Entry found in the ZIP central directory.
#[derive(Debug, Clone)]
pub struct ZipEntry {
    pub name: String,
    pub compression_method: u16,
    pub compressed_size: u32,
    pub uncompressed_size: u32,
    pub local_header_offset: u32,
    pub crc32: u32,
}

/// Parse the ZIP End-of-Central-Directory record.
/// Scans backward from end of file for the EOCD signature (max comment 64KB).
pub fn find_eocd(data: &[u8]) -> Option<usize> {
    let len = data.len();
    if len < 22 {
        return None;
    }
    let search_start = if len > 65557 { len - 65557 } else { 0 };
    let sig = [0x50, 0x4b, 0x05, 0x06];
    for i in (search_start..len - 21).rev() {
        if data[i..i + 4] == sig {
            return Some(i);
        }
    }
    None
}

pub fn read_u16_le(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

pub fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// List all entries in a ZIP archive by traversing the central directory.
pub fn list_zip_entries(data: &[u8]) -> Result<Vec<ZipEntry>, String> {
    let eocd_offset = find_eocd(data).ok_or_else(|| "EOCD not found".to_string())?;
    let cd_offset = read_u32_le(data, eocd_offset + 16) as usize;
    let cd_size = read_u32_le(data, eocd_offset + 12) as usize;
    let total_entries = read_u16_le(data, eocd_offset + 10) as usize;

    let cd_end = cd_offset + cd_size;
    if cd_end > data.len() {
        return Err("Central directory exceeds file size".to_string());
    }

    let mut entries = Vec::with_capacity(total_entries);
    let mut pos = cd_offset;
    for _ in 0..total_entries {
        if pos + 46 > cd_end {
            break;
        }
        if data[pos..pos + 4] != [0x50, 0x4b, 0x01, 0x02] {
            return Err(format!("Invalid central directory entry at offset {pos}"));
        }
        let compression = read_u16_le(data, pos + 10);
        let compressed_size = read_u32_le(data, pos + 20);
        let uncompressed_size = read_u32_le(data, pos + 24);
        let name_len = read_u16_le(data, pos + 28) as usize;
        let extra_len = read_u16_le(data, pos + 30) as usize;
        let comment_len = read_u16_le(data, pos + 32) as usize;
        let local_offset = read_u32_le(data, pos + 42);
        let name_start = pos + 46;
        let name_bytes = &data[name_start..name_start + name_len];
        let name = String::from_utf8_lossy(name_bytes).to_string();
        entries.push(ZipEntry {
            name,
            compression_method: compression,
            compressed_size,
            uncompressed_size,
            local_header_offset: local_offset,
            crc32: read_u32_le(data, pos + 16),
        });
        pos += 46 + name_len + extra_len + comment_len;
    }
    Ok(entries)
}

/// Read and decompress a single entry from the ZIP archive.
pub fn read_zip_entry(data: &[u8], entry: &ZipEntry) -> Result<Vec<u8>, String> {
    let lh_offset = entry.local_header_offset as usize;
    if lh_offset + 30 > data.len() {
        return Err("Local header offset out of range".to_string());
    }
    if data[lh_offset..lh_offset + 4] != [0x50, 0x4b, 0x03, 0x04] {
        return Err(format!("Invalid local header at offset {lh_offset}"));
    }
    let name_len = read_u16_le(data, lh_offset + 26) as usize;
    let extra_len = read_u16_le(data, lh_offset + 28) as usize;
    let data_start = lh_offset + 30 + name_len + extra_len;
    let raw = &data[data_start..data_start + entry.compressed_size as usize];

    match entry.compression_method {
        0 => Ok(raw.to_vec()),
        8 => {
            let mut decoder = ZlibDecoder::new(raw);
            let mut buf = Vec::with_capacity(entry.uncompressed_size as usize);
            decoder
                .read_to_end(&mut buf)
                .map_err(|e| format!("DEFLATE decompress failed: {e}"))?;
            Ok(buf)
        }
        other => Err(format!("Unsupported ZIP compression method: {other}")),
    }
}

/// Convenience: find a ZIP entry by name and read its content.
pub fn read_zip_named(data: &[u8], name: &str) -> Result<Vec<u8>, String> {
    let entries = list_zip_entries(data)?;
    let entry = entries
        .iter()
        .find(|e| e.name == name || e.name == format!("/{name}"))
        .ok_or_else(|| format!("ZIP entry not found: {name}"))?;
    read_zip_entry(data, entry)
}

// ---------------------------------------------------------------------------
// Simple XML scanner (no quick-xml dependency)
// ---------------------------------------------------------------------------

/// Extract text content from the first occurrence of `<tag>...</tag>`.
/// Handles simple XML: finds the opening tag, then scans for `</tag>`.
pub fn extract_xml_tag_text(xml: &[u8], tag: &str) -> Option<String> {
    let s = String::from_utf8_lossy(xml);
    let tag_open = format!("<{}", tag);
    let tag_close = format!("</{}>", tag);

    let start = s.find(&tag_open)?;
    let content_start = s[start..].find('>')? + start + 1;
    let end = s[content_start..].find(&tag_close)? + content_start;
    let raw = &s[content_start..end];
    Some(xml_unescape(raw))
}

/// Extract all occurrences of `<tag>...</tag>` content.
pub fn extract_all_xml_tags(xml: &[u8], tag: &str) -> Vec<String> {
    let s = String::from_utf8_lossy(xml);
    let tag_open = format!("<{}", tag);
    let tag_close = format!("</{}>", tag);
    let mut results = Vec::new();
    let mut pos = 0;
    while let Some(start) = s[pos..].find(&tag_open) {
        let abs_start = pos + start;
        let content_start = match s[abs_start..].find('>') {
            Some(o) => abs_start + o + 1,
            None => break,
        };
        let remaining = &s[content_start..];
        let end = match remaining.find(&tag_close) {
            Some(o) => o + content_start,
            None => break,
        };
        results.push(xml_unescape(&s[content_start..end]));
        pos = end + tag_close.len();
    }
    results
}

/// Extract text from `<t>...</t>` tags (shared string items in xlsx).
pub fn extract_shared_strings(xml: &[u8]) -> Vec<String> {
    let s = String::from_utf8_lossy(xml);
    let mut results = Vec::new();
    let mut pos = 0;
    // Each <si> block contains text in <t>...</t>, possibly with formatting
    while let Some(si_start) = s[pos..].find("<si>") {
        let abs_si = pos + si_start;
        let si_end = match s[abs_si..].find("</si>") {
            Some(o) => abs_si + o + 5,
            None => break,
        };
        let si_block = &s[abs_si..si_end];
        // Try <t> first, fall back to raw content
        if let Some(t) = extract_xml_tag_text(si_block.as_bytes(), "t") {
            results.push(xml_unescape(&t));
        } else {
            // Some xlsx use <r> (rich text) runs
            let runs: Vec<String> = extract_all_xml_tags(si_block.as_bytes(), "t");
            let combined = runs.join("");
            if !combined.is_empty() {
                results.push(xml_unescape(&combined));
            } else {
                results.push(String::new());
            }
        }
        pos = si_end;
    }
    results
}

/// Extract sheet names and their index IDs from `<sheet>` elements.
/// Returns Vec of (r:id, sheet_name).
pub fn extract_sheet_names(workbook_xml: &[u8]) -> Vec<(String, String)> {
    let s = String::from_utf8_lossy(workbook_xml);
    let mut sheets = Vec::new();
    let mut pos = 0;
    while let Some(sh_start) = s[pos..].find("<sheet") {
        let abs_start = pos + sh_start;
        let sh_end = s[abs_start..]
            .find("/>")
            .or_else(|| s[abs_start..].find(">"))
            .map(|o| abs_start + o + 2);
        let sh_end = match sh_end {
            Some(o) => o,
            None => break,
        };
        let block = &s[abs_start..sh_end];
        // Extract sheetId and name attributes
        let name = extract_attr(block, "name");
        let id = extract_attr(block, "r:id").or_else(|| extract_attr(block, "id"));
        if let (Some(n), Some(i)) = (name, id) {
            sheets.push((i, n));
        }
        pos = sh_end;
    }
    sheets
}

/// Extract relationship target from `.rels` XML by Id.
pub fn extract_rels_target(rels_xml: &[u8], rel_id: &str) -> Option<String> {
    let s = String::from_utf8_lossy(rels_xml);
    let target_str = format!("Id=\"{rel_id}\"");
    let pos = s.find(&target_str)?;
    let after = &s[pos + target_str.len()..];
    let target = extract_attr_from_segment(after, "Target")?;
    Some(target)
}

/// Extract relationship map from `.rels` XML: Id → Target.
pub fn extract_rels_map(rels_xml: &[u8]) -> HashMap<String, String> {
    let s = String::from_utf8_lossy(rels_xml);
    let mut map = HashMap::new();
    let mut pos = 0;
    while let Some(rel_start) = s[pos..].find("<Relationship") {
        let abs_start = pos + rel_start;
        let rel_end = match s[abs_start..].find("/>") {
            Some(o) => abs_start + o + 2,
            None => break,
        };
        let block = &s[abs_start..rel_end];
        let id = extract_attr(block, "Id");
        let target = extract_attr(block, "Target");
        if let (Some(i), Some(t)) = (id, target) {
            map.insert(i, t);
        }
        pos = rel_end;
    }
    map
}

/// Extract an XML attribute value by name from a tag string.
pub fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    let search = format!("{attr}=\"");
    let start = tag.find(&search)?;
    let value_start = start + search.len();
    let end = tag[value_start..].find('"')? + value_start;
    Some(tag[value_start..end].to_string())
}

fn extract_attr_from_segment(segment: &str, attr: &str) -> Option<String> {
    let search = format!("{attr}=\"");
    let start = segment.find(&search)?;
    let value_start = start + search.len();
    let end = segment[value_start..].find('"')? + value_start;
    Some(segment[value_start..end].to_string())
}

/// Unescape XML entities in a string.
pub fn xml_unescape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '&' {
            let mut entity = String::new();
            for ec in chars.by_ref() {
                if ec == ';' {
                    break;
                }
                entity.push(ec);
            }
            match entity.as_str() {
                "amp" => result.push('&'),
                "lt" => result.push('<'),
                "gt" => result.push('>'),
                "quot" => result.push('"'),
                "apos" => result.push('\''),
                _ => {
                    result.push('&');
                    result.push_str(&entity);
                    result.push(';');
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// Parse xlsx cell reference like "A1" into (column_index, row_index).
pub fn parse_cell_ref(ref_str: &str) -> Option<(usize, usize)> {
    let col_end = ref_str.find(|c: char| c.is_ascii_digit())?;
    let col_str = &ref_str[..col_end];
    let row_str = &ref_str[col_end..];
    let mut col = 0usize;
    for c in col_str.chars() {
        col = col * 26 + (c as usize - 'A' as usize + 1);
    }
    let row: usize = row_str.parse().ok()?;
    Some((col.saturating_sub(1), row.saturating_sub(1)))
}

// ---------------------------------------------------------------------------
// OLE2 Compound Document reader (for .xls files)
// ---------------------------------------------------------------------------

const OLE2_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
const FREESECT: u32 = 0xFFFFFFFF;
const ENDOFCHAIN: u32 = 0xFFFFFFFE;
const FATSECT: u32 = 0xFFFFFFFD;
const DIFSECT: u32 = 0xFFFFFFFC;

/// Parsed OLE2 compound document header.
#[derive(Debug, Clone)]
pub struct Ole2Header {
    pub minor_version: u16,
    pub major_version: u16,
    pub sector_size: usize,
    pub mini_sector_size: usize,
    pub total_sat_sectors: u32,
    pub first_fat_secid: u32,
    pub mini_cutoff: u32,
    pub first_minifat_secid: u32,
    pub total_minifat_sectors: u32,
    pub first_difat_secid: u32,
    pub total_difat_sectors: u32,
    /// First 109 DIFAT entries embedded in header
    pub difat_entries: Vec<u32>,
}

#[derive(Debug, Clone)]
pub struct Ole2DirectoryEntry {
    pub name: String,
    pub entry_type: u8,
    pub left_sibling: u32,
    pub right_sibling: u32,
    pub child: u32,
    pub starting_sector: u32,
    pub stream_size: u64,
}

/// Read-only OLE2 compound document reader.
#[derive(Debug, Clone)]
pub struct Ole2Reader {
    pub header: Ole2Header,
    fat: Vec<u32>,
    minifat: Vec<u32>,
    pub dir_entries: Vec<Ole2DirectoryEntry>,
    pub raw_data: Vec<u8>,
}

impl Ole2Reader {
    pub fn new(data: &[u8]) -> Result<Self, String> {
        if data.len() < 512 || data[..8] != OLE2_MAGIC {
            return Err("Not a valid OLE2 file".to_string());
        }

        let minor_version = u16::from_le_bytes([data[24], data[25]]);
        let major_version = u16::from_le_bytes([data[26], data[27]]);
        let byte_order = u16::from_le_bytes([data[28], data[29]]);
        if byte_order != 0xFFFE {
            return Err("Unsupported byte order".to_string());
        }
        let sector_size_power = data[30];
        let mini_sector_size_power = data[31];
        let sector_size = 1 << sector_size_power;
        let mini_sector_size = 1 << mini_sector_size_power;
        let total_sat_sectors = u32::from_le_bytes([data[44], data[45], data[46], data[47]]);
        let first_fat_secid = u32::from_le_bytes([data[48], data[49], data[50], data[51]]);
        let mini_cutoff = u32::from_le_bytes([data[56], data[57], data[58], data[59]]);
        let first_minifat_secid = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
        let total_minifat_sectors = u32::from_le_bytes([data[64], data[65], data[66], data[67]]);
        let first_difat_secid = u32::from_le_bytes([data[68], data[69], data[70], data[71]]);
        let total_difat_sectors = u32::from_le_bytes([data[72], data[73], data[74], data[75]]);

        // Extract embedded DIFAT entries (109 entries starting at offset 76)
        let mut difat_entries = Vec::with_capacity(109);
        for i in 0..109 {
            let off = 76 + i * 4;
            let entry =
                u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
            difat_entries.push(entry);
        }

        let header = Ole2Header {
            minor_version,
            major_version,
            sector_size,
            mini_sector_size,
            total_sat_sectors,
            first_fat_secid,
            mini_cutoff,
            first_minifat_secid,
            total_minifat_sectors,
            first_difat_secid,
            total_difat_sectors,
            difat_entries,
        };

        // Build FAT from DIFAT chain
        let fat = Self::build_fat(data, &header)?;
        let minifat = Self::build_minifat(data, &header, &fat)?;

        // Parse directory entries
        // Directory is in the first directory sector (root entry's child chain)
        // The root entry's starting sector points to the directory stream
        let dir_entries = Self::parse_directory(data, &header, &fat, &minifat)?;

        Ok(Ole2Reader {
            header,
            fat,
            minifat,
            dir_entries,
            raw_data: data.to_vec(),
        })
    }

    fn sector_offset(sector: u32, sector_size: usize) -> usize {
        512 + sector as usize * sector_size
    }

    fn read_sector(data: &[u8], sector: u32, sector_size: usize) -> Result<Vec<u8>, String> {
        let off = Self::sector_offset(sector, sector_size);
        let end = off + sector_size;
        if end > data.len() {
            return Err(format!("Sector {sector} out of bounds"));
        }
        Ok(data[off..end].to_vec())
    }

    fn build_fat(data: &[u8], header: &Ole2Header) -> Result<Vec<u32>, String> {
        let sector_size = header.sector_size;
        let entries_per_sector = sector_size / 4;
        let mut fat_entries = Vec::new();

        // Walk DIFAT chain
        let mut difat_secids: Vec<u32> = Vec::new();
        // First 109 entries from header
        for &entry in &header.difat_entries {
            if entry == FREESECT {
                break;
            }
            difat_secids.push(entry);
        }
        // Additional DIFAT sectors
        let mut next_difat = header.first_difat_secid;
        for _ in 0..header.total_difat_sectors {
            if next_difat == ENDOFCHAIN || next_difat == FREESECT {
                break;
            }
            let sector_end = Self::sector_offset(next_difat, sector_size) + sector_size;
            if sector_end > data.len() {
                break;
            }
            let sec_data = Self::read_sector(data, next_difat, sector_size)?;
            // Last entry in DIFAT sector points to next DIFAT sector
            let count = entries_per_sector.saturating_sub(1);
            for i in 0..count {
                let off = i * 4;
                if off + 4 > sec_data.len() {
                    break;
                }
                let entry = u32::from_le_bytes([
                    sec_data[off],
                    sec_data[off + 1],
                    sec_data[off + 2],
                    sec_data[off + 3],
                ]);
                if entry == FREESECT {
                    break;
                }
                difat_secids.push(entry);
            }
            // Last entry is the next DIFAT sector
            let last_off = (entries_per_sector - 1) * 4;
            if last_off + 4 <= sec_data.len() {
                next_difat = u32::from_le_bytes([
                    sec_data[last_off],
                    sec_data[last_off + 1],
                    sec_data[last_off + 2],
                    sec_data[last_off + 3],
                ]);
            } else {
                break;
            }
        }

        // Read each FAT sector
        for &fat_secid in &difat_secids {
            let sec_data = Self::read_sector(data, fat_secid, sector_size)?;
            for i in 0..entries_per_sector {
                let off = i * 4;
                if off + 4 > sec_data.len() {
                    break;
                }
                let entry = u32::from_le_bytes([
                    sec_data[off],
                    sec_data[off + 1],
                    sec_data[off + 2],
                    sec_data[off + 3],
                ]);
                fat_entries.push(entry);
            }
        }

        Ok(fat_entries)
    }

    fn build_minifat(data: &[u8], header: &Ole2Header, fat: &[u32]) -> Result<Vec<u32>, String> {
        if header.first_minifat_secid == ENDOFCHAIN || header.first_minifat_secid == FREESECT {
            return Ok(Vec::new());
        }
        let sector_size = header.sector_size;
        let entries_per_sector = sector_size / 4;
        let mut minifat = Vec::new();
        let mut secid = header.first_minifat_secid;
        loop {
            if secid == ENDOFCHAIN || secid == FREESECT {
                break;
            }
            let sec_data = Self::read_sector(data, secid, sector_size)?;
            for i in 0..entries_per_sector {
                let off = i * 4;
                if off + 4 > sec_data.len() {
                    break;
                }
                minifat.push(u32::from_le_bytes([
                    sec_data[off],
                    sec_data[off + 1],
                    sec_data[off + 2],
                    sec_data[off + 3],
                ]));
            }
            let idx = secid as usize;
            secid = if idx < fat.len() {
                fat[idx]
            } else {
                ENDOFCHAIN
            };
        }
        Ok(minifat)
    }

    fn parse_directory(
        data: &[u8],
        header: &Ole2Header,
        fat: &[u32],
        _minifat: &[u32],
    ) -> Result<Vec<Ole2DirectoryEntry>, String> {
        // Directory stream starts at root entry's starting sector
        // Root entry is always the first directory entry
        // We need to read the directory stream first
        // The directory is stored as a stream, starting at sector 0 (first directory sector)
        // Actually, the root entry's starting sector tells us where the directory stream is

        // Read the first 512 bytes of directory stream (one regular sector size)
        let sector_size = header.sector_size;
        let _dir_start = 512; // directory stream starts at first sector after header
        let dir_size = 128 * 512 / 128; // enough for 512 directory entries = 64KB

        let mut dir_data = Vec::new();
        let mut secid = 0u32; // directory usually starts at sector 0
                              // Actually, let's read sectors via FAT starting from the root's starting sector
                              // For now, read from the first sector after header
        for i in 0..(dir_size / sector_size).min(4) {
            // read up to 4 sectors
            if secid == ENDOFCHAIN || secid == FREESECT {
                break;
            }
            let off = Self::sector_offset(if secid == 0 { i as u32 } else { secid }, sector_size);
            let end = (off + sector_size).min(data.len());
            if off < end {
                dir_data.extend_from_slice(&data[off..end]);
            }
            let idx = secid as usize;
            secid = if idx < fat.len() {
                fat[idx]
            } else {
                ENDOFCHAIN
            };
            if secid == ENDOFCHAIN {
                break;
            }
            if i == 0 {
                // After first sector, follow FAT chain from sector 0's chain
                // The first 4 bytes of FAT at index 0 tell us the next directory sector
                secid = if !fat.is_empty() { fat[0] } else { ENDOFCHAIN };
            }
        }

        let mut entries = Vec::new();
        let mut offset = 0;
        while offset + 128 <= dir_data.len() {
            let name_size =
                u16::from_le_bytes([dir_data[offset + 64], dir_data[offset + 65]]) as usize;
            let name_bytes = &dir_data[offset..offset + name_size.min(64)];
            let name_utf16: Vec<u16> = name_bytes
                .chunks(2)
                .filter(|c| c.len() == 2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let name = String::from_utf16_lossy(&name_utf16)
                .trim_end_matches('\0')
                .to_string();
            let entry_type = dir_data[offset + 66];
            let left = u32::from_le_bytes([
                dir_data[offset + 68],
                dir_data[offset + 69],
                dir_data[offset + 70],
                dir_data[offset + 71],
            ]);
            let right = u32::from_le_bytes([
                dir_data[offset + 72],
                dir_data[offset + 73],
                dir_data[offset + 74],
                dir_data[offset + 75],
            ]);
            let child = u32::from_le_bytes([
                dir_data[offset + 76],
                dir_data[offset + 77],
                dir_data[offset + 78],
                dir_data[offset + 79],
            ]);
            let starting = u32::from_le_bytes([
                dir_data[offset + 116],
                dir_data[offset + 117],
                dir_data[offset + 118],
                dir_data[offset + 119],
            ]);
            let stream_size = u64::from_le_bytes([
                dir_data[offset + 120],
                dir_data[offset + 121],
                dir_data[offset + 122],
                dir_data[offset + 123],
                dir_data[offset + 124],
                dir_data[offset + 125],
                dir_data[offset + 126],
                dir_data[offset + 127],
            ]);

            entries.push(Ole2DirectoryEntry {
                name,
                entry_type,
                left_sibling: left,
                right_sibling: right,
                child,
                starting_sector: starting,
                stream_size,
            });
            offset += 128;
        }

        Ok(entries)
    }

    /// Read a stream by following the FAT chain starting from the given sector.
    pub fn read_stream(&self, starting_sector: u32, stream_size: u64) -> Result<Vec<u8>, String> {
        if stream_size == 0 {
            return Ok(Vec::new());
        }
        let sector_size = self.header.sector_size;
        let is_mini = stream_size < self.header.mini_cutoff as u64;

        if is_mini {
            let mini_sector_size = self.header.mini_sector_size;
            let mut result = Vec::with_capacity(stream_size as usize);
            // Mini stream: stored in the root entry's stream
            // Find root entry (first entry, type 5)
            let root_start = self
                .dir_entries
                .first()
                .filter(|e| e.entry_type == 5)
                .map(|e| e.starting_sector)
                .unwrap_or(starting_sector);
            let root_stream = self.read_stream_from_fat(root_start, sector_size as u64)?;
            let mut secid = starting_sector;
            loop {
                if secid == ENDOFCHAIN || secid == FREESECT {
                    break;
                }
                let off = secid as usize * mini_sector_size;
                if off + mini_sector_size <= root_stream.len() {
                    let end = (off + mini_sector_size)
                        .min(root_stream.len())
                        .min(result.len() + stream_size as usize - result.len());
                    result.extend_from_slice(&root_stream[off..end]);
                    if result.len() >= stream_size as usize {
                        break;
                    }
                }
                let idx = secid as usize;
                secid = if idx < self.minifat.len() {
                    self.minifat[idx]
                } else {
                    ENDOFCHAIN
                };
            }
            result.truncate(stream_size as usize);
            Ok(result)
        } else {
            self.read_stream_from_fat(starting_sector, stream_size)
        }
    }

    fn read_stream_from_fat(
        &self,
        starting_sector: u32,
        stream_size: u64,
    ) -> Result<Vec<u8>, String> {
        let sector_size = self.header.sector_size;
        let mut result = Vec::with_capacity(stream_size as usize);
        let mut secid = starting_sector;
        loop {
            if secid == ENDOFCHAIN || secid == FREESECT {
                break;
            }
            let off = Self::sector_offset(secid, sector_size);
            if off >= self.raw_data.len() {
                break;
            }
            let end = (off + sector_size)
                .min(self.raw_data.len())
                .min(off + (stream_size as usize).saturating_sub(result.len()));
            result.extend_from_slice(&self.raw_data[off..end]);
            if result.len() >= stream_size as usize {
                break;
            }
            let idx = secid as usize;
            secid = if idx < self.fat.len() {
                self.fat[idx]
            } else {
                ENDOFCHAIN
            };
        }
        result.truncate(stream_size as usize);
        Ok(result)
    }

    /// Find a directory entry by name.
    pub fn find_entry(&self, name: &str) -> Option<&Ole2DirectoryEntry> {
        self.dir_entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
    }

    /// Find directory entries recursively under a given parent entry.
    pub fn find_entries_recursive<'a>(
        &'a self,
        parent: &'a Ole2DirectoryEntry,
        entries: &mut Vec<&'a Ole2DirectoryEntry>,
    ) {
        if parent.child != FREESECT && (parent.child as usize) < self.dir_entries.len() {
            let child = &self.dir_entries[parent.child as usize];
            entries.push(child);
            self.find_entries_recursive(child, entries);
        }
    }

    /// List all stream entries matching a prefix.
    pub fn stream_names(&self) -> Vec<String> {
        self.dir_entries
            .iter()
            .filter(|e| e.entry_type == 2) // stream type
            .map(|e| e.name.clone())
            .collect()
    }

    /// Read the Workbook stream from an .xls file.
    pub fn read_workbook(&self) -> Result<Vec<u8>, String> {
        // Try "Workbook" (BIFF8) or "Book" (BIFF5)
        if let Some(entry) = self.find_entry("Workbook") {
            self.read_stream(entry.starting_sector, entry.stream_size)
        } else if let Some(entry) = self.find_entry("Book") {
            self.read_stream(entry.starting_sector, entry.stream_size)
        } else {
            Err("No Workbook stream found".to_string())
        }
    }
}

// ---------------------------------------------------------------------------
// BIFF record parser (for .xls files)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BiffRecord {
    pub record_type: u16,
    pub record_size: u16,
    pub data: Vec<u8>,
}

/// Parse BIFF records from a stream of bytes.
pub fn parse_biff_records(data: &[u8]) -> Vec<BiffRecord> {
    let mut records = Vec::new();
    let mut pos = 0;
    while pos + 4 <= data.len() {
        let record_type = u16::from_le_bytes([data[pos], data[pos + 1]]);
        let record_size = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
        let data_start = pos + 4;
        let data_end = (data_start + record_size).min(data.len());
        let record_data = data[data_start..data_end].to_vec();
        records.push(BiffRecord {
            record_type,
            record_size: record_size as u16,
            data: record_data,
        });
        if record_type == 0x000A {
            break;
        } // EOF
        pos = data_end;
    }
    records
}

/// Extract text from BIFF records. Returns sheet names and cell text.
pub fn extract_text_from_biff(data: &[u8]) -> Vec<String> {
    let records = parse_biff_records(data);
    let mut texts = Vec::new();
    let mut sst: Vec<String> = Vec::new(); // Shared String Table
    let mut sheet_names: Vec<String> = Vec::new();
    let _current_sheet = 0usize;

    // First pass: extract sheet names and SST
    for rec in &records {
        match rec.record_type {
            0x0085 => {
                // SHEET
                if rec.data.len() >= 6 {
                    let name_len = if rec.data[2] == 0 {
                        // Unicode flag in byte 2 bit 2
                        let raw_len = u16::from_le_bytes([rec.data[4], rec.data[5]]) as usize;
                        raw_len
                    } else {
                        let raw_len = rec.data[4] as usize;
                        raw_len
                    };
                    let name_start = if rec.data[2] & 0x04 != 0 { 8 } else { 6 };
                    let name_end = (name_start + name_len).min(rec.data.len());
                    // Handle Unicode strings: if high byte is non-zero, it's UTF-16LE
                    if rec.data[2] & 0x01 != 0 {
                        let u16_chars: Vec<u16> = rec.data
                            [name_start..name_end.min(name_start + name_len * 2)]
                            .chunks(2)
                            .filter(|c| c.len() == 2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect();
                        sheet_names.push(String::from_utf16_lossy(&u16_chars));
                    } else {
                        sheet_names.push(
                            String::from_utf8_lossy(&rec.data[name_start..name_end]).to_string(),
                        );
                    }
                }
            }
            0x00FC => {
                // SST (Shared String Table)
                // Skip first 8 bytes (total strings + unique strings)
                let mut sp = 8usize;
                while sp + 2 <= rec.data.len() {
                    let strlen = u16::from_le_bytes([rec.data[sp], rec.data[sp + 1]]) as usize;
                    sp += 2;
                    if sp + strlen > rec.data.len() {
                        break;
                    }
                    let flags = if sp < rec.data.len() {
                        rec.data[sp - 1]
                    } else {
                        0
                    };
                    if flags & 0x01 != 0 {
                        // UTF-16LE string
                        let u16_chars: Vec<u16> = rec.data[sp..sp + strlen * 2]
                            .chunks(2)
                            .filter(|c| c.len() == 2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect();
                        let s = String::from_utf16_lossy(&u16_chars);
                        sst.push(s);
                        sp += strlen * 2;
                    } else {
                        // Byte string
                        let end = (sp + strlen).min(rec.data.len());
                        sst.push(String::from_utf8_lossy(&rec.data[sp..end]).to_string());
                        sp = end;
                    }
                    // Skip optional formatting runs (if flags & 0x08)
                    if flags & 0x08 != 0 && sp + 4 <= rec.data.len() {
                        let fmt_runs =
                            u16::from_le_bytes([rec.data[sp], rec.data[sp + 1]]) as usize;
                        sp += 2 + fmt_runs * 4;
                    }
                    // Skip extended text (if flags & 0x10)
                    if flags & 0x10 != 0 && sp + 4 <= rec.data.len() {
                        let ext_size = u32::from_le_bytes([
                            rec.data[sp],
                            rec.data[sp + 1],
                            rec.data[sp + 2],
                            rec.data[sp + 3],
                        ]) as usize;
                        sp += 4 + ext_size;
                    }
                }
            }
            _ => {}
        }
    }

    // Second pass: extract cell text
    for rec in &records {
        match rec.record_type {
            0x00C5 => {
                // LABEL (BIFF8 text cell)
                // Row(2) + Col(2) + XF index(2) + text
                if rec.data.len() > 6 {
                    let flags = rec.data[6];
                    let strlen = u16::from_le_bytes([rec.data[7], rec.data[8]]) as usize;
                    let text_start = if flags & 0x01 != 0 { 9 } else { 9 };
                    if flags & 0x01 != 0 {
                        let u16_chars: Vec<u16> = rec.data[text_start..text_start + strlen * 2]
                            .chunks(2)
                            .filter(|c| c.len() == 2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect();
                        let s = String::from_utf16_lossy(&u16_chars);
                        if !s.trim().is_empty() {
                            texts.push(s);
                        }
                    } else {
                        let end = (text_start + strlen).min(rec.data.len());
                        let s = String::from_utf8_lossy(&rec.data[text_start..end]).to_string();
                        if !s.trim().is_empty() {
                            texts.push(s);
                        }
                    }
                }
            }
            0x0204 => {
                // LABEL (BIFF2-5)
                if rec.data.len() > 7 {
                    let strlen = rec.data[6] as usize;
                    let end = (7 + strlen).min(rec.data.len());
                    if end > 7 {
                        let s = String::from_utf8_lossy(&rec.data[7..end]).to_string();
                        if !s.trim().is_empty() {
                            texts.push(s);
                        }
                    }
                }
            }
            0x0207 => {
                // STRING (result of FORMULA)
                if rec.data.len() > 3 {
                    let flags = rec.data[0];
                    let strlen = if flags & 0x01 != 0 {
                        u16::from_le_bytes([rec.data[2], rec.data[3]]) as usize
                    } else {
                        rec.data[2] as usize
                    };
                    let text_start = if flags & 0x01 != 0 { 4 } else { 3 };
                    if flags & 0x01 != 0 {
                        let u16_chars: Vec<u16> = rec.data[text_start..text_start + strlen * 2]
                            .chunks(2)
                            .filter(|c| c.len() == 2)
                            .map(|c| u16::from_le_bytes([c[0], c[1]]))
                            .collect();
                        let s = String::from_utf16_lossy(&u16_chars);
                        if !s.trim().is_empty() {
                            texts.push(s);
                        }
                    } else {
                        let end = (text_start + strlen).min(rec.data.len());
                        let s = String::from_utf8_lossy(&rec.data[text_start..end]).to_string();
                        if !s.trim().is_empty() {
                            texts.push(s);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if texts.is_empty() {
        // Fallback: extract any printable strings from the stream data
        let s = String::from_utf8_lossy(data);
        for line in s.lines() {
            let trimmed = line.trim();
            if trimmed.len() > 2 && trimmed.chars().all(|c| c.is_ascii() && !c.is_control()) {
                texts.push(trimmed.to_string());
            }
        }
    }

    texts
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_eocd_not_found_empty() {
        assert!(find_eocd(b"").is_none());
    }

    #[test]
    fn test_xml_unescape_amp() {
        assert_eq!(xml_unescape("a&amp;b"), "a&b");
    }

    #[test]
    fn test_xml_unescape_lt_gt() {
        assert_eq!(xml_unescape("&lt;tag&gt;"), "<tag>");
    }

    #[test]
    fn test_xml_unescape_no_entities() {
        assert_eq!(xml_unescape("hello world"), "hello world");
    }

    #[test]
    fn test_extract_xml_tag_text_simple() {
        let xml = b"<root><name>Alice</name></root>";
        assert_eq!(extract_xml_tag_text(xml, "name"), Some("Alice".to_string()));
    }

    #[test]
    fn test_extract_xml_tag_text_missing() {
        let xml = b"<root><name>Alice</name></root>";
        assert_eq!(extract_xml_tag_text(xml, "age"), None);
    }

    #[test]
    fn test_extract_all_xml_tags() {
        let xml = b"<root><item>A</item><item>B</item></root>";
        let items = extract_all_xml_tags(xml, "item");
        assert_eq!(items, vec!["A", "B"]);
    }

    #[test]
    fn test_parse_cell_ref_a1() {
        assert_eq!(parse_cell_ref("A1"), Some((0, 0)));
    }

    #[test]
    fn test_parse_cell_ref_z100() {
        assert_eq!(parse_cell_ref("Z100"), Some((25, 99)));
    }

    #[test]
    fn test_parse_cell_ref_aa1() {
        assert_eq!(parse_cell_ref("AA1"), Some((26, 0)));
    }

    #[test]
    fn test_extract_shared_strings_simple() {
        let xml = br#"<?xml version="1.0"?><sst><si><t>Hello</t></si><si><t>World</t></si></sst>"#;
        let strings = extract_shared_strings(xml);
        assert_eq!(strings, vec!["Hello", "World"]);
    }

    #[test]
    fn test_ole2_header_invalid_magic() {
        let result = Ole2Reader::new(&[0u8; 512]);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_attr_simple() {
        let tag = r#"<sheet name="Sheet1" sheetId="1" r:id="rId1"/>"#;
        assert_eq!(extract_attr(tag, "name"), Some("Sheet1".to_string()));
        assert_eq!(extract_attr(tag, "r:id"), Some("rId1".to_string()));
    }

    #[test]
    fn test_extract_sheet_names() {
        let xml = br#"<?xml version="1.0"?>
        <workbook>
          <sheets>
            <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
            <sheet name="Data" sheetId="2" r:id="rId2"/>
          </sheets>
        </workbook>"#;
        let sheets = extract_sheet_names(xml);
        assert_eq!(sheets.len(), 2);
        assert_eq!(sheets[0].1, "Sheet1");
        assert_eq!(sheets[1].1, "Data");
    }

    #[test]
    fn test_extract_rels_map() {
        let xml = br#"<?xml version="1.0"?>
        <Relationships>
          <Relationship Id="rId1" Target="worksheets/sheet1.xml" Type="http://..."/>
          <Relationship Id="rId2" Target="sharedStrings.xml" Type="http://..."/>
        </Relationships>"#;
        let map = extract_rels_map(xml);
        assert_eq!(
            map.get("rId1").map(|s| s.as_str()),
            Some("worksheets/sheet1.xml")
        );
        assert_eq!(
            map.get("rId2").map(|s| s.as_str()),
            Some("sharedStrings.xml")
        );
    }

    #[test]
    fn test_parse_biff_records_empty() {
        let records = parse_biff_records(&[]);
        assert!(records.is_empty());
    }

    #[test]
    fn test_parse_biff_records_single() {
        let mut data = Vec::new();
        // BOF record type 0x0809, size 16
        data.extend_from_slice(&[0x09, 0x08, 0x10, 0x00]);
        data.extend_from_slice(&[0u8; 16]);
        // EOF record type 0x000A, size 0
        data.extend_from_slice(&[0x0A, 0x00, 0x00, 0x00]);
        let records = parse_biff_records(&data);
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].record_type, 0x0809);
        assert_eq!(records[1].record_type, 0x000A);
    }

    #[test]
    fn test_extract_shared_strings_rich_text() {
        let xml = br#"<sst><si><r><rPr><sz val="11"/></rPr><t>Rich</t></r><r><rPr><sz val="11"/></rPr><t>Text</t></r></si></sst>"#;
        let strings = extract_shared_strings(xml);
        assert_eq!(strings, vec!["RichText"]);
    }
}

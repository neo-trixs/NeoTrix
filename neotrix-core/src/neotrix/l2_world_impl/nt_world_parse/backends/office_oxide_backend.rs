use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::Path;
use quick_xml::events::Event;
use quick_xml::Reader;
use quick_xml::XmlVersion;
use zip::ZipArchive;

// ─── Document IR (internal intermediate representation) ───

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum IrElement {
    Heading(u8, String),
    Paragraph(Vec<IrInline>),
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    List { ordered: bool, items: Vec<String> },
    Image { alt: String, src: String },
    ThematicBreak,
    CodeBlock(String),
    Slide { title: String, body: Vec<IrInline> },
    Note(String),
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct IrInline {
    text: String,
    bold: bool,
    italic: bool,
    strike: bool,
    code: bool,
    link: Option<(String, String)>,
}

#[allow(dead_code)]
fn text_inline(t: &str) -> IrInline {
    IrInline { text: t.to_string(), bold: false, italic: false, strike: false, code: false, link: None }
}

#[allow(dead_code)]
fn inlines_to_text(inlines: &[IrInline]) -> String {
    let mut out = String::new();
    for inc in inlines {
        let mut t = inc.text.clone();
        if inc.bold { t = format!("**{}**", t); }
        if inc.italic { t = format!("*{}*", t); }
        if inc.strike { t = format!("~~{}~~", t); }
        if inc.code { t = format!("`{}`", t); }
        if let Some((url, label)) = &inc.link {
            t = format!("[{}]({})", label, url);
        }
        out.push_str(&t);
    }
    out
}

#[allow(dead_code)]
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[allow(dead_code)]
fn col_letter(i: usize) -> String {
    let mut n = i + 1;
    let mut s = String::new();
    while n > 0 {
        n -= 1;
        s.insert(0, (b'A' + (n % 26) as u8) as char);
        n /= 26;
    }
    s
}

// ─── OPC (Open Packaging Conventions) Reader ───

struct OpcReader {
    #[allow(dead_code)]
    archive: ZipArchive<std::fs::File>,
    parts: HashMap<String, Vec<u8>>,
}

impl OpcReader {
    fn open(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("open {:?}: {}", path, e))?;
        let mut archive = ZipArchive::new(file).map_err(|e| format!("zip open {:?}: {}", path, e))?;
        let mut parts = HashMap::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| format!("entry {}: {}", i, e))?;
            let name = entry.name().to_string();
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).map_err(|e| format!("read {}: {}", name, e))?;
            parts.insert(name, buf);
        }
        Ok(Self { archive: ZipArchive::new(std::fs::File::open(path).map_err(|e| format!("reopen: {}", e))?).map_err(|e| format!("zip: {}", e))?, parts })
    }

    fn read_xml(&self, path: &str) -> Result<String, String> {
        let data = self.parts.get(path).ok_or_else(|| format!("part not found: {}", path))?;
        String::from_utf8(data.clone()).map_err(|e| format!("utf8 {}: {}", path, e))
    }

    fn part_names(&self) -> Vec<&str> {
        self.parts.keys().map(|s| s.as_str()).collect()
    }

    #[allow(dead_code)]
    fn part_data(&self, path: &str) -> Option<&[u8]> {
        self.parts.get(path).map(|v| v.as_slice())
    }
}

// ─── CFB (OLE2 Compound Binary) Reader — minimal implementation ───

struct CfbReader {
    data: Vec<u8>,
    sector_size: u32,
    mini_sector_size: u32,
    fat: Vec<u32>,
    mini_fat: Vec<u32>,
    directory: Vec<CfbDirEntry>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CfbDirEntry {
    name: String,
    object_type: u8,
    left_sibling: u32,
    right_sibling: u32,
    child: u32,
    start_sector: u32,
    size: u64,
}

impl CfbReader {
    fn open(path: &Path) -> Result<Self, String> {
        let data = std::fs::read(path).map_err(|e| format!("read {:?}: {}", path, e))?;
        Self::from_bytes(&data)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() < 512 {
            return Err("file too small for CFB".into());
        }
        let sig: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        if data[0..8] != sig[..] {
            return Err("not a CFB file (bad magic)".into());
        }
        let sector_size = u16::from_le_bytes([data[30], data[31]]) as u32;
        let mini_sector_size = u16::from_le_bytes([data[64], data[65]]) as u32;
        let _num_fat = u32::from_le_bytes([data[44], data[45], data[46], data[47]]);
        let dir_start = u32::from_le_bytes([data[48], data[49], data[50], data[51]]);
        let mini_fat_start = u32::from_le_bytes([data[60], data[61], data[62], data[63]]);
        let mini_fat_size = u32::from_le_bytes([data[56], data[57], data[58], data[59]]);

        let ss = sector_size as usize;
        let mss = mini_sector_size as usize;
        if ss < 512 { return Err("sector size too small".into()); }

        // Read DIFAT
        let mut difat: Vec<u32> = Vec::new();
        for i in 0..109 {
            let val = u32::from_le_bytes([data[76 + i * 4], data[77 + i * 4], data[78 + i * 4], data[79 + i * 4]]);
            if val != u32::MAX { difat.push(val); }
        }
        let num_extra_difat = u32::from_le_bytes([data[68], data[69], data[70], data[71]]);
        if num_extra_difat > 0 {
            let extra_difat_start = u32::from_le_bytes([data[72], data[73], data[74], data[75]]) as usize;
            if extra_difat_start < data.len() / ss {
                let mut pos = extra_difat_start * ss;
                for _ in 0..num_extra_difat {
                    if pos + 4 > data.len() { break; }
                    let val = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]);
                    if val != u32::MAX { difat.push(val); }
                    pos += 4;
                }
            }
        }

        // Read FAT
        let mut fat = Vec::new();
        for &sector in &difat {
            let start = sector as usize * ss;
            if start + ss > data.len() { continue; }
            for j in (0..ss).step_by(4) {
                if start + j + 4 > data.len() { break; }
                let val = u32::from_le_bytes([data[start + j], data[start + j + 1], data[start + j + 2], data[start + j + 3]]);
                fat.push(val);
            }
        }

        // Read Mini FAT
        let mut mini_fat = Vec::new();
        if mini_fat_start != u32::MAX && mini_fat_size > 0 {
            let chain = Self::read_sector_chain(data, ss, mini_fat_start as usize, &fat);
            for s in &chain {
                let start = *s as usize * ss;
                if start + ss > data.len() { continue; }
                for j in (0..ss).step_by(4) {
                    if start + j + 4 > data.len() { break; }
                    let val = u32::from_le_bytes([data[start + j], data[start + j + 1], data[start + j + 2], data[start + j + 3]]);
                    mini_fat.push(val);
                }
            }
        }

        // Read directory
        let dir_chain = Self::read_sector_chain(data, ss, dir_start as usize, &fat);
        let mut directory = Vec::new();
        for &sector in &dir_chain {
            let start = sector as usize * ss;
            for entry_idx in 0..(ss / 128) {
                let pos = start + entry_idx * 128;
                if pos + 128 > data.len() { break; }
                let name_size = u16::from_le_bytes([data[pos + 64], data[pos + 65]]) as usize;
                let name_bytes = &data[pos..pos + name_size.min(64)];
                let name = String::from_utf16_lossy(
                    &name_bytes.chunks(2)
                        .filter(|c| c.len() == 2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .collect::<Vec<_>>()
                ).trim_end_matches('\0').to_string();
                let obj_type = data[pos + 66];
                if obj_type == 0 { continue; }
                directory.push(CfbDirEntry {
                    name,
                    object_type: obj_type,
                    left_sibling: u32::from_le_bytes([data[pos+68], data[pos+69], data[pos+70], data[pos+71]]),
                    right_sibling: u32::from_le_bytes([data[pos+72], data[pos+73], data[pos+74], data[pos+75]]),
                    child: u32::from_le_bytes([data[pos+76], data[pos+77], data[pos+78], data[pos+79]]),
                    start_sector: u32::from_le_bytes([data[pos+116], data[pos+117], data[pos+118], data[pos+119]]),
                    size: u64::from_le_bytes([data[pos+120], data[pos+121], data[pos+122], data[pos+123], data[pos+124], data[pos+125], data[pos+126], data[pos+127]]),
                });
            }
        }

        Ok(Self { data: data.to_vec(), sector_size: ss as u32, mini_sector_size: mss as u32, fat, mini_fat, directory })
    }

    fn read_sector_chain(data: &[u8], sector_size: usize, start: usize, fat: &[u32]) -> Vec<u32> {
        let mut chain = Vec::new();
        let mut current = start as u32;
        let max_sector = data.len() / sector_size;
        // A valid chain visits each sector at most once. Crafted FAT can
        // contain cycles (fat[i]==i or a->b->a); a visited set stops the
        // loop before it grows unbounded on untrusted files.
        let mut visited = vec![false; max_sector];
        loop {
            if current as usize >= max_sector { break; }
            let idx = current as usize;
            if visited[idx] { break; }
            visited[idx] = true;
            chain.push(current);
            if idx >= fat.len() { break; }
            let next = fat[idx];
            if next == u32::MAX || next == 0xFFFFFFFE { break; }
            current = next;
        }
        chain
    }

    fn read_stream(&self, start_sector: u32, size: u64) -> Vec<u8> {
        let ss = self.sector_size as usize;
        if size < 4096 {
            // Mini stream
            let mss = self.mini_sector_size as usize;
            if mss == 0 { return vec![]; }
            let mini_stream_start = self.directory.iter()
                .find(|e| e.name == "Root Entry")
                .map(|e| e.start_sector)
                .unwrap_or(u32::MAX);
            if mini_stream_start == u32::MAX { return vec![]; }
            let chain = Self::read_sector_chain(&self.data, ss, mini_stream_start as usize, &self.fat);
            let mut mini_data = Vec::new();
            for &s in &chain {
                let pos = s as usize * ss;
                if pos + ss > self.data.len() { break; }
                mini_data.extend_from_slice(&self.data[pos..pos + ss]);
            }
            let mini_chain = Self::read_sector_chain(&mini_data, mss, start_sector as usize, &self.mini_fat);
            let mut out = Vec::new();
            for &s in &mini_chain {
                let pos = s as usize * mss;
                if pos + mss > mini_data.len() { break; }
                out.extend_from_slice(&mini_data[pos..pos + mss]);
            }
            out.truncate(size as usize);
            out
        } else {
            let chain = Self::read_sector_chain(&self.data, ss, start_sector as usize, &self.fat);
            let mut out = Vec::new();
            for &s in &chain {
                let pos = s as usize * ss;
                if pos + ss > self.data.len() { break; }
                out.extend_from_slice(&self.data[pos..pos + ss]);
            }
            out.truncate(size as usize);
            out
        }
    }

    fn find_stream_by_name(&self, name: &str) -> Option<(u32, u64)> {
        for entry in &self.directory {
            if entry.object_type == 2 && entry.name.trim_end_matches('\0') == name {
                return Some((entry.start_sector, entry.size));
            }
        }
        None
    }

    fn all_streams(&self) -> Vec<(String, Vec<u8>)> {
        let mut result = Vec::new();
        for entry in &self.directory {
            if entry.object_type == 2 && !entry.name.is_empty() {
                let data = self.read_stream(entry.start_sector, entry.size);
                result.push((entry.name.clone(), data));
            }
        }
        result
    }
}

// ─── XML parser helpers ───

fn read_xml_events(xml: &str) -> Vec<(String, HashMap<String, String>, String)> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut events = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut attrs = HashMap::new();
                for attr in e.attributes().flatten() {
                    if let Ok(v) = attr.normalized_value(XmlVersion::Implicit1_0) {
                        attrs.insert(String::from_utf8_lossy(attr.key.as_ref()).to_string(), v.to_string());
                    }
                }
                events.push((tag, attrs, String::new()));
            }
            Ok(Event::Text(ref e)) => {
                if let Ok(t) = e.decode() {
                    if let Some(last) = events.last_mut() {
                        last.2.push_str(&t);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                events.push((format!("/{}", tag), HashMap::new(), String::new()));
            }
            Ok(Event::Empty(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut attrs = HashMap::new();
                for attr in e.attributes().flatten() {
                    if let Ok(v) = attr.normalized_value(XmlVersion::Implicit1_0) {
                        attrs.insert(String::from_utf8_lossy(attr.key.as_ref()).to_string(), v.to_string());
                    }
                }
                let tag_clone = tag.clone();
                events.push((tag, attrs, String::new()));
                events.push((format!("/{}", tag_clone), HashMap::new(), String::new()));
            }
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    events
}

// ─── Format-specific parsers ───

fn parse_rels(xml: &str) -> HashMap<String, String> {
    let events = read_xml_events(xml);
    let mut map = HashMap::new();
    for (tag, attrs, _) in &events {
        if tag == "Relationship" {
            if let (Some(id), Some(target)) = (attrs.get("Id"), attrs.get("Target")) {
                map.insert(id.clone(), target.clone());
            }
        }
    }
    map
}

fn parse_docx_to_markdown(path: &Path) -> Result<String, String> {
    let opc = OpcReader::open(path)?;
    let document_xml = opc.read_xml("word/document.xml")?;
    let events = read_xml_events(&document_xml);

    // Read relationships for hyperlink resolution
    let rels: HashMap<String, String> = match opc.read_xml("word/_rels/document.xml.rels") {
        Ok(xml) => parse_rels(&xml),
        Err(_) => HashMap::new(),
    };

    let mut markdown = String::new();
    let mut in_paragraph = false;
    let mut in_run = false;
    let mut in_table = false;
    let mut in_cell = false;
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();
    let mut bold = false;
    let mut italic = false;
    let mut underline = false;
    let mut superscript = false;
    let mut subscript = false;
    let mut par_text = String::new();
    let mut depth_stack: Vec<String> = Vec::new();
    let mut in_hyperlink = false;
    let mut hyperlink_id = String::new();
    let mut hyperlink_anchor = String::new();
    let mut hyperlink_text = String::new();
    let mut bookmark_names: HashSet<String> = HashSet::new();
    let mut grid_span: usize = 1;
    let mut vmerge_state: Vec<bool> = Vec::new();
    let mut col_idx: usize = 0;

    for (tag, attrs, text) in &events {
        let tag_short = tag.trim_start_matches('/').trim_start_matches("w:").trim_start_matches("a:").trim_start_matches("m:");

        match tag_short {
            "p" | "pPr" | "rPr" | "numPr" if tag.starts_with('/') => {
                if tag_short == "p" {
                    if in_paragraph {
                        if !current_cell.is_empty() || !par_text.trim().is_empty() {
                            if in_cell {
                                if bold || italic {
                                    let mut t = par_text.trim().to_string();
                                    if bold { t = format!("**{}**", t); bold = false; }
                                    if italic { t = format!("*{}*", t); italic = false; }
                                    current_cell.push_str(&t);
                                } else {
                                    current_cell.push_str(par_text.trim());
                                }
                                current_cell.push(' ');
                            } else {
                                depth_stack.push(par_text.trim().to_string());
                            }
                        }
                        par_text.clear();
                    }
                    in_paragraph = false;
                }
                if tag_short == "p" && in_paragraph && !par_text.trim().is_empty() {
                    if in_cell { current_cell.push_str(par_text.trim()); current_cell.push(' '); }
                    else { depth_stack.push(par_text.trim().to_string()); }
                    par_text.clear();
                }
            }
            "p" | "pPr" | "rPr" | "numPr" => {
                if tag_short == "p" {
                    if !par_text.trim().is_empty() || !current_cell.is_empty() {
                        if in_cell { current_cell.push(' '); }
                        else if !par_text.trim().is_empty() { depth_stack.push(par_text.trim().to_string()); }
                    }
                    in_paragraph = true;
                    par_text.clear();
                    bold = false;
                    italic = false;
                    underline = false;
                    superscript = false;
                    subscript = false;
                }
            }
            "r" => {
                if tag.starts_with('/') {
                    bold = false;
                    italic = false;
                    underline = false;
                    superscript = false;
                    subscript = false;
                }
                in_run = !tag.starts_with('/');
            }
            "t" => {
                if !tag.starts_with('/') && in_run && in_paragraph {
                    let t = text.trim();
                    if in_hyperlink {
                        hyperlink_text.push_str(t);
                    } else {
                        let formatted = if superscript { format!("^{}^", t) }
                            else if subscript { format!("~{}~", t) }
                            else if underline { format!("++{}++", t) }
                            else if bold && italic { format!("***{}***", t) }
                            else if bold { format!("**{}**", t) }
                            else if italic { format!("*{}*", t) }
                            else { t.to_string() };
                        par_text.push_str(&formatted);
                    }
                }
            }
            "b" if !tag.starts_with('/') => bold = true,
            "i" if !tag.starts_with('/') => italic = true,
            "u" if !tag.starts_with('/') => {
                underline = attrs.get("w:val").map(|v| v == "single" || v == "words").unwrap_or(true);
            }
            "vertAlign" if !tag.starts_with('/') => {
                superscript = attrs.get("w:val").map(|v| v == "superscript").unwrap_or(false);
                subscript = attrs.get("w:val").map(|v| v == "subscript").unwrap_or(false);
            }
            "br" => { par_text.push('\n'); }
            "tab" => { par_text.push(' '); }
            "hyperlink" => {
                if !tag.starts_with('/') {
                    in_hyperlink = true;
                    hyperlink_id = attrs.get("r:id").or_else(|| attrs.get("id")).cloned().unwrap_or_default();
                    hyperlink_anchor = attrs.get("w:anchor").or_else(|| attrs.get("anchor")).cloned().unwrap_or_default();
                    hyperlink_text.clear();
                    hyperlink_text.push('|'); // marker for hyperlink start
                } else {
                    if in_hyperlink {
                        let url = if !hyperlink_anchor.is_empty() {
                            format!("#{}", hyperlink_anchor)
                        } else {
                            rels.get(&hyperlink_id).cloned().unwrap_or_default()
                        };
                        if !url.is_empty() && !hyperlink_text.is_empty() {
                            let display = hyperlink_text.trim_start_matches('|').trim();
                            if !display.is_empty() {
                                par_text.push_str(&format!("[{}]({})", display, url));
                            }
                        } else if !hyperlink_text.is_empty() {
                            par_text.push_str(hyperlink_text.trim_start_matches('|').trim());
                        }
                        in_hyperlink = false;
                        hyperlink_id.clear();
                        hyperlink_anchor.clear();
                        hyperlink_text.clear();
                    }
                }
            }
            "bookmarkStart" if !tag.starts_with('/') => {
                if let Some(name) = attrs.get("w:name").or_else(|| attrs.get("name")) {
                    if !name.is_empty() {
                        bookmark_names.insert(name.clone());
                    }
                }
            }
            "tbl" => {
                if tag.starts_with('/') && in_table {
                    // Flush table
                    if !table_rows.is_empty() {
                        markdown.push('\n');
                        for (ri, row) in table_rows.iter().enumerate() {
                            markdown.push('|');
                            for cell in row {
                                markdown.push_str(&format!(" {} |", cell.trim()));
                            }
                            markdown.push('\n');
                            if ri == 0 {
                                markdown.push('|');
                                for _ in row {
                                    markdown.push_str(" --- |");
                                }
                                markdown.push('\n');
                            }
                        }
                        markdown.push('\n');
                    }
                    table_rows.clear();
                    in_table = false;
                } else {
                    in_table = true;
                    table_rows.clear();
                }
            }
            "tr" if !tag.starts_with('/') => { current_cell.clear(); current_row.clear(); }
            "tr" if tag.starts_with('/') => {
                if in_table && !current_row.is_empty() {
                    table_rows.push(current_row.clone());
                    current_row.clear();
                }
            }
            "tc" => {
                if !tag.starts_with('/') {
                    in_cell = true;
                    current_cell.clear();
                    let gs = attrs.get("w:gridSpan").or_else(|| attrs.get("gridSpan"))
                        .and_then(|v| v.parse::<usize>().ok()).unwrap_or(1);
                    grid_span = gs;
                    // Check vMerge
                    let has_vmerge = attrs.get("w:vMerge").is_some() || attrs.get("vMerge").is_some();
                    let vmerge_val = attrs.get("w:vMerge")
                        .or_else(|| attrs.get("vMerge"));
                    if has_vmerge && vmerge_val.map(|v| v != "restart").unwrap_or(true) {
                        // Continuation cell — skip adding to row; track column advance
                        in_cell = false;
                    } else {
                        if !has_vmerge && col_idx < vmerge_state.len() {
                            vmerge_state[col_idx] = false;
                        }
                        if has_vmerge && vmerge_val.map(|v| v == "restart").unwrap_or(false) {
                            if col_idx >= vmerge_state.len() {
                                vmerge_state.resize(col_idx + 1, false);
                            }
                            vmerge_state[col_idx] = true;
                        }
                    }
                } else {
                    in_cell = false;
                    if in_table && grid_span > 0 {
                        // Check if this cell was skipped by vMerge
                        if col_idx < vmerge_state.len() && vmerge_state[col_idx] {
                            // Advance col_idx past this merge's span cells
                            col_idx += grid_span;
                            grid_span = 1;
                        } else {
                            current_row.push(current_cell.trim().to_string());
                            for _ in 1..grid_span {
                                current_row.push(String::new());
                            }
                            col_idx += grid_span;
                            grid_span = 1;
                        }
                    }
                }
            }
            "tr" if !tag.starts_with('/') => { current_cell.clear(); current_row.clear(); col_idx = 0; }
            "tr" if tag.starts_with('/') => {
                if in_table && !current_row.is_empty() {
                    table_rows.push(current_row.clone());
                    current_row.clear();
                }
            }
            "pStyle" | "ilvl" | "numId" => { /* style tracking — future use */ }
            "drawing" | "pict" => { /* image placeholder */ }
            "footnoteReference" => {
                if !tag.starts_with('/') {
                    if let Some(fid) = attrs.get("w:id").or_else(|| attrs.get("id")) {
                        par_text.push_str(&format!("[^{}]", fid.trim()));
                    }
                }
            }
            "endnoteReference" => {
                if !tag.starts_with('/') {
                    if let Some(eid) = attrs.get("w:id").or_else(|| attrs.get("id")) {
                        par_text.push_str(&format!("[^{}]", eid.trim()));
                    }
                }
            }
            _ => {}
        }
    }

    // Process paragraphs into markdown
    for line in &depth_stack {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }
        if let Some(h) = trimmed.strip_prefix("Heading1:") { markdown.push_str(&format!("# {}\n\n", h.trim())); }
        else if let Some(h) = trimmed.strip_prefix("Heading2:") { markdown.push_str(&format!("## {}\n\n", h.trim())); }
        else if let Some(h) = trimmed.strip_prefix("Heading3:") { markdown.push_str(&format!("### {}\n\n", h.trim())); }
        else { markdown.push_str(&format!("{}\n\n", trimmed)); }
    }

    if markdown.is_empty() {
        // Fallback: simple text extraction
        let mut reader = Reader::from_str(&document_xml);
        reader.config_mut().trim_text(true);
        let mut buf = Vec::new();
        let mut in_t = false;
        let mut text = String::new();
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "w:t" || tag == "t" { in_t = true; }
                }
                Ok(Event::Text(ref e)) => {
                    if in_t { if let Ok(t) = e.decode() { text.push_str(t.trim()); text.push(' '); } }
                }
                Ok(Event::End(ref e)) => {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if tag == "w:t" || tag == "t" { in_t = false; }
                    if tag == "w:p" || tag == "w:br" { text.push('\n'); }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
        markdown = text.trim().to_string();
    }

    // Detect embedded images
    let image_count = opc.part_names().iter().filter(|n| n.contains("word/media/")).count();
    if image_count > 0 {
        markdown.push_str(&format!("\n<!-- {} embedded images -->\n", image_count));
    }

    // Parse footnotes
    if let Ok(footnotes_xml) = opc.read_xml("word/footnotes.xml") {
        let fn_events = read_xml_events(&footnotes_xml);
        let mut fn_lines: Vec<(String, String)> = Vec::new();
        let mut in_footnote = false;
        let mut fn_id = String::new();
        let mut fn_type = String::new();
        let mut fn_text = String::new();
        for (fntag, fnattrs, fntext) in &fn_events {
            let short = fntag.trim_start_matches("w:");
            match short {
                "footnote" if !fntag.starts_with('/') => {
                    in_footnote = true;
                    fn_id = fnattrs.get("w:id").or_else(|| fnattrs.get("id")).cloned().unwrap_or_default();
                    fn_type = fnattrs.get("w:type").or_else(|| fnattrs.get("type")).cloned().unwrap_or_default();
                    fn_text.clear();
                }
                "footnote" if fntag.starts_with('/') => {
                    if in_footnote && fn_type != "separator" && fn_type != "continuationSeparator" && !fn_text.trim().is_empty() {
                        fn_lines.push((fn_id.clone(), fn_text.trim().to_string()));
                    }
                    in_footnote = false;
                }
                "t" if !fntag.starts_with('/') && in_footnote => {
                    fn_text.push_str(fntext.trim());
                }
                _ => {}
            }
        }
        if !fn_lines.is_empty() {
            markdown.push_str("\n---\n");
            for (id, text) in &fn_lines {
                markdown.push_str(&format!("[^{}]: {}\n", id, text));
            }
            markdown.push('\n');
        }
    }

    // Parse endnotes
    if let Ok(endnotes_xml) = opc.read_xml("word/endnotes.xml") {
        let en_events = read_xml_events(&endnotes_xml);
        let mut en_lines: Vec<(String, String)> = Vec::new();
        let mut in_en = false;
        let mut en_id = String::new();
        let mut en_type = String::new();
        let mut en_text = String::new();
        for (entag, enattrs, entext) in &en_events {
            let short = entag.trim_start_matches("w:");
            match short {
                "endnote" if !entag.starts_with('/') => {
                    in_en = true;
                    en_id = enattrs.get("w:id").or_else(|| enattrs.get("id")).cloned().unwrap_or_default();
                    en_type = enattrs.get("w:type").or_else(|| enattrs.get("type")).cloned().unwrap_or_default();
                    en_text.clear();
                }
                "endnote" if entag.starts_with('/') => {
                    if in_en && en_type != "separator" && en_type != "continuationSeparator" && !en_text.trim().is_empty() {
                        en_lines.push((en_id.clone(), en_text.trim().to_string()));
                    }
                    in_en = false;
                }
                "t" if !entag.starts_with('/') && in_en => {
                    en_text.push_str(entext.trim());
                }
                _ => {}
            }
        }
        if !en_lines.is_empty() {
            markdown.push_str("\n---\n");
            for (id, text) in &en_lines {
                markdown.push_str(&format!("[^{}]: {}\n", id, text));
            }
            markdown.push('\n');
        }
    }

    Ok(markdown)
}

fn parse_xlsx_to_markdown(path: &Path) -> Result<String, String> {
    let opc = OpcReader::open(path)?;

    // Read shared strings
    let shared_strings: Vec<String> = match opc.read_xml("xl/sharedStrings.xml") {
        Ok(xml) => {
            let events = read_xml_events(&xml);
            let mut ss = Vec::new();
            let mut in_si = false;
            let mut in_t = false;
            let mut current = String::new();
            for (tag, _, text) in &events {
                match tag.as_str() {
                    "si" => { in_si = true; current.clear(); }
                    "/si" => { if in_si { ss.push(current.trim().to_string()); } in_si = false; }
                    "t" if !tag.starts_with('/') && in_si => { in_t = true; if !text.is_empty() { current.push_str(text); } }
                    "/t" => in_t = false,
                    _ => { if in_t && !text.is_empty() { current.push_str(text); } }
                }
            }
            ss
        }
        Err(_) => vec![],
    };

    // Read workbook for sheet names
    let _sheet_names: HashMap<String, String> = match opc.read_xml("xl/workbook.xml") {
        Ok(xml) => {
            let events = read_xml_events(&xml);
            let mut names = HashMap::new();
            let mut in_sheet = false;
            let mut sheet_id = String::new();
            let mut sheet_name = String::new();
            for (tag, attrs, _) in &events {
                match tag.as_str() {
                    "sheet" => { in_sheet = true; sheet_id = attrs.get("sheetId").cloned().unwrap_or_default(); sheet_name = attrs.get("name").cloned().unwrap_or_default(); }
                    "/sheet" => { if in_sheet && !sheet_id.is_empty() { names.insert(sheet_id.clone(), sheet_name.clone()); } in_sheet = false; }
                    _ => {}
                }
            }
            names
        }
        Err(_) => HashMap::new(),
    };

    // Read styles for date format detection
    let date_style_ids: Vec<usize> = match opc.read_xml("xl/styles.xml") {
        Ok(xml) => {
            let events = read_xml_events(&xml);
            let mut date_ids = Vec::new();
            for (tag, attrs, _text) in &events {
                if tag.as_str() == "numFmt" {
                    if let (Some(fmt_id), Some(fmt_code)) = (
                        attrs.get("numFmtId").and_then(|v| v.parse::<usize>().ok()),
                        attrs.get("formatCode"),
                    ) {
                        if fmt_code.contains('y') || fmt_code.contains('d') || fmt_code.contains('m') {
                            date_ids.push(fmt_id);
                        }
                    }
                }
            }
            // Also include standard date format IDs
            date_ids.extend_from_slice(&[14, 15, 16, 17, 18, 19, 20, 21, 22, 27, 28, 29, 30, 31, 36, 45, 46, 47, 48, 55, 56, 57, 58]);
            date_ids
        }
        Err(_) => vec![14, 15, 16, 17, 18, 19, 20, 21, 22, 27, 28, 29, 30, 31, 36, 45, 46, 47, 48, 55, 56, 57, 58],
    };

    let mut markdown = String::new();

    // Find all worksheets
    let sheet_rels: HashMap<String, String> = opc.read_xml("xl/_rels/workbook.xml.rels").ok().map(|xml| {
        let events = read_xml_events(&xml);
        let mut map = HashMap::new();
        for (tag, attrs, _) in &events {
            if tag == "Relationship" {
                let id = attrs.get("Id").cloned().unwrap_or_default();
                let target = attrs.get("Target").cloned().unwrap_or_default();
                if target.contains("worksheets/") {
                    let sheet_name = target.trim_start_matches("worksheets/").trim_end_matches(".xml").to_string();
                    map.insert(id, sheet_name);
                }
            }
        }
        map
    }).unwrap_or_default();

    // Order sheets by the workbook relationship order
    let sheet_order: Vec<(String, String)> = opc.read_xml("xl/workbook.xml").ok().map(|xml| {
        let events = read_xml_events(&xml);
        let mut order = Vec::new();
        for (tag, attrs, _) in &events {
            if tag == "sheet" {
                let rid = attrs.get("r:id").or_else(|| attrs.get("id")).cloned().unwrap_or_default();
                let name = attrs.get("name").cloned().unwrap_or_default();
                order.push((rid, name));
            }
        }
        order
    }).unwrap_or_default();

    let mut processed_sheets = 0;
    for (rid, sheet_name) in &sheet_order {
        // Find the actual target path
        let target = if rid.starts_with("rId") {
            sheet_rels.get(rid.as_str()).cloned().unwrap_or_default()
        } else {
            rid.clone()
        };
        let sheet_path = format!("xl/worksheets/{}.xml", if target.ends_with(".xml") { target.trim_end_matches(".xml") } else { &target });

        let sheet_xml = match opc.read_xml(&sheet_path) {
            Ok(x) => x,
            Err(_) => continue,
        };
        // Read sheet-level relationships for hyperlink resolution
        let sheet_rels_path = format!("xl/worksheets/_rels/{}", sheet_path.trim_start_matches("xl/worksheets/"), );
        let sheet_rels_path = sheet_rels_path.trim_end_matches(".xml").to_string() + ".xml.rels";
        let sheet_rels_map: HashMap<String, String> = match opc.read_xml(&sheet_rels_path) {
            Ok(xml) => parse_rels(&xml),
            Err(_) => HashMap::new(),
        };

        let events = read_xml_events(&sheet_xml);

        // Pre-pass: collect hyperlinks (hyperlinks section comes after sheetData)
        let mut hyperlink_refs: HashMap<String, String> = HashMap::new();
        for (tag, attrs, _) in &events {
            if tag == "hyperlink" {
                if let Some(cell) = attrs.get("ref") {
                    let href_id = attrs.get("r:id").or_else(|| attrs.get("id")).cloned().unwrap_or_default();
                    if let Some(url) = sheet_rels_map.get(&href_id) {
                        hyperlink_refs.insert(cell.clone(), url.clone());
                    }
                }
            }
        }

        let mut rows: Vec<Vec<(usize, String)>> = Vec::new();
        let mut current_row_data: Vec<(usize, String)> = Vec::new();
        let mut in_cell = false;
        let mut in_v = false;
        let mut in_is = false;
        let mut in_is_t = false;
        let mut cell_ref = String::new();
        let mut cell_type = String::new();
        let mut cell_style = String::new();
        let mut cell_value = String::new();
        let mut inline_str = String::new();

        for (tag, attrs, text) in &events {
            match tag.as_str() {
                "row" if !tag.starts_with('/') => {
                    current_row_data.clear();
                }
                "/row" => {
                    if !current_row_data.is_empty() {
                        rows.push(current_row_data.clone());
                    }
                }
                "c" if !tag.starts_with('/') => {
                    in_cell = true;
                    cell_ref = attrs.get("r").cloned().unwrap_or_default();
                    cell_type = attrs.get("t").cloned().unwrap_or_default();
                    cell_style = attrs.get("s").cloned().unwrap_or_default();
                    cell_value.clear();
                    inline_str.clear();
                }
                "/c" => {
                    if in_cell {
                        let col = col_index(&cell_ref);
                        let mut val = if cell_type == "s" {
                            let idx: usize = cell_value.trim().parse().unwrap_or(usize::MAX);
                            shared_strings.get(idx).cloned().unwrap_or_default()
                        } else if cell_type == "inlineStr" || cell_type == "str" {
                            inline_str.clone()
                        } else if cell_type == "b" {
                            if cell_value == "1" { "TRUE".to_string() } else { "FALSE".to_string() }
                        } else if cell_type == "e" {
                            format!("#{}", cell_value.trim())
                        } else if !cell_style.is_empty() {
                            let style_id: usize = cell_style.parse().unwrap_or(0);
                            let numfmt_id = date_style_ids.iter().find(|&&id| id == style_id);
                            if numfmt_id.is_some() || date_style_ids.contains(&style_id) {
                                if let Ok(serial) = cell_value.trim().parse::<f64>() {
                                    if serial > 1.0 && serial < 100000.0 {
                                        format!("[date:{}]", serial)
                                    } else {
                                        cell_value.trim().to_string()
                                    }
                                } else {
                                    cell_value.trim().to_string()
                                }
                            } else {
                                cell_value.trim().to_string()
                            }
                        } else {
                            cell_value.trim().to_string()
                        };
                        // Apply hyperlink if this cell has one
                        if !val.is_empty() {
                            if let Some(url) = hyperlink_refs.get(&cell_ref) {
                                val = format!("[{}]({})", val, url);
                            }
                        }
                        if !val.is_empty() {
                            current_row_data.push((col, val));
                        }
                    }
                    in_cell = false;
                }
                "v" if !tag.starts_with('/') => in_v = true,
                "/v" => in_v = false,
                "is" if !tag.starts_with('/') => in_is = true,
                "/is" => in_is = false,
                "t" if !tag.starts_with('/') => {
                    if in_is { in_is_t = true; }
                    if in_is_t && !text.is_empty() { inline_str.push_str(text); }
                }
                "/t" => { in_is_t = false; }
                _ => {
                    if in_v && !text.is_empty() { cell_value.push_str(text); }
                    if in_is_t && !text.is_empty() { inline_str.push_str(text); }
                }
            }
        }

        if rows.is_empty() { continue; }

        processed_sheets += 1;
        let display_name = if sheet_name.is_empty() { format!("Sheet{}", processed_sheets) } else { sheet_name.clone() };
        if processed_sheets > 1 || sheet_order.len() > 1 {
            markdown.push_str(&format!("## Sheet: {}\n\n", display_name));
        }

        let max_col = rows.iter().flat_map(|r| r.iter().map(|(c, _)| *c)).max().unwrap_or(0);
        for (ri, row) in rows.iter().enumerate() {
            let mut cells: Vec<&str> = vec![""; max_col + 1];
            for (c, v) in row {
                if *c <= max_col { cells[*c] = v; }
            }
            markdown.push('|');
            for cell in &cells {
                markdown.push_str(&format!(" {} |", cell));
            }
            markdown.push('\n');
            if ri == 0 {
                markdown.push('|');
                for _ in 0..cells.len() {
                    markdown.push_str(" --- |");
                }
                markdown.push('\n');
            }
        }
        markdown.push('\n');
    }

    if processed_sheets == 0 {
        return Err("no worksheets found in XLSX".into());
    }

    let img_count = opc.part_names().iter().filter(|n| n.contains("xl/media/")).count();
    if img_count > 0 {
        markdown.push_str(&format!("\n<!-- {} embedded images -->\n", img_count));
    }

    Ok(markdown.trim().to_string())
}

fn col_index(cell_ref: &str) -> usize {
    let col_part: String = cell_ref.chars().take_while(|c| c.is_ascii_alphabetic()).collect();
    let mut idx = 0usize;
    for c in col_part.chars() {
        idx = idx * 26 + (c.to_ascii_uppercase() as usize - b'A' as usize + 1);
    }
    idx.saturating_sub(1)
}

fn parse_pptx_to_markdown(path: &Path) -> Result<String, String> {
    let opc = OpcReader::open(path)?;
    let mut slides: Vec<(u32, String, String)> = Vec::new();

    // Find all slide files
    for name in opc.part_names() {
        if name.starts_with("ppt/slides/slide") && name.ends_with(".xml") {
            let num: u32 = name.trim_start_matches("ppt/slides/slide")
                .trim_end_matches(".xml")
                .parse().unwrap_or(0);

            // Read slide-level relationships for hyperlink resolution
            let slide_rels_name = format!("ppt/slides/_rels/slide{}.xml.rels", num);
            let slide_rels: HashMap<String, String> = match opc.read_xml(&slide_rels_name) {
                Ok(xml) => parse_rels(&xml),
                Err(_) => HashMap::new(),
            };

            let xml = opc.read_xml(name)?;
            let events = read_xml_events(&xml);

            let mut title = String::new();
            let mut body = String::new();
            let in_text = false;
            let mut in_p = false;
            let mut current_par = String::new();
            let mut is_title_shape = false;
            let mut shape_idx = 0;
            let mut is_subtitle = false;
            let mut in_cn_vpr = false;
            let mut in_a_rpr = false;
            let mut shape_hyperlink = String::new();
            let mut run_hyperlink = String::new();

            for (tag, attrs, text) in &events {
                let tag_short = tag.trim_start_matches('/').trim_start_matches("p:").trim_start_matches("a:").trim_start_matches("r:");
                match tag_short {
                    "sp" if !tag.starts_with('/') => {
                        shape_idx += 1;
                        is_title_shape = false;
                        is_subtitle = false;
                        shape_hyperlink.clear();
                    }
                    "sp" if tag.starts_with('/') => {
                        if !current_par.is_empty() {
                            let raw = if !shape_hyperlink.is_empty() {
                                format!("[{}]({})", current_par.trim(), shape_hyperlink)
                            } else {
                                current_par.trim().to_string()
                            };
                            if is_title_shape || is_subtitle || (shape_idx <= 2 && title.is_empty()) {
                                if title.is_empty() { title = raw; }
                                else { title.push_str(&format!(". {}", raw)); }
                            } else {
                                body.push_str(&raw);
                                body.push('\n');
                            }
                            current_par.clear();
                        }
                    }
                    "nvSpPr" | "nvs" | "cNvSpPr" | "spPr" => {}
                    "cNvPr" if !tag.starts_with('/') => { in_cn_vpr = true; }
                    "cNvPr" if tag.starts_with('/') => { in_cn_vpr = false; }
                    "hlinkClick" if !tag.starts_with('/') => {
                        let href_id = attrs.get("r:id").or_else(|| attrs.get("id")).cloned().unwrap_or_default();
                        let url = slide_rels.get(&href_id).cloned().unwrap_or_default();
                        if !url.is_empty() {
                            if in_cn_vpr {
                                shape_hyperlink = url;
                            } else if in_a_rpr {
                                run_hyperlink = url;
                            }
                        }
                    }
                    "rPr" if !tag.starts_with('/') => { in_a_rpr = true; run_hyperlink.clear(); }
                    "rPr" if tag.starts_with('/') => { in_a_rpr = false; }
                    "ph" => {
                        if let Some(val) = attrs.get("type") {
                            is_title_shape = val == "title" || val == "ctrTitle";
                            is_subtitle = val == "subTitle";
                        }
                    }
                    "p" if !tag.starts_with('/') => { in_p = true; if !current_par.is_empty() { current_par.push('\n'); } }
                    "p" if tag.starts_with('/') => { in_p = false; }
                    "t" => {
                        if !tag.starts_with('/') && (in_p || in_text) {
                            let t = text.trim();
                            let formatted = if !run_hyperlink.is_empty() {
                                format!("[{}]({})", t, run_hyperlink)
                            } else {
                                t.to_string()
                            };
                            current_par.push_str(&formatted);
                        }
                    }
                    "r" => { run_hyperlink.clear(); }
                    _ => {
                        if !tag.starts_with('/') && !text.is_empty() && in_p {
                            current_par.push_str(text.trim());
                        }
                    }
                }
            }

            // Fallback: use first text-heavy shape as title
            if title.is_empty() {
                let mut reader = Reader::from_str(&xml);
                reader.config_mut().trim_text(true);
                let mut buf = Vec::new();
                let mut in_a_t = false;
                let mut texts: Vec<String> = Vec::new();
                loop {
                    match reader.read_event_into(&mut buf) {
                        Ok(Event::Start(ref e)) => {
                            if e.name().as_ref() == b"a:t" || e.name().as_ref() == b"t" { in_a_t = true; }
                        }
                        Ok(Event::Text(ref e)) => {
                            if in_a_t { if let Ok(t) = e.decode() { texts.push(t.trim().to_string()); } }
                        }
                        Ok(Event::End(ref e)) => {
                            if e.name().as_ref() == b"a:t" || e.name().as_ref() == b"t" { in_a_t = false; }
                            if e.name().as_ref() == b"a:p" || e.name().as_ref() == b"p" { texts.push("\n".to_string()); }
                        }
                        Ok(Event::Eof) => break,
                        _ => {}
                    }
                    buf.clear();
                }
                if !texts.is_empty() {
                    let all = texts.concat();
                    let parts: Vec<&str> = all.split('\n').collect();
                    if parts.len() >= 2 {
                        title = parts[0].trim().to_string();
                        body = parts[1..].join("\n").trim().to_string();
                    } else {
                        title = all.trim().to_string();
                    }
                }
            }

            slides.push((num, title, body));
        }
    }

    if slides.is_empty() {
        return Err("no slides found in PPTX".into());
    }

    slides.sort_by_key(|(num, _, _)| *num);

    let mut markdown = String::new();
    for (i, (_num, title, body)) in slides.iter().enumerate() {
        markdown.push_str(&format!("## Slide {}\n\n", i + 1));
        if !title.is_empty() {
            markdown.push_str(&format!("**{}**\n\n", title));
        }
        if !body.is_empty() {
            markdown.push_str(body);
            markdown.push('\n');
        }
    }

    let img_count = opc.part_names().iter().filter(|n| n.contains("ppt/media/")).count();
    if img_count > 0 {
        markdown.push_str(&format!("\n<!-- {} embedded images -->\n", img_count));
    }

    Ok(markdown.trim().to_string())
}

fn parse_doc_to_markdown(path: &Path) -> Result<String, String> {
    let cfb = CfbReader::open(path)?;

    // Find the WordDocument stream
    let (start, size) = cfb.find_stream_by_name("WordDocument")
        .or_else(|| cfb.find_stream_by_name("1Table"))
        .or_else(|| cfb.find_stream_by_name("0Table"))
        .ok_or_else(|| "WordDocument stream not found in DOC".to_string())?;

    let data = cfb.read_stream(start, size);
    if data.len() < 10 {
        return Err("WordDocument stream too small".into());
    }

    // Try to extract text from the stream
    // DOC text often starts at offset 0 or after the FIB (File Information Block)
    let mut text = String::new();
    let mut in_text = false;

    // Simple approach: scan for text runs (Unicode or ASCII)
    // The WordDocument stream contains a FIB followed by text
    let mut i = 0;
    while i < data.len().saturating_sub(1) {
        let b = data[i];
        // Check for ASCII text
        if b.is_ascii_graphic() || b == b' ' {
            text.push(b as char);
            in_text = true;
        } else if b == b'\r' || b == b'\n' {
            text.push('\n');
            in_text = false;
        } else {
            if in_text {
                text.push(' ');
                in_text = false;
            }
            // Try UTF-16LE
            if i + 1 < data.len() {
                let u16_val = u16::from_le_bytes([data[i], data[i + 1]]);
                if (0x20..=0x7E).contains(&u16_val) {
                    text.push(u16_val as u8 as char);
                    i += 1;
                    in_text = true;
                }
            }
        }
        i += 1;
    }

    // Also check 1Table or 0Table for additional text (the CLX)
    if let Some((ts, ts_size)) = cfb.find_stream_by_name("1Table")
        .or_else(|| cfb.find_stream_by_name("0Table"))
    {
        let table_data = cfb.read_stream(ts, ts_size);
        if !table_data.is_empty() {
            // Try to find any readable ASCII in table streams
            let mut extra = String::new();
            for &b in &table_data {
                if b.is_ascii_graphic() || b == b' ' {
                    extra.push(b as char);
                } else if b == b'\r' || b == b'\n' {
                    extra.push('\n');
                } else if !extra.ends_with(' ') {
                    extra.push(' ');
                }
            }
            let extra_trimmed = extra.trim();
            if !extra_trimmed.is_empty() && extra_trimmed.len() > 20 {
                text.push_str("\n\n");
                text.push_str(extra_trimmed);
            }
        }
    }

    let clean: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.len() < 10 {
        return Err("could not extract meaningful text from DOC".into());
    }

    // Apply simple paragraph detection
    let paragraphs: Vec<&str> = text.split('\n')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    let mut markdown = String::new();
    for p in &paragraphs {
        let words: Vec<&str> = p.split_whitespace().collect();
        let cleaned = words.join(" ");
        // Heuristic: short lines could be headings
        if !cleaned.is_empty() && cleaned.len() < 60 && !cleaned.ends_with('.') && !cleaned.ends_with('?') && !cleaned.ends_with('!') {
            // Check if it looks like a heading (short, no sentence punctuation)
            markdown.push_str(&format!("## {}\n\n", cleaned));
        } else { markdown.push_str(&format!("{}\n\n", cleaned)); }
    }

    Ok(markdown.trim().to_string())
}

fn parse_xls_to_markdown(path: &Path) -> Result<String, String> {
    let cfb = CfbReader::open(path)?;
    let mut markdown = String::new();

    // Look for the Workbook stream (XLS)
    if let Some((start, size)) = cfb.find_stream_by_name("Workbook")
        .or_else(|| cfb.find_stream_by_name("Book"))
    {
        let data = cfb.read_stream(start, size);
        if data.len() >= 8 {
            // Parse BIFF8: look for BOF records (0x0809) and Sheet records (0x0085)
            let mut pos = 0;
            let mut sheet_names: Vec<String> = Vec::new();

            while pos + 4 <= data.len() {
                let rec_type = u16::from_le_bytes([data[pos], data[pos + 1]]);
                let rec_len = u16::from_le_bytes([data[pos + 2], data[pos + 3]]) as usize;
                if pos + 4 + rec_len > data.len() { break; }

                match rec_type {
                    0x0809 => { /* BOF */ }
                    0x0085 => {
                        // Sheet record
                        if rec_len >= 2 {
                            let flags = u32::from_le_bytes([data[pos + 4], data[pos + 5], 0, 0]);
                            let _ = flags;
                            // Sheet name is at offset 6 as UTF-16LE
                            let name_bytes = &data[pos + 6..pos + 4 + rec_len.min(64)];
                            let name: String = name_bytes.chunks(2)
                                .filter(|c| c.len() == 2)
                                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                .take_while(|&c| c != 0)
                                .map(|c| c as u8 as char)
                                .collect();
                            if !name.is_empty() {
                                sheet_names.push(name);
                            }
                        }
                    }
                    0x00FD => { /* Label (string cell) */ }
                    _ => {}
                }
                pos += 4 + rec_len;
            }

            // Extract text from the Workbook stream
            let mut text = String::new();
            for &b in &data {
                if b.is_ascii_graphic() || b == b' ' || b == b'\t' {
                    text.push(b as char);
                } else if b == b'\r' || b == b'\n' {
                    text.push('\n');
                } else if !text.ends_with(' ') {
                    text.push(' ');
                }
            }

            if !sheet_names.is_empty() {
                markdown.push_str(&format!("## Sheets: {}\n\n", sheet_names.join(", ")));
            }

            if !text.trim().is_empty() {
                markdown.push_str(text.trim());
            }
        }
    }

    if markdown.trim().is_empty() {
        // Fallback: extract any readable text from all streams
        for (name, data) in cfb.all_streams() {
            if name.contains("SummaryInformation") || name.contains("DocumentSummaryInformation") {
                continue;
            }
            let mut text = String::new();
            for &b in &data {
                if b.is_ascii_graphic() || b == b' ' { text.push(b as char); }
                else if b == b'\r' || b == b'\n' { text.push('\n'); }
                else if !text.ends_with(' ') { text.push(' '); }
            }
            let trimmed = text.trim();
            if !trimmed.is_empty() && trimmed.len() > 20 {
                markdown.push_str(&format!("### {}\n\n{}\n\n", name, trimmed));
            }
        }
    }

    let final_md = markdown.trim().to_string();
    if final_md.len() < 10 {
        Err("could not extract meaningful text from XLS".into())
    } else {
        Ok(final_md)
    }
}

fn parse_ppt_to_markdown(path: &Path) -> Result<String, String> {
    let cfb = CfbReader::open(path)?;
    let mut markdown = String::new();
    let mut slide_num = 0;

    // Look for PowerPoint Document stream
    if let Some((start, size)) = cfb.find_stream_by_name("PowerPoint Document") {
        let data = cfb.read_stream(start, size);
        if data.len() >= 8 {
            // Extract text by scanning for readable content
            let mut slide_text = String::new();
            let mut text = String::new();
            let mut i = 0;
            while i < data.len() {
                let b = data[i];
                if b.is_ascii_graphic() || b == b' ' {
                    if text.ends_with('\n') && b.is_ascii_alphabetic() {
                        // Potential heading
                    }
                    text.push(b as char);
                } else if b == b'\r' || b == b'\n' {
                    text.push('\n');
                } else if b == 0x00 && i + 1 < data.len() {
                    // Unicode text
                    let next = data[i + 1];
                    if next.is_ascii_graphic() || next == b' ' {
                        // UTF-16LE ASCII-range character
                        i += 1;
                        continue;
                    }
                }
                i += 1;
            }

            // Try to split into slides by looking for specific patterns
            let lines: Vec<&str> = text.lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .collect();

            for line in lines {
                if line.len() < 60 && !line.ends_with('.') && !line.ends_with(',') {
                    // Could be a slide title
                    if slide_text.len() > 20 {
                        slide_num += 1;
                        markdown.push_str(&format!("## Slide {}\n\n{}\n\n", slide_num, slide_text.trim()));
                        slide_text.clear();
                    }
                    slide_text.push_str(line);
                    slide_text.push('\n');
                } else {
                    slide_text.push_str(line);
                    slide_text.push('\n');
                }
            }

            if !slide_text.trim().is_empty() {
                slide_num += 1;
                markdown.push_str(&format!("## Slide {}\n\n{}\n\n", slide_num, slide_text.trim()));
            }

            if slide_num == 0 {
                // Just output all text
                markdown.push_str(text.trim());
            }
        }
    }

    // Current Text stream
    if let Some((cs, cs_size)) = cfb.find_stream_by_name("Current User") {
        let user_data = cfb.read_stream(cs, cs_size);
        let user_text: String = user_data.iter()
            .filter(|&&b| b.is_ascii_graphic() || b == b' ')
            .map(|&b| b as char)
            .collect();
        if !user_text.trim().is_empty() {
            // Append as note
        }
    }

    let final_md = markdown.trim().to_string();
    if final_md.len() < 10 {
        Err("could not extract meaningful text from PPT".into())
    } else {
        Ok(final_md)
    }
}

// ─── Format detection ───

#[derive(Debug, Clone, Copy, PartialEq)]
enum DocumentFormat {
    Docx, Xlsx, Pptx, Doc, Xls, Ppt,
}

impl DocumentFormat {
    fn from_path(path: &Path) -> Option<Self> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "docx" => Some(Self::Docx),
            "xlsx" => Some(Self::Xlsx),
            "pptx" => Some(Self::Pptx),
            "doc" => Some(Self::Doc),
            "xls" => Some(Self::Xls),
            "ppt" => Some(Self::Ppt),
            _ => None,
        }
    }

    fn from_magic(data: &[u8]) -> Option<Self> {
        // ZIP magic: PK\x03\x04
        if data.len() >= 4 && data[0] == 0x50 && data[1] == 0x4B && data[2] == 0x03 && data[3] == 0x04 {
            if data.len() > 30 {
                // Check for OOXML magic strings in the ZIP comment or file names
                // We can't easily distinguish without parsing ZIP entries
            }
            return Some(Self::Docx); // Default ZIP → DOCX (most common)
        }
        // CFB magic: D0CF11E0
        if data.len() >= 8 {
            let cfb_magic: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
            if data[0..8] == cfb_magic {
                return Some(Self::Doc); // Default CFB → DOC (most common)
            }
        }
        None
    }

    #[allow(dead_code)]
    fn extensions(&self) -> Vec<&'static str> {
        match self {
            Self::Docx => vec!["docx"],
            Self::Xlsx => vec!["xlsx"],
            Self::Pptx => vec!["pptx"],
            Self::Doc => vec!["doc"],
            Self::Xls => vec!["xls", "xla", "xlt"],
            Self::Ppt => vec!["ppt", "ppa", "pot"],
        }
    }
}

// ─── Main converter ───

fn detect_format(path: &Path) -> Result<DocumentFormat, String> {
    if let Some(fmt) = DocumentFormat::from_path(path) {
        return Ok(fmt);
    }
    // Try magic bytes
    let data = std::fs::read(path).map_err(|e| format!("read {:?}: {}", path, e))?;
    DocumentFormat::from_magic(&data)
        .ok_or_else(|| format!("unable to detect format: {:?}", path))
}

fn convert_to_markdown(path: &Path) -> Result<String, String> {
    let fmt = detect_format(path)?;
    match fmt {
        DocumentFormat::Docx => parse_docx_to_markdown(path),
        DocumentFormat::Xlsx => parse_xlsx_to_markdown(path),
        DocumentFormat::Pptx => parse_pptx_to_markdown(path),
        DocumentFormat::Doc => parse_doc_to_markdown(path),
        DocumentFormat::Xls => parse_xls_to_markdown(path),
        DocumentFormat::Ppt => parse_ppt_to_markdown(path),
    }
}

fn is_supported_format(path: &Path) -> bool {
    DocumentFormat::from_path(path).is_some()
}

// ─── Public API ───

pub struct OfficeOxideBackend;

impl super::super::doc_parser::DocParser for OfficeOxideBackend {
    fn parse_pdf(&self, path: &Path) -> Result<super::super::doc_parser::ParsedDocument, String> {
        if is_supported_format(path) {
            self.parse_office(path)
        } else {
            Err("OfficeOxideBackend does not support PDF parsing".into())
        }
    }

    fn parse_image(&self, _path: &Path) -> Result<super::super::doc_parser::ParsedDocument, String> {
        Err("OfficeOxideBackend does not support image parsing".into())
    }

    fn parse_office(&self, path: &Path) -> Result<super::super::doc_parser::ParsedDocument, String> {
        let markdown = convert_to_markdown(path)?;

        let page_count = markdown.lines()
            .filter(|l| l.starts_with("## Slide") || l.starts_with("## Sheet"))
            .count().max(1);

        let pages = vec![super::super::doc_parser::PageResult {
            page_num: 1,
            markdown: markdown.clone(),
            confidence: 0.95,
            backend_used: "OfficeOxideBackend".into(),
            metadata: Default::default(),
        }];

        let fmt = detect_format(path).ok();
        let mut meta = std::collections::HashMap::new();
        if let Some(f) = fmt {
            meta.insert("format".into(), format!("{:?}", f));
        }

        Ok(super::super::doc_parser::ParsedDocument {
            title: path.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()),
            pages,
            full_markdown: markdown,
            full_json: serde_json::json!({"pages": page_count}),
            avg_confidence: 0.95,
            metadata: meta,
        })
    }

    fn supported_formats(&self) -> Vec<&str> {
        vec!["docx", "xlsx", "pptx", "doc", "xls", "ppt"]
    }

    fn tier(&self) -> super::super::doc_parser::ParseTier {
        super::super::doc_parser::ParseTier::Tier0Fast
    }
}

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::doc_parser::{DocParser, ParseTier};
    use std::io::Write;
    use std::path::Path;
    use zip::write::FileOptions;
    use zip::CompressionMethod;
    use zip::ZipWriter;

    #[test]
    fn test_is_supported_format() {
        assert!(is_supported_format(Path::new("doc.docx")));
        assert!(is_supported_format(Path::new("deck.pptx")));
        assert!(is_supported_format(Path::new("sheet.xlsx")));
        assert!(is_supported_format(Path::new("legacy.doc")));
        assert!(is_supported_format(Path::new("legacy.xls")));
        assert!(is_supported_format(Path::new("legacy.ppt")));
        assert!(!is_supported_format(Path::new("doc.pdf")));
        assert!(!is_supported_format(Path::new("text.txt")));
    }

    #[test]
    fn test_format_from_path() {
        assert_eq!(DocumentFormat::from_path(Path::new("a.docx")), Some(DocumentFormat::Docx));
        assert_eq!(DocumentFormat::from_path(Path::new("a.xlsx")), Some(DocumentFormat::Xlsx));
        assert_eq!(DocumentFormat::from_path(Path::new("a.pptx")), Some(DocumentFormat::Pptx));
        assert_eq!(DocumentFormat::from_path(Path::new("a.doc")), Some(DocumentFormat::Doc));
        assert_eq!(DocumentFormat::from_path(Path::new("a.xls")), Some(DocumentFormat::Xls));
        assert_eq!(DocumentFormat::from_path(Path::new("a.ppt")), Some(DocumentFormat::Ppt));
        assert_eq!(DocumentFormat::from_path(Path::new("a.pdf")), None);
    }

    #[test]
    fn test_magic_detection() {
        let zip_magic = [0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00];
        assert!(DocumentFormat::from_magic(&zip_magic).is_some());

        let cfb_magic = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        assert!(DocumentFormat::from_magic(&cfb_magic).is_some());

        let pdf_magic = [0x25, 0x50, 0x44, 0x46];
        assert!(DocumentFormat::from_magic(&pdf_magic).is_none());
    }

    #[test]
    fn test_col_index() {
        assert_eq!(col_index("A1"), 0);
        assert_eq!(col_index("B1"), 1);
        assert_eq!(col_index("Z1"), 25);
        assert_eq!(col_index("AA1"), 26);
        assert_eq!(col_index("AZ1"), 51);
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a&b"), "a&amp;b");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("hello"), "hello");
    }

    #[test]
    fn test_read_sector_chain_breaks_on_cycle() {
        // Crafted FAT where sector 2 points at itself (fat[2]==2): a naive
        // loop would spin forever / grow unbounded. The guard must stop at
        // max_sector+1 entries.
        let data = vec![0u8; 8 * 512];
        let sector_size = 512;
        let mut fat = vec![u32::MAX; 8];
        fat[2] = 2; // self-cycle
        let chain = CfbReader::read_sector_chain(&data, sector_size, 2, &fat);
        assert_eq!(chain.len(), 1, "self-cycle must not produce an unbounded chain");

        // Two-cycle a->b->a
        let mut fat2 = vec![u32::MAX; 8];
        fat2[3] = 4;
        fat2[4] = 3;
        let chain2 = CfbReader::read_sector_chain(&data, sector_size, 3, &fat2);
        assert_eq!(chain2.len(), 2, "two-cycle must stop after visiting each sector once");

        // Valid chain still reads fully
        let mut fat3 = vec![u32::MAX; 8];
        fat3[5] = 6;
        let chain3 = CfbReader::read_sector_chain(&data, sector_size, 5, &fat3);
        assert_eq!(chain3, vec![5, 6]);
    }

    #[test]
    fn test_col_letter() {
        assert_eq!(col_letter(0), "A");
        assert_eq!(col_letter(25), "Z");
        assert_eq!(col_letter(26), "AA");
        assert_eq!(col_letter(701), "ZZ");
    }

    #[test]
    fn test_parse_nonexistent_file() {
        let backend = OfficeOxideBackend;
        let result = backend.parse_office(Path::new("/nonexistent/file.docx"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nonexistent_xlsx() {
        let backend = OfficeOxideBackend;
        let result = backend.parse_office(Path::new("/nonexistent/file.xlsx"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_nonexistent_pptx() {
        let backend = OfficeOxideBackend;
        let result = backend.parse_office(Path::new("/nonexistent/file.pptx"));
        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_format_pdf() {
        let backend = OfficeOxideBackend;
        let result = backend.parse_pdf(Path::new("doc.pdf"));
        assert!(result.is_err());
    }

    #[test]
    fn test_supported_formats() {
        let backend = OfficeOxideBackend;
        let formats = backend.supported_formats();
        assert!(formats.contains(&"docx"));
        assert!(formats.contains(&"doc"));
        assert!(formats.contains(&"xls"));
        assert!(formats.contains(&"ppt"));
        assert_eq!(formats.len(), 6);
    }

    #[test]
    fn test_tier() {
        let backend = OfficeOxideBackend;
        assert_eq!(backend.tier(), ParseTier::Tier0Fast);
    }

    #[test]
    fn test_extensions_all_formats() {
        let formats = [DocumentFormat::Docx, DocumentFormat::Xlsx, DocumentFormat::Pptx,
                       DocumentFormat::Doc, DocumentFormat::Xls, DocumentFormat::Ppt];
        for f in &formats {
            let exts = f.extensions();
            assert!(!exts.is_empty(), "{:?} should have extensions", f);
        }
    }

    #[test]
    fn test_docx_detection() {
        assert_eq!(DocumentFormat::from_path(Path::new("report.docx")), Some(DocumentFormat::Docx));
    }

    #[test]
    fn test_xlsx_detection() {
        assert_eq!(DocumentFormat::from_path(Path::new("data.xlsx")), Some(DocumentFormat::Xlsx));
    }

    #[test]
    fn test_pptx_detection() {
        assert_eq!(DocumentFormat::from_path(Path::new("slides.pptx")), Some(DocumentFormat::Pptx));
    }

    #[test]
    fn test_cfb_reader_invalid() {
        let result = CfbReader::from_bytes(&[0; 100]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cfb_reader_too_small() {
        let result = CfbReader::from_bytes(&[0; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_opc_reader_nonexistent() {
        let result = OpcReader::open(Path::new("/nonexistent/file.docx"));
        assert!(result.is_err());
    }

    // ─── Round-trip integration tests ───

    #[test]
    fn test_roundtrip_docx() {
        // Create DOCX via renderer, then parse it back
        let out = std::env::temp_dir().join("test_roundtrip_docx.docx");
        let _ = std::fs::remove_file(&out);

        let md = "# Hello World\n\nThis is a **paragraph**.\n\n- Item A\n- Item B\n";
        let result = crate::neotrix::l2_world_impl::nt_world_parse::renderers::office_renderer::OfficeRenderer::docx_from_markdown(md, &out);
        assert!(result.is_ok(), "docx creation: {:?}", result);
        assert!(out.exists(), "docx file should exist");

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        assert!(parsed.full_markdown.contains("Hello"), "should contain 'Hello', got: {}", parsed.full_markdown);
        assert!(parsed.full_markdown.contains("paragraph"), "should contain 'paragraph'");
        assert!(parsed.full_markdown.contains("Item"), "should contain 'Item'");

        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_roundtrip_docx_hyperlink() {
        let out = std::env::temp_dir().join("test_roundtrip_docx_hyperlink.docx");
        let _ = std::fs::remove_file(&out);

        let md = "Click [here](https://example.com) for more info.\n\nAlso visit [GitHub](https://github.com).\n";
        let result = crate::neotrix::l2_world_impl::nt_world_parse::renderers::office_renderer::OfficeRenderer::docx_from_markdown(md, &out);
        assert!(result.is_ok(), "docx creation: {:?}", result);
        assert!(out.exists(), "docx file should exist");

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        assert!(parsed.full_markdown.contains("[here](https://example.com)"),
            "should preserve hyperlink, got: {}", parsed.full_markdown);
        assert!(parsed.full_markdown.contains("[GitHub](https://github.com)"),
            "should preserve GitHub hyperlink, got: {}", parsed.full_markdown);

        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_roundtrip_docx_table() {
        let out = std::env::temp_dir().join("test_roundtrip_docx_table.docx");
        let _ = std::fs::remove_file(&out);

        let md = "| A | B |\n| --- | --- |\n| 1 | 2 |\n| 3 | 4 |\n";
        let result = crate::neotrix::l2_world_impl::nt_world_parse::renderers::office_renderer::OfficeRenderer::docx_from_markdown(md, &out);
        assert!(result.is_ok(), "docx creation: {:?}", result);
        assert!(out.exists(), "docx file should exist");

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        assert!(parsed.full_markdown.contains("**A**"), "should contain bold header **A**, got: {}", parsed.full_markdown);
        assert!(parsed.full_markdown.contains("| 1 |"), "should contain '| 1 |'");
        assert!(parsed.full_markdown.contains("3"), "should contain '3'");

        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_roundtrip_xlsx() {
        let out = std::env::temp_dir().join("test_roundtrip_xlsx.xlsx");
        let _ = std::fs::remove_file(&out);

        let md = "| Name | Score |\n| --- | --- |\n| Alice | 95 |\n| Bob | 87 |\n";
        let result = crate::neotrix::l2_world_impl::nt_world_parse::renderers::office_renderer::OfficeRenderer::xlsx_from_markdown(md, &out);
        assert!(result.is_ok(), "xlsx creation: {:?}", result);
        assert!(out.exists(), "xlsx file should exist");

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        assert!(parsed.full_markdown.contains("Alice"), "should contain 'Alice', got: {}", parsed.full_markdown);
        assert!(parsed.full_markdown.contains("87"), "should contain '87'");

        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_roundtrip_pptx() {
        let out = std::env::temp_dir().join("test_roundtrip_pptx.pptx");
        let _ = std::fs::remove_file(&out);

        let md = "# Slide One\n\nContent for first slide.\n\n# Slide Two\n\nContent for second slide.\n";
        let result = crate::neotrix::l2_world_impl::nt_world_parse::renderers::office_renderer::OfficeRenderer::pptx_from_markdown(md, &out);
        assert!(result.is_ok(), "pptx creation: {:?}", result);
        assert!(out.exists(), "pptx file should exist");

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        assert!(parsed.full_markdown.contains("Slide"), "should contain 'Slide', got: {}", parsed.full_markdown);
        assert!(parsed.full_markdown.contains("first"), "should contain 'first'");
        assert!(parsed.full_markdown.contains("second"), "should contain 'second'");

        let _ = std::fs::remove_file(&out);
    }

    // ─── Direct Parser Tests (construct minimal Office files) ───

    fn create_minimal_docx_zip(path: &Path, doc_xml: &str, rels_xml: &str) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts: FileOptions<'_, ()> = FileOptions::default().compression_method(CompressionMethod::Stored);

        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
        zip.add_directory("_rels/", opts).unwrap();
        zip.add_directory("word/", opts).unwrap();
        zip.add_directory("word/_rels/", opts).unwrap();

        zip.start_file("[Content_Types].xml", opts).unwrap();
        write!(zip, "{}", content_types).unwrap();

        zip.start_file("_rels/.rels", opts).unwrap();
        write!(zip, r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#).unwrap();

        zip.start_file("word/_rels/document.xml.rels", opts).unwrap();
        write!(zip, "{}", rels_xml).unwrap();

        zip.start_file("word/document.xml", opts).unwrap();
        write!(zip, "{}", doc_xml).unwrap();

        zip.finish().unwrap();
    }

    fn create_minimal_xlsx_zip(path: &Path, sheet_xml: &str, sheet_rels_xml: &str, shared_strings_xml: &str) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts: FileOptions<'_, ()> = FileOptions::default().compression_method(CompressionMethod::Stored);

        zip.add_directory("_rels/", opts).unwrap();
        zip.add_directory("xl/", opts).unwrap();
        zip.add_directory("xl/_rels/", opts).unwrap();
        zip.add_directory("xl/worksheets/", opts).unwrap();
        zip.add_directory("xl/worksheets/_rels/", opts).unwrap();

        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
</Types>"#;
        zip.start_file("[Content_Types].xml", opts).unwrap();
        write!(zip, "{}", content_types).unwrap();

        zip.start_file("_rels/.rels", opts).unwrap();
        write!(zip, r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#).unwrap();

        zip.start_file("xl/workbook.xml", opts).unwrap();
        write!(zip, r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
          xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#).unwrap();

        zip.start_file("xl/_rels/workbook.xml.rels", opts).unwrap();
        write!(zip, r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
</Relationships>"#).unwrap();

        if !shared_strings_xml.is_empty() {
            zip.start_file("xl/sharedStrings.xml", opts).unwrap();
            write!(zip, "{}", shared_strings_xml).unwrap();
        }

        zip.start_file("xl/worksheets/sheet1.xml", opts).unwrap();
        write!(zip, "{}", sheet_xml).unwrap();

        if !sheet_rels_xml.is_empty() {
            zip.start_file("xl/worksheets/_rels/sheet1.xml.rels", opts).unwrap();
            write!(zip, "{}", sheet_rels_xml).unwrap();
        }

        zip.finish().unwrap();
    }

    #[test]
    fn test_parse_docx_bookmarks() {
        let out = std::env::temp_dir().join("test_docx_bookmarks.docx");
        let _ = std::fs::remove_file(&out);

        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body>
  <w:p>
    <w:bookmarkStart w:id="0" w:name="section1"/>
    <w:r><w:t>Section 1</w:t></w:r>
    <w:bookmarkEnd w:id="0"/>
  </w:p>
  <w:p>
    <w:hyperlink w:anchor="section1">
      <w:r><w:t>Go to Section 1</w:t></w:r>
    </w:hyperlink>
  </w:p>
</w:body>
</w:document>"#;
        let rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;
        create_minimal_docx_zip(&out, doc_xml, rels_xml);

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        assert!(parsed.full_markdown.contains("[Go to Section 1](#section1)"),
            "should resolve w:anchor → [#section1], got: {}", parsed.full_markdown);

        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_parse_docx_footnotes() {
        let out = std::env::temp_dir().join("test_docx_footnotes.docx");
        let _ = std::fs::remove_file(&out);

        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body>
  <w:p>
    <w:r><w:t>Some text</w:t></w:r>
    <w:footnoteReference w:id="1"/>
    <w:r><w:t> and more.</w:t></w:r>
  </w:p>
  <w:footnoteReference w:id="2"/>
</w:body>
</w:document>"#;
        let rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;
        create_minimal_docx_zip(&out, doc_xml, rels_xml);

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        assert!(parsed.full_markdown.contains("[^1]"),
            "should contain footnote reference [^1], got: {}", parsed.full_markdown);

        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_parse_docx_merged_table() {
        let out = std::env::temp_dir().join("test_docx_merged_table.docx");
        let _ = std::fs::remove_file(&out);

        let doc_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body>
  <w:tbl>
    <w:tr>
      <w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p><w:r><w:t>Normal</w:t></w:r></w:p></w:tc>
      <w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p><w:r><w:t>Normal2</w:t></w:r></w:p></w:tc>
    </w:tr>
    <w:tr>
      <w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p><w:r><w:t>Left</w:t></w:r></w:p></w:tc>
      <w:tc><w:tcPr>
        <w:tcW w:w="0" w:type="auto"/>
        <w:gridSpan w:val="2"/>
      </w:tcPr><w:p><w:r><w:t>Merged</w:t></w:r></w:p></w:tc>
    </w:tr>
    <w:tr>
      <w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/>
        <w:vMerge w:val="restart"/>
      </w:tcPr><w:p><w:r><w:t>VMergeStart</w:t></w:r></w:p></w:tc>
      <w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p><w:r><w:t>B</w:t></w:r></w:p></w:tc>
    </w:tr>
    <w:tr>
      <w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/>
        <w:vMerge/>
      </w:tcPr><w:p><w:r><w:t>VMergeCont</w:t></w:r></w:p></w:tc>
      <w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p><w:r><w:t>C</w:t></w:r></w:p></w:tc>
    </w:tr>
  </w:tbl>
</w:body>
</w:document>"#;
        let rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;
        create_minimal_docx_zip(&out, doc_xml, rels_xml);

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        let md = &parsed.full_markdown;
        assert!(md.contains("Normal"), "should contain Normal");
        assert!(md.contains("Normal2"), "should contain Normal2");
        assert!(md.contains("Left"), "should contain Left");
        assert!(md.contains("Merged"), "should contain Merged cell value");
        assert!(md.contains("VMergeStart"), "should contain VMergeStart");
        assert!(md.contains("B"), "should contain B");
        assert!(md.contains("C"), "should contain C");

        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_parse_xlsx_hyperlinks() {
        let out = std::env::temp_dir().join("test_xlsx_hyperlinks.xlsx");
        let _ = std::fs::remove_file(&out);

        let sheet_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
           xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>Click Here</t></is></c>
      <c r="B1" t="inlineStr"><is><t>Visit Example</t></is></c>
    </row>
    <row r="2">
      <c r="A2" t="inlineStr"><is><t>Search</t></is></c>
    </row>
  </sheetData>
  <hyperlinks>
    <hyperlink ref="A1" r:id="rId1"/>
    <hyperlink ref="B1" r:id="rId2"/>
  </hyperlinks>
</worksheet>"#;

        let sheet_rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com" TargetMode="External"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://google.com" TargetMode="External"/>
</Relationships>"#;

        create_minimal_xlsx_zip(&out, sheet_xml, sheet_rels_xml, "");

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        let md = &parsed.full_markdown;
        assert!(md.contains("[Click Here](https://example.com)"),
            "should resolve hyperlink A1, got: {}", md);
        assert!(md.contains("[Visit Example](https://google.com)"),
            "should resolve hyperlink B1, got: {}", md);

        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_parse_pptx_hyperlinks() {
        let out = std::env::temp_dir().join("test_pptx_hyperlinks.pptx");
        let _ = std::fs::remove_file(&out);

        let slide_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sp>
    <p:nvSpPr>
      <p:cNvPr id="2" name="Title 2">
        <a:hlinkClick r:id="rId1"/>
      </p:cNvPr>
    </p:nvSpPr>
    <p:txBody>
      <a:bodyPr/>
      <a:p>
        <a:r>
          <a:t>Example Link</a:t>
        </a:r>
      </a:p>
    </p:txBody>
  </p:sp>
</p:sld>"#;

        let slide_rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com" TargetMode="External"/>
</Relationships>"#;

        create_minimal_pptx_zip(&out, slide_xml, slide_rels_xml);

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        let md = &parsed.full_markdown;
        assert!(md.contains("[Example Link](https://example.com)"),
            "should resolve a:hlinkClick, got: {}", md);

        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn test_parse_pptx_run_hyperlinks() {
        let out = std::env::temp_dir().join("test_pptx_run_hyperlinks.pptx");
        let _ = std::fs::remove_file(&out);

        let slide_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
       xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
       xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sp>
    <p:nvSpPr>
      <p:cNvPr id="2" name="Title 2"/>
    </p:nvSpPr>
    <p:txBody>
      <a:bodyPr/>
      <a:p>
        <a:r>
          <a:rPr>
            <a:hlinkClick r:id="rId1"/>
          </a:rPr>
          <a:t>Run Link</a:t>
        </a:r>
      </a:p>
    </p:txBody>
  </p:sp>
</p:sld>"#;

        let slide_rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://run.link" TargetMode="External"/>
</Relationships>"#;

        create_minimal_pptx_zip(&out, slide_xml, slide_rels_xml);

        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&out).expect("parse_office should succeed");
        let md = &parsed.full_markdown;
        assert!(md.contains("[Run Link](https://run.link)"),
            "should resolve run-level a:hlinkClick, got: {}", md);

        let _ = std::fs::remove_file(&out);
    }

    fn create_minimal_pptx_zip(path: &Path, slide_xml: &str, slide_rels_xml: &str) {
        let file = std::fs::File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts: FileOptions<'_, ()> = FileOptions::default().compression_method(CompressionMethod::Stored);

        let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
  <Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
</Types>"#;

        zip.add_directory("_rels/", opts).unwrap();
        zip.add_directory("ppt/", opts).unwrap();
        zip.add_directory("ppt/_rels/", opts).unwrap();
        zip.add_directory("ppt/slides/", opts).unwrap();
        zip.add_directory("ppt/slides/_rels/", opts).unwrap();

        zip.start_file("[Content_Types].xml", opts).unwrap();
        write!(zip, "{}", content_types).unwrap();

        zip.start_file("_rels/.rels", opts).unwrap();
        write!(zip, r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#).unwrap();

        zip.start_file("ppt/_rels/presentation.xml.rels", opts).unwrap();
        write!(zip, r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
</Relationships>"#).unwrap();

        zip.start_file("ppt/presentation.xml", opts).unwrap();
        write!(zip, r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"
                xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldIdLst>
    <p:sldId id="256" r:id="rId1"/>
  </p:sldIdLst>
  <p:sldSz cx="9144000" cy="6858000"/>
  <p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#).unwrap();

        zip.start_file("ppt/slides/slide1.xml", opts).unwrap();
        write!(zip, "{}", slide_xml).unwrap();

        zip.start_file("ppt/slides/_rels/slide1.xml.rels", opts).unwrap();
        write!(zip, "{}", slide_rels_xml).unwrap();

        zip.finish().unwrap();
    }
}

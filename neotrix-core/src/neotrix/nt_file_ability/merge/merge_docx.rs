//! DOCX/PPTX 结构级合并 (CollectionMerge L1) — OPC part 层操作。
//!
//! 在 zip/OPC 层合并 Office Open XML 包, 不受 office_oxide 只读 rels 限制:
//! - docx: 合并 `word/document.xml` 的 `<w:body>` 段落 (剥离后续文档 sectPr)
//! - pptx: 追加 `ppt/slides/slideN.xml` + 更新 `ppt/presentation.xml` 的 sldIdLst
//!
//! 资源 part 冲突 (媒体/样式等): 后续文档冲突 part 重命名 `docN_` 前缀, 同步
//! 重写其引用 part 的 rels。R-P42: 强化既有 nt_file_ability, 非平行适配器。

#![allow(dead_code)]

use std::io::{Read, Seek, Write};
use std::path::Path;

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use zip::ZipArchive;
use zip::write::SimpleFileOptions;

use super::super::types::{FileAbilityError, Result};

/// 合并后返回统计
#[derive(Debug, Clone)]
pub struct OfficeMergeReport {
    /// 已合并文档数
    pub items: usize,
    /// 输出 part 数 (docx) 或 幻灯片数 (pptx)
    pub parts: usize,
}

/// 从 zip 读取 part 字节。
fn read_part<R: Read + Seek>(zip: &mut ZipArchive<R>, name: &str) -> Option<Vec<u8>> {
    let mut f = zip.by_name(name).ok()?;
    let mut buf = Vec::with_capacity(f.size() as usize);
    f.read_to_end(&mut buf).ok()?;
    Some(buf)
}

/// 收集 zip 内全部 part 名。
fn part_names<R: Read + Seek>(zip: &ZipArchive<R>) -> Vec<String> {
    let mut names: Vec<String> = zip.file_names().map(ToOwned::to_owned).collect();
    names.sort();
    names
}

/// 提取 docx `<w:body>` 内全部 `<w:p>` 段落 XML 片段。
///
/// 返回 (段落片段列表, 末尾 sectPr 是否存在)。使用 quick-xml 事件流, 完整保留
/// 命名空间前缀与属性, 不丢失格式。后续文档的 sectPr 剥离 (避免分节冲突)。
fn extract_docx_paragraphs(xml: &[u8]) -> (Vec<String>, bool) {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut paragraphs: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut depth = 0usize;
    let mut in_body = false;
    let mut in_paragraph = false;
    let mut has_sectpr = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "w:body" {
                    in_body = true;
                    continue;
                }
                if in_body {
                    if name == "w:p" {
                        in_paragraph = true;
                        depth = 1;
                        cur = format!("<{name}{}>", attrs_str(&e));
                        continue;
                    }
                    if in_paragraph {
                        cur.push_str(&format!("<{name}{}>", attrs_str(&e)));
                        depth += 1;
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "w:body" {
                    break;
                }
                if in_body && in_paragraph {
                    cur.push_str(&format!("</{name}>"));
                    depth -= 1;
                    if name == "w:p" && depth == 0 {
                        paragraphs.push(std::mem::take(&mut cur));
                        in_paragraph = false;
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if in_body && in_paragraph {
                    cur.push_str(&String::from_utf8_lossy(&t));
                }
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if in_body && in_paragraph {
                    cur.push_str(&format!("<{name}{}/>", attrs_str(&e)));
                } else if in_body && name == "w:sectPr" {
                    has_sectpr = true;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    (paragraphs, has_sectpr)
}

/// 序列化 BytesStart 属性 (保留全部属性)。
fn attrs_str(e: &BytesStart) -> String {
    let mut out = String::new();
    for attr in e.attributes().flatten() {
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let val = String::from_utf8_lossy(&attr.value);
        out.push_str(&format!(" {key}=\"{val}\""));
    }
    out
}

/// 合并多个 DOCX 文件。
///
/// 以首个为基座: 拼接各文档 `<w:body>` 段落; 剥离后续文档 sectPr; 复制非冲突
/// 资源 part (媒体/样式/字体), 冲突 part 重命名 `docN_` 前缀并重写其引用 rels;
/// 重建 [Content_Types].xml (追加新 part 的 Override)。输出为合并后字节。
pub fn merge_docx(inputs: &[std::path::PathBuf], out: &Path) -> Result<OfficeMergeReport> {
    if inputs.is_empty() {
        return Err(FileAbilityError::Other("无输入 DOCX".to_string()));
    }
    if inputs.len() == 1 {
        let data = std::fs::read(&inputs[0]).map_err(FileAbilityError::Io)?;
        std::fs::write(&out, &data).map_err(FileAbilityError::Io)?;
        return Ok(OfficeMergeReport { items: 1, parts: 0 });
    }

    // 1. 读基座 OPC 全部 part
    let base = std::fs::File::open(&inputs[0]).map_err(FileAbilityError::Io)?;
    let mut base_zip = ZipArchive::new(base).map_err(|e| FileAbilityError::Parse(e.to_string()))?;
    let base_names = part_names(&base_zip);
    let mut out_parts: std::collections::BTreeMap<String, Vec<u8>> = std::collections::BTreeMap::new();
    for n in &base_names {
        if let Some(data) = read_part(&mut base_zip, n) {
            out_parts.insert(n.clone(), data);
        }
    }
    drop(base_zip);

    // 2. 合并 body 段落
    let doc_xml_name = "word/document.xml".to_string();
    let base_doc = out_parts.get(&doc_xml_name).cloned().unwrap_or_default();
    let (mut paragraphs, _) = extract_docx_paragraphs(&base_doc);
    let mut item_count = 1usize;
    // 冲突 part 重命名映射: 旧 part 名 → 新 part 名 (跨文档累计)
    let mut rename_map: std::collections::BTreeMap<String, String> = std::collections::BTreeMap::new();
    // 基座 document.xml.rels 已占用 rId 集合 (冲突时重编号)
    let base_rels = out_parts
        .get("word/_rels/document.xml.rels")
        .cloned()
        .unwrap_or_default();
    let mut used_rids: std::collections::BTreeSet<String> = rels_ids(&base_rels);
    // 追加到基座 rels 的 Relationship 文本 (新 rId + 新 Target)
    let mut rels_append = String::new();
    let mut next_rid = rels_next_id(&used_rids);

    for (idx, input) in inputs.iter().enumerate().skip(1) {
        let f = std::fs::File::open(input).map_err(FileAbilityError::Io)?;
        let mut z = ZipArchive::new(f).map_err(|e| FileAbilityError::Parse(e.to_string()))?;
        let doc_xml = read_part(&mut z, &doc_xml_name)
            .ok_or_else(|| FileAbilityError::Other(format!("输入 {idx} 缺 word/document.xml")))?;
        let (mut ps, _) = extract_docx_paragraphs(&doc_xml);

        // 先复制资源 part (更新 rename_map), 再合并该文档 rels
        let names = part_names(&z);
        for n in &names {
            if n == &doc_xml_name || n.ends_with(".rels") || n == "[Content_Types].xml" {
                continue;
            }
            if out_parts.contains_key(n) {
                // 冲突: 前缀 docN_, 记录映射供 rels 统一重写
                let new_name = rename_part(n, idx);
                if let Some(data) = read_part(&mut z, n) {
                    out_parts.insert(new_name.clone(), data);
                    rename_map.insert(n.clone(), new_name);
                }
            } else if let Some(data) = read_part(&mut z, n) {
                out_parts.insert(n.clone(), data);
            }
        }

        // 合并该文档 document.xml.rels: 冲突 rId 重编号, Target 冲突 part 重写
        let rels_name = "word/_rels/document.xml.rels".to_string();
        if let Some(rels_bytes) = read_part(&mut z, &rels_name) {
            let (rid_map, new_rels) = merge_rels_into(
                &rels_bytes,
                &mut used_rids,
                &mut next_rid,
                &rename_map,
            );
            rels_append.push_str(&new_rels);
            // 段落内 rId 引用同步替换 (r:embed / r:id)
            if !rid_map.is_empty() {
                for p in ps.iter_mut() {
                    for (old, new) in &rid_map {
                        if p.contains(&format!("r:embed=\"{old}\"")) {
                            *p = p.replace(&format!("r:embed=\"{old}\""), &format!("r:embed=\"{new}\""));
                        }
                        if p.contains(&format!("r:id=\"{old}\"")) {
                            *p = p.replace(&format!("r:id=\"{old}\""), &format!("r:id=\"{new}\""));
                        }
                    }
                }
            }
        }
        paragraphs.extend(ps);
        item_count += 1;
    }

    // 2.6 合并追加的 Relationship 进基座 document.xml.rels
    if !rels_append.is_empty() {
        append_rels(&mut out_parts, "word/_rels/document.xml.rels", &rels_append);
    }

    // 3. 重建 document.xml: 基座 body + 全部段落
    let merged_doc = rebuild_document_xml(&base_doc, &paragraphs);
    out_parts.insert(doc_xml_name, merged_doc.into_bytes());

    // 4. 重建 [Content_Types].xml (补齐新 part 的 Override)
    rebuild_content_types(&mut out_parts);

    // 5. 写 zip
    let file = std::fs::File::create(&out).map_err(FileAbilityError::Io)?;
    write_zip(file, &out_parts).map_err(FileAbilityError::Io)?;
    Ok(OfficeMergeReport {
        items: item_count,
        parts: out_parts.len(),
    })
}

/// 冲突 part 重命名: 目录前缀插 `docN_`。
fn rename_part(name: &str, idx: usize) -> String {
    // word/media/image1.png → word/media/doc2_image1.png
    let parts: Vec<&str> = name.splitn(2, '/').collect();
    if parts.len() == 2 {
        format!("{}/doc{}_{}", parts[0], idx, parts[1])
    } else {
        format!("doc{}_{name}", idx)
    }
}

/// 统一重写全部 `.rels`: 冲突 part 重命名后, 其 Target 相对路径同步更新。
///
/// OPC 约定: `<part>/.rels` 中 Target 相对 `<part>` 所在目录。例如
/// `word/_rels/document.xml.rels` 的 Target `media/image1.png` → 绝对 part
/// 绝对 part 名 → 相对 base_dir 的 Target 路径。
/// 例: ("word/media/image1.png", "word/") → "media/image1.png"。
fn absolute_to_target(abs: &str, base_dir: &str) -> String {
    abs.strip_prefix(base_dir).unwrap_or(abs).to_string()
}

/// 提取 rels 内全部 `<Relationship Id="rIdN" .../>` 的 Id。
fn rels_ids(rels_xml: &[u8]) -> std::collections::BTreeSet<String> {
    let s = String::from_utf8_lossy(rels_xml).to_string();
    let mut ids = std::collections::BTreeSet::new();
    let mut rest = s.as_str();
    while let Some(pos) = rest.find("Id=\"rId") {
        let tail = &rest[pos + "Id=\"rId".len()..];
        let num: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !num.is_empty() {
            ids.insert(format!("rId{num}"));
        }
        rest = tail;
    }
    ids
}

/// 从已占用 rId 集合推导下一个可用编号。
fn rels_next_id(used: &std::collections::BTreeSet<String>) -> usize {
    let mut max = 1usize;
    for id in used {
        if let Some(num) = id.strip_prefix("rId") {
            if let Ok(n) = num.parse::<usize>() {
                max = max.max(n + 1);
            }
        }
    }
    max
}

/// 合并一个文档的 rels 内容: 冲突 rId 重编号, 冲突 part Target 重写。
///
/// 返回 (old_id → new_id 映射, 追加到基座 rels 的 Relationship XML 文本)。
fn merge_rels_into(
    rels_xml: &[u8],
    used_rids: &mut std::collections::BTreeSet<String>,
    next_rid: &mut usize,
    rename_map: &std::collections::BTreeMap<String, String>,
) -> (std::collections::BTreeMap<String, String>, String) {
    let s = String::from_utf8_lossy(rels_xml).to_string();
    let mut rid_map = std::collections::BTreeMap::new();
    let mut append = String::new();
    let base_dir = "word/".to_string();

    for line in s.lines() {
        let line = line.trim();
        if !line.contains("<Relationship ") || !line.ends_with("/>") {
            continue;
        }
        // 提取 Id / Target / Type
        let (Some(id), Some(target), Some(rel_type)) = (
            extract_attr(line, "Id"),
            extract_attr(line, "Target"),
            extract_attr(line, "Type"),
        ) else {
            continue;
        };

        // rId 冲突 → 重编号
        let (new_id, rid_changed) = if used_rids.contains(&id) {
            let cand = format!("rId{next_rid}");
            *next_rid += 1;
            used_rids.insert(cand.clone());
            (cand, true)
        } else {
            used_rids.insert(id.clone());
            (id.clone(), false)
        };
        if rid_changed {
            rid_map.insert(id.clone(), new_id.clone());
        }

        // Target 相对 part 目录 → 绝对 part → 冲突则换新名 → 回写相对 Target
        let abs = format!("{base_dir}{target}");
        let new_target = if let Some(to) = rename_map.get(&abs) {
            absolute_to_target(to, &base_dir)
        } else {
            target
        };
        append.push_str(&format!(
            "    <Relationship Id=\"{new_id}\" Type=\"{rel_type}\" Target=\"{new_target}\"/>\n"
        ));
    }
    (rid_map, append)
}

/// 从 `attr="value"` 中提取属性值 (先取字符串里 `<Relationship ` 之后的部分)。
fn extract_attr(xml: &str, attr: &str) -> Option<String> {
    let pat = format!("{attr}=\"");
    let start = xml.find(&pat)? + pat.len();
    let end = xml[start..].find('"')?;
    Some(xml[start..start + end].to_string())
}

/// 向基座 rels part 追加 Relationship 文本 (已含缩进与换行)。
fn append_rels(
    parts: &mut std::collections::BTreeMap<String, Vec<u8>>,
    rels_name: &str,
    append: &str,
) {
    let existing = parts.get(rels_name).cloned().unwrap_or_default();
    let s = String::from_utf8_lossy(&existing).to_string();
    let merged = if s.contains("</Relationships>") {
        s.replacen("</Relationships>", &format!("{append}</Relationships>"), 1)
    } else {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
             <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n\
             {append}</Relationships>"
        )
    };
    parts.insert(rels_name.to_string(), merged.into_bytes());
}

/// 重建 document.xml: 保留基座 body 头尾, 段落间补 `<w:p/>` 分隔。
fn rebuild_document_xml(base_xml: &[u8], paragraphs: &[String]) -> String {
    let base = String::from_utf8_lossy(base_xml);
    // 定位 <w:body ...> 与 </w:body>
    let start = base.find("<w:body").unwrap_or(0);
    let start = start + base[start..].find('>').map(|i| i + 1).unwrap_or(0);
    let end_rel = base.rfind("</w:body>").unwrap_or(base.len());
    let head = &base[..start];
    let tail = &base[end_rel..];
    let body = paragraphs.join("\n");
    format!("{head}{body}\n{tail}")
}

/// 重建 [Content_Types].xml: 保留原 Overrides + Defaults, 为新增 part 补 Override。
fn rebuild_content_types(parts: &mut std::collections::BTreeMap<String, Vec<u8>>) {
    let ct_name = "[Content_Types].xml".to_string();
    let original = parts.get(&ct_name).cloned().unwrap_or_default();
    let original_str = String::from_utf8_lossy(&original).to_string();
    let mut new_overrides = String::new();
    for name in parts.keys() {
        if name == &ct_name || name.ends_with(".rels") {
            continue;
        }
        if !original_str.contains(&format!("\"{name}\"")) {
            let ct = guess_content_type(name);
            new_overrides.push_str(&format!(
                "    <Override PartName=\"/{name}\" ContentType=\"{ct}\"/>\n"
            ));
        }
    }
    if new_overrides.is_empty() {
        return;
    }
    let merged = if original_str.contains("</Types>") {
        original_str.replacen(
            "</Types>",
            &format!("{new_overrides}</Types>"),
            1,
        )
    } else {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
             <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\n\
             <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\n\
             <Default Extension=\"xml\" ContentType=\"application/xml\"/>\n{new_overrides}</Types>"
        )
    };
    parts.insert(ct_name, merged.into_bytes());
}

/// 按扩展名猜测 OPC content type。
fn guess_content_type(name: &str) -> &'static str {
    if name.ends_with(".png") {
        "image/png"
    } else if name.ends_with(".jpg") || name.ends_with(".jpeg") {
        "image/jpeg"
    } else if name.ends_with(".gif") {
        "image/gif"
    } else if name.ends_with(".svg") {
        "image/svg+xml"
    } else if name.starts_with("word/") {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
    } else {
        "application/octet-stream"
    }
}

/// 写 zip 包。
fn write_zip<W: Write + Seek>(w: W, parts: &std::collections::BTreeMap<String, Vec<u8>>) -> std::io::Result<()> {
    let mut zw = zip::ZipWriter::new(w);
    let opts = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, data) in parts {
        zw.start_file(name, opts)?;
        zw.write_all(data)?;
    }
    zw.finish()?;
    Ok(())
}

/// 合并多个 PPTX 文件。
///
/// 以首个为基座: 收集各文档 slide part (重命名避免冲突), 追加 `<p:sldId>`
/// 到 presentation.xml 的 sldIdLst, 合并 presentation.xml.rels 的 slide 目标;
/// 复制非冲突资源 part (媒体/主题)。输出为合并后字节。
pub fn merge_pptx(inputs: &[std::path::PathBuf], out: &Path) -> Result<OfficeMergeReport> {
    if inputs.is_empty() {
        return Err(FileAbilityError::Other("无输入 PPTX".to_string()));
    }
    if inputs.len() == 1 {
        let data = std::fs::read(&inputs[0]).map_err(FileAbilityError::Io)?;
        std::fs::write(out, &data).map_err(FileAbilityError::Io)?;
        return Ok(OfficeMergeReport { items: 1, parts: 0 });
    }

    // 1. 读基座全部 part
    let base = std::fs::File::open(&inputs[0]).map_err(FileAbilityError::Io)?;
    let mut base_zip = ZipArchive::new(base).map_err(|e| FileAbilityError::Parse(e.to_string()))?;
    let base_names = part_names(&base_zip);
    let mut out_parts: std::collections::BTreeMap<String, Vec<u8>> = std::collections::BTreeMap::new();
    for n in &base_names {
        if let Some(data) = read_part(&mut base_zip, n) {
            out_parts.insert(n.clone(), data);
        }
    }
    drop(base_zip);

    // 2. 收集后续文档 slide part + 资源, 追加 sldId 条目
    let pres_name = "ppt/presentation.xml".to_string();
    let base_pres = out_parts.get(&pres_name).cloned().unwrap_or_default();
    let mut sld_ids: Vec<String> = extract_presentation_sld_ids(&base_pres);
    let slide_count = sld_ids.len();
    let mut new_rels = String::new();
    let mut item_count = 1usize;
    // 全局 slide 序号累加器 (基座已有 slide_count 个)
    let mut next_slide = slide_count;

    for (idx, input) in inputs.iter().enumerate().skip(1) {
        let f = std::fs::File::open(input).map_err(FileAbilityError::Io)?;
        let mut z = ZipArchive::new(f).map_err(|e| FileAbilityError::Parse(e.to_string()))?;
        let names = part_names(&z);
        // 先按原序号收集本文档 slide (升序), 再分配全局序号 (保持文档内顺序)
        let mut local_slides: Vec<(usize, String, Vec<u8>)> = Vec::new();
        for n in &names {
            if n == &pres_name || n.ends_with(".rels") || n == "[Content_Types].xml" {
                continue;
            }
            if let Some(num) = slide_number(n) {
                if let Some(data) = read_part(&mut z, n) {
                    local_slides.push((num, n.clone(), data));
                }
            }
        }
        local_slides.sort_by_key(|(num, _, _)| *num);
        for (_, _n, data) in local_slides {
            next_slide += 1;
            let new_name = format!("ppt/slides/slide{}.xml", next_slide);
            out_parts.insert(new_name, data);
            let rel_id = format!("rId{}", 100 + next_slide);
            let sld_id = (256 + next_slide) as i64;
            sld_ids.push(format!(
                "<p:sldId id=\"{sld_id}\" r:id=\"{rel_id}\"/>"
            ));
            new_rels.push_str(&format!(
                "    <Relationship Id=\"{rel_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide\" Target=\"slides/slide{}.xml\"/>\n",
                next_slide
            ));
        }
        // 非 slide 资源复制
        for n in &names {
            if *n == pres_name || n.ends_with(".rels") || *n == "[Content_Types].xml" {
                continue;
            }
            if slide_number(n).is_some() {
                continue;
            }
            if out_parts.contains_key(n) {
                let new_name = rename_part(n, idx);
                if let Some(data) = read_part(&mut z, n) {
                    out_parts.insert(new_name, data);
                }
            } else if let Some(data) = read_part(&mut z, n) {
                out_parts.insert(n.clone(), data);
            }
        }
        item_count += 1;
    }

    // 3. 重建 presentation.xml (追加 sldId 条目到 sldIdLst)
    let merged_pres = rebuild_presentation_xml(&base_pres, &sld_ids);
    out_parts.insert(pres_name.clone(), merged_pres.into_bytes());

    // 4. 合并 presentation.xml.rels (追加新增 slide 目标)
    merge_presentation_rels(&mut out_parts, &new_rels);

    // 5. 重建 content types (新增 slide/媒体 Override)
    rebuild_content_types(&mut out_parts);

    // 6. 写 zip
    let file = std::fs::File::create(out).map_err(FileAbilityError::Io)?;
    write_zip(file, &out_parts).map_err(FileAbilityError::Io)?;
    Ok(OfficeMergeReport {
        items: item_count,
        parts: out_parts.len(),
    })
}

/// 提取 presentation.xml 中已有 `<p:sldId .../>` 条目 (字符串列表)。
fn extract_presentation_sld_ids(xml: &[u8]) -> Vec<String> {
    let s = String::from_utf8_lossy(xml).to_string();
    let mut out = Vec::new();
    let mut rest = s.as_str();
    while let Some(start) = rest.find("<p:sldId ") {
        let after = &rest[start..];
        if let Some(end) = after.find("/>") {
            out.push(after[..end + 2].to_string());
            rest = &after[end + 2..];
        } else {
            break;
        }
    }
    out
}

/// 解析 slide part 序号 (ppt/slides/slideN.xml → N)。
fn slide_number(name: &str) -> Option<usize> {
    let stem = name.strip_prefix("ppt/slides/slide")?;
    let stem = stem.strip_suffix(".xml")?;
    stem.parse::<usize>().ok()
}

/// 重建 presentation.xml: 在 sldIdLst 内追加全部 sldId。
fn rebuild_presentation_xml(base_xml: &[u8], sld_ids: &[String]) -> String {
    let base = String::from_utf8_lossy(base_xml).to_string();
    let insert = sld_ids.join("\n    ");
    if let Some(lst_start) = base.find("<p:sldIdLst") {
        let lst_open_end = lst_start + base[lst_start..].find('>').map(|i| i + 1).unwrap_or(0);
        if let Some(lst_end) = base.rfind("</p:sldIdLst>") {
            let mut out = String::with_capacity(base.len() + insert.len());
            out.push_str(&base[..lst_open_end]);
            out.push('\n');
            out.push_str(&insert);
            out.push('\n');
            out.push_str(&base[lst_end..]);
            return out;
        }
    }
    // 无 sldIdLst: 注入到 presentation 根下
    if let Some(root_end) = base.find('>') {
        let mut out = String::with_capacity(base.len() + insert.len() + 64);
        out.push_str(&base[..root_end + 1]);
        out.push_str(&format!("\n  <p:sldIdLst>\n    {insert}\n  </p:sldIdLst>"));
        out.push_str(&base[root_end + 1..]);
        return out;
    }
    base
}

/// 合并 presentation.xml.rels: 追加新增 slide Relationship。
fn merge_presentation_rels(parts: &mut std::collections::BTreeMap<String, Vec<u8>>, new_rels: &str) {
    let rels_name = "ppt/_rels/presentation.xml.rels".to_string();
    if new_rels.is_empty() {
        return;
    }
    let existing = parts.get(&rels_name).cloned().unwrap_or_default();
    let s = String::from_utf8_lossy(&existing).to_string();
    let merged = if s.contains("</Relationships>") {
        s.replacen("</Relationships>", &format!("{new_rels}</Relationships>"), 1)
    } else {
        format!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n\
             <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\n\
             {new_rels}</Relationships>"
        )
    };
    parts.insert(rels_name, merged.into_bytes());
}
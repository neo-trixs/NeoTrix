//! FileAbility 门面测试辅助 — 最小 DOCX/PPTX 构造夹具 (#[cfg(test)] 专用)。
//!
//! 纯搬移自 `nt_file_ability.rs` (God-file 拆分): 零行为变更,
//! 经门面 `pub use`/`pub(crate) use` 保持 `crate::neotrix::nt_file_ability::make_min_*` 路径不变。

/// 构造最小 DOCX (zip 包: [Content_Types].xml + _rels/.rels + word/document.xml)。
/// 测试辅助: 模块内 + 意识核心 dispatch 测试复用 (R-P42 复用, 不平行重造)。
#[cfg(test)]
pub fn make_min_docx(text: &str) -> Vec<u8> {
    use std::io::Write;
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    zw.start_file("[Content_Types].xml", opts)
        .expect("ct start");
    zw.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
    )
    .expect("ct write");
    zw.start_file("_rels/.rels", opts).expect("rels start");
    zw.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
    )
    .expect("rels write");
    zw.start_file("word/document.xml", opts).expect("doc start");
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p><w:r><w:t>{text}</w:t></w:r></w:p></w:body>
</w:document>"#
    );
    zw.write_all(doc.as_bytes()).expect("doc write");
    let buf = zw.finish().expect("zip finish");
    buf.into_inner()
}

/// 构造带内嵌图片的最小 DOCX: 段落含 `<w:drawing>` 引用 `rId2` → `media/image1.png`。
/// 用于验证合并时媒体 part 冲突重命名 + rels 的 rId 冲突重编号。
#[cfg(test)]
pub(crate) fn make_min_docx_with_media(text: &str, media_name: &str, media_bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    zw.start_file("[Content_Types].xml", opts).expect("ct");
    zw.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Default Extension="png" ContentType="image/png"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#,
    )
    .expect("ct write");
    zw.start_file("_rels/.rels", opts).expect("rels");
    zw.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#,
    )
    .expect("rels write");
    zw.start_file("word/document.xml", opts).expect("doc");
    let doc = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body>
<w:p><w:r><w:t>{text}</w:t></w:r></w:p>
<w:p><w:r><w:drawing xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:blip r:embed="rId2"/></w:drawing></w:r></w:p>
</w:body>
</w:document>"#
    );
    zw.write_all(doc.as_bytes()).expect("doc write");
    zw.start_file("word/_rels/document.xml.rels", opts).expect("doc rels");
    let rels = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="{media_name}"/>
</Relationships>"#
    );
    zw.write_all(rels.as_bytes()).expect("doc rels write");
    zw.start_file("word/media/image1.png", opts).expect("media");
    zw.write_all(media_bytes).expect("media write");
    let buf = zw.finish().expect("zip");
    buf.into_inner()
}

/// 构造最小 PPTX (zip 包: [Content_Types].xml + _rels/.rels + ppt/presentation.xml +
/// ppt/_rels/presentation.xml.rels + ppt/slides/slide1.xml)。测试辅助。
#[cfg(test)]
pub fn make_min_pptx(text: &str) -> Vec<u8> {
    use std::io::Write;
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opts = zip::write::SimpleFileOptions::default();
    zw.start_file("[Content_Types].xml", opts).expect("ct");
    zw.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
<Override PartName="/ppt/slides/slide1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
</Types>"#,
    )
    .expect("ct w");
    zw.start_file("_rels/.rels", opts).expect("rels");
    zw.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
</Relationships>"#,
    )
    .expect("rels w");
    zw.start_file("ppt/presentation.xml", opts).expect("pres");
    let pres = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <p:sldIdLst><p:sldId id="256" r:id="rId2"/></p:sldIdLst>
  <p:sldSz cx="9144000" cy="6858000"/>
</p:presentation>"#
    );
    zw.write_all(pres.as_bytes()).expect("pres w");
    zw.start_file("ppt/_rels/presentation.xml.rels", opts).expect("pres rels");
    zw.write_all(
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide1.xml"/>
</Relationships>"#,
    )
    .expect("pres rels w");
    zw.start_file("ppt/slides/slide1.xml", opts).expect("slide");
    let slide = format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
  <p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id="1" name=""/></p:nvGrpSpPr><p:grpSpPr/>
    <p:sp><p:nvSpPr><p:cNvPr id="2" name="t"/><p:nvPr/></p:nvSpPr><p:spPr/><p:txBody><a:p xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"><a:r><a:t>{text}</a:t></a:r></a:p></p:txBody></p:sp>
  </p:spTree></p:cSld>
</p:sld>"#
    );
    zw.write_all(slide.as_bytes()).expect("slide w");
    let buf = zw.finish().expect("zip");
    buf.into_inner()
}

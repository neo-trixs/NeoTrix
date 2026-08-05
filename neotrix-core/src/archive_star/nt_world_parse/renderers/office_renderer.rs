use std::io::Write;
use std::path::Path;
use zip::ZipWriter;
use zip::write::FileOptions;

// ─── Markdown → Document IR (simple inline parser) ───

#[derive(Debug)]
pub struct MdElement {
    pub kind: MdKind,
    pub alignment: Alignment,
}

fn elem(kind: MdKind) -> MdElement {
    MdElement { kind, alignment: Alignment::default() }
}

#[derive(Debug)]
pub enum MdKind {
    Heading(u8, Vec<MdInline>),
    Paragraph(Vec<MdInline>),
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    List { ordered: bool, items: Vec<(u8, Vec<MdInline>)> },
    TableOfContents,
    CodeBlock(String),
    ThematicBreak,
    PageBreak,
    SectionBreak,
    #[allow(dead_code)]
    BlankLine,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone)]
pub struct MdInline {
    text: String,
    bold: bool,
    italic: bool,
    strike: bool,
    underline: bool,
    superscript: bool,
    subscript: bool,
    code: bool,
    #[allow(dead_code)]
    link: Option<(String, String)>,
    image: bool,
}

impl MdInline {
    fn plain(text: &str) -> Self {
        Self { text: text.to_string(), bold: false, italic: false, strike: false, underline: false, superscript: false, subscript: false, code: false, link: None, image: false }
    }
    fn new(text: &str, opts: InlineOpts) -> Self {
        Self { text: text.to_string(), bold: opts.bold, italic: opts.italic, strike: opts.strike, underline: opts.underline, superscript: opts.superscript, subscript: opts.subscript, code: opts.code, link: None, image: false }
    }
    fn link(text: &str, url: String) -> Self {
        Self { text: text.to_string(), bold: false, italic: false, strike: false, underline: false, superscript: false, subscript: false, code: false, link: Some((url, text.to_string())), image: false }
    }
    fn image(url: String, alt: String) -> Self {
        Self { text: alt.clone(), bold: false, italic: false, strike: false, underline: false, superscript: false, subscript: false, code: false, link: Some((url, alt)), image: true }
    }
}

#[derive(Default)]
struct InlineOpts { bold: bool, italic: bool, strike: bool, underline: bool, superscript: bool, subscript: bool, code: bool }

fn parse_markdown(md: &str) -> Vec<MdElement> {
    let mut elements = Vec::new();
    let mut in_code_block = false;
    let mut code_block_text = String::new();
    let mut in_table = false;
    let mut table_headers: Vec<String> = Vec::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();
    let mut in_list = false;
    let mut ordered_list = false;
    let mut list_items: Vec<(u8, Vec<MdInline>)> = Vec::new();

    for line in md.lines() {
        let leading_spaces = line.len() - line.trim_start().len();
        let list_level = (leading_spaces / 2).min(8) as u8;
        let trimmed = line.trim();

        // Code block
        if trimmed.starts_with("```") {
            if in_code_block {
                elements.push(elem(MdKind::CodeBlock(code_block_text.clone())));
                code_block_text.clear();
                in_code_block = false;
            } else {
                if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                    &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                    elements.push(p);
                }
                in_code_block = true;
            }
            continue;
        }
        if in_code_block {
            code_block_text.push_str(line);
            code_block_text.push('\n');
            continue;
        }

        // Thematic break
        if trimmed.starts_with("---") || trimmed.starts_with("***") || trimmed.starts_with("___") {
            if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                elements.push(p);
            }
            elements.push(elem(MdKind::ThematicBreak));
            continue;
        }

        // Page break
        if trimmed.eq_ignore_ascii_case("<!--- pagebreak -->")
            || trimmed.eq_ignore_ascii_case("\\page")
            || trimmed.eq_ignore_ascii_case("<!--- pb -->")
        {
            if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                elements.push(p);
            }
            elements.push(elem(MdKind::PageBreak));
            continue;
        }

        // Section break
        if trimmed.eq_ignore_ascii_case("<!--- sectionbreak -->")
            || trimmed.eq_ignore_ascii_case("<!--- sb -->")
        {
            if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                elements.push(p);
            }
            elements.push(elem(MdKind::SectionBreak));
            continue;
        }

        // Table of contents marker
        if trimmed.eq_ignore_ascii_case("[toc]") {
            if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                elements.push(p);
            }
            elements.push(elem(MdKind::TableOfContents));

            continue;
        }

        // Blank line
        if trimmed.is_empty() {
            if in_table { continue; }
            if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                elements.push(p);
            }
            continue;
        }

        // Table row
        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            let cells: Vec<String> = trimmed[1..trimmed.len()-1]
                .split('|')
                .map(|c| c.trim().to_string())
                .collect();

            // Separator row
            if cells.iter().all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')) {
                continue;
            }

            if !in_table {
                if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                    &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                    elements.push(p);
                }
                table_headers = cells;
                table_rows.clear();
                in_table = true;
            } else {
                table_rows.push(cells);
            }
            continue;
        } else if in_table {
            if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                elements.push(p);
            }
        }

        // Heading
        if let Some(rest) = trimmed.strip_prefix("### ") {
            if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                elements.push(p);
            }
            elements.push(elem(MdKind::Heading(3, parse_inline(rest))));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                elements.push(p);
            }
            elements.push(elem(MdKind::Heading(2, parse_inline(rest))));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# ") {
            if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                elements.push(p);
            }
            elements.push(elem(MdKind::Heading(1, parse_inline(rest))));
            continue;
        }

        // List items
        if let Some(item) = trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
            if !in_list || ordered_list {
                if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                    &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                    elements.push(p);
                }
                in_list = true;
                ordered_list = false;
                list_items.clear();
            }
            list_items.push((list_level, parse_inline(item)));
            continue;
        }
        if trimmed.strip_prefix("1. ").is_some() {
            if !in_list || !ordered_list {
                if let Some(p) = finalize_paragraph(&mut in_table, &mut table_headers, &mut table_rows,
                    &mut in_list, &mut ordered_list, &mut list_items, &mut elements) {
                    elements.push(p);
                }
                in_list = true;
                ordered_list = true;
                list_items.clear();
            }
            // Strip the number prefix
            let rest = trimmed.split_once(' ').map(|x| x.1).unwrap_or(trimmed);
            list_items.push((list_level, parse_inline(rest)));
            continue;
        }
        let numbered = trimmed.starts_with(|c: char| c.is_ascii_digit()) && trimmed.contains(". ");
        if numbered && in_list && ordered_list {
            let rest = trimmed.split_once(' ').map(|x| x.1).unwrap_or(trimmed);
            list_items.push((list_level, parse_inline(rest)));
            continue;
        }

        // Regular paragraph
        if !in_list {
            let (inlines, alignment) = parse_paragraph_with_alignment(trimmed);
            elements.push(MdElement { kind: MdKind::Paragraph(inlines), alignment });
        } else {
            list_items.push((list_level, parse_inline(trimmed)));
        }
    }

    // Flush remaining
    if in_code_block && !code_block_text.is_empty() {
        elements.push(elem(MdKind::CodeBlock(code_block_text)));
    }
    if in_table {
        elements.push(MdElement {
            kind: MdKind::Table { headers: table_headers, rows: table_rows }, alignment: Alignment::default()
        });
    }
    if in_list && !list_items.is_empty() {
        elements.push(MdElement {
            kind: MdKind::List { ordered: ordered_list, items: list_items }, alignment: Alignment::default()
        });
    }

    elements
}

fn parse_paragraph_with_alignment(line: &str) -> (Vec<MdInline>, Alignment) {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("-> ") {
        (parse_inline(rest), Alignment::Center)
    } else if let Some(rest) = trimmed.strip_prefix("<- ") {
        (parse_inline(rest), Alignment::Right)
    } else if let Some(rest) = trimmed.strip_prefix(">< ") {
        (parse_inline(rest), Alignment::Justify)
    } else {
        (parse_inline(trimmed), Alignment::default())
    }
}

fn parse_inline(text: &str) -> Vec<MdInline> {
    let mut inlines = Vec::new();
    let mut chars = text.chars().peekable();
    let mut current = String::new();

    while let Some(c) = chars.next() {
        match c {
            '*' if chars.peek() == Some(&'*') => {
                chars.next(); // consume second *
                if !current.is_empty() {
                    inlines.push(MdInline::plain(&current));
                    current.clear();
                }
                // Read until **
                let mut bold_text = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '*' && chars.peek() == Some(&'*') {
                        chars.next();
                        break;
                    }
                    bold_text.push(ch);
                }
                inlines.push(MdInline::new(&bold_text, InlineOpts { bold: true, ..Default::default() }));
            }
            '~' if chars.peek() == Some(&'~') => {
                chars.next();
                if !current.is_empty() {
                    inlines.push(MdInline::plain(&current));
                    current.clear();
                }
                let mut strike_text = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '~' && chars.peek() == Some(&'~') {
                        chars.next();
                        break;
                    }
                    strike_text.push(ch);
                }
                inlines.push(MdInline::new(&strike_text, InlineOpts { strike: true, ..Default::default() }));
            }
            '+' if chars.peek() == Some(&'+') => {
                chars.next();
                if !current.is_empty() {
                    inlines.push(MdInline::plain(&current));
                    current.clear();
                }
                let mut ul_text = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '+' && chars.peek() == Some(&'+') {
                        chars.next();
                        break;
                    }
                    ul_text.push(ch);
                }
                inlines.push(MdInline::new(&ul_text, InlineOpts { underline: true, ..Default::default() }));
            }
            '^' => {
                if !current.is_empty() {
                    inlines.push(MdInline::plain(&current));
                    current.clear();
                }
                let mut sup_text = String::new();
                for ch in chars.by_ref() {
                    if ch == '^' { break; }
                    sup_text.push(ch);
                }
                inlines.push(MdInline::new(&sup_text, InlineOpts { superscript: true, ..Default::default() }));
            }
            '~' => {
                if !current.is_empty() {
                    inlines.push(MdInline::plain(&current));
                    current.clear();
                }
                let mut sub_text = String::new();
                for ch in chars.by_ref() {
                    if ch == '~' { break; }
                    sub_text.push(ch);
                }
                inlines.push(MdInline::new(&sub_text, InlineOpts { subscript: true, ..Default::default() }));
            }
            '*' => {
                if !current.is_empty() {
                    inlines.push(MdInline::plain(&current));
                    current.clear();
                }
                let mut italic_text = String::new();
                for ch in chars.by_ref() {
                    if ch == '*' { break; }
                    italic_text.push(ch);
                }
                inlines.push(MdInline::new(&italic_text, InlineOpts { italic: true, ..Default::default() }));
            }
            '`' => {
                if !current.is_empty() {
                    inlines.push(MdInline::plain(&current));
                    current.clear();
                }
                let mut code_text = String::new();
                for ch in chars.by_ref() {
                    if ch == '`' { break; }
                    code_text.push(ch);
                }
                inlines.push(MdInline::new(&code_text, InlineOpts { code: true, ..Default::default() }));
            }
            '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            '!' if chars.peek() == Some(&'[') => {
                // Image: ![alt](url)
                chars.next(); // consume [
                if !current.is_empty() {
                    inlines.push(MdInline::plain(&current));
                    current.clear();
                }
                let mut alt = String::new();
                for ch in chars.by_ref() {
                    if ch == ']' { break; }
                    alt.push(ch);
                }
                let mut url = String::new();
                if chars.next() == Some('(') {
                    for ch in chars.by_ref() {
                        if ch == ')' { break; }
                        url.push(ch);
                    }
                }
                inlines.push(MdInline::image(url, alt));
            }
            '[' => {
                // Link: [text](url)
                if !current.is_empty() {
                    inlines.push(MdInline::plain(&current));
                    current.clear();
                }
                let mut link_text = String::new();
                for ch in chars.by_ref() {
                    if ch == ']' { break; }
                    link_text.push(ch);
                }
                let mut url = String::new();
                if chars.next() == Some('(') {
                    for ch in chars.by_ref() {
                        if ch == ')' { break; }
                        url.push(ch);
                    }
                }
                inlines.push(MdInline::link(&link_text, url));
            }
            _ => current.push(c),
        }
    }

    if !current.is_empty() {
        inlines.push(MdInline::plain(&current));
    }

    inlines
}

fn finalize_paragraph(
    in_table: &mut bool,
    table_headers: &mut Vec<String>,
    table_rows: &mut Vec<Vec<String>>,
    in_list: &mut bool,
    ordered_list: &mut bool,
    list_items: &mut Vec<(u8, Vec<MdInline>)>,
    _elements: &mut Vec<MdElement>,
) -> Option<MdElement> {
    if *in_table {
        *in_table = false;
        let h = std::mem::take(table_headers);
        let r = std::mem::take(table_rows);
        return Some(elem(MdKind::Table { headers: h, rows: r }));
    }
    if *in_list && !list_items.is_empty() {
        *in_list = false;
        let o = *ordered_list;
        let items = std::mem::take(list_items);
        return Some(elem(MdKind::List { ordered: o, items }));
    }
    None
}

// ─── DOCX Writer ───

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn inline_to_docx_xml(inlines: &[MdInline], hyperlinks: &mut Vec<(String, String)>, images: &mut Vec<(String, String)>) -> String {
    let mut xml = String::new();
    for inc in inlines {
        let mut rpr = String::new();
        if inc.bold { rpr.push_str("<w:b/>"); }
        if inc.italic { rpr.push_str("<w:i/>"); }
        if inc.strike { rpr.push_str("<w:strike/>"); }
        if inc.underline { rpr.push_str(r#"<w:u w:val="single"/>"#); }
        if inc.superscript { rpr.push_str(r#"<w:vertAlign w:val="superscript"/>"#); }
        if inc.subscript { rpr.push_str(r#"<w:vertAlign w:val="subscript"/>"#); }
        if inc.code {
            rpr.push_str("<w:rFonts w:ascii=\"Courier New\" w:hAnsi=\"Courier New\"/>");
            rpr.push_str("<w:sz w:val=\"20\"/>");
            rpr.push_str("<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F2F2F2\"/>");
        }
        let text = escape_xml(&inc.text);
        let rpr_tag = if rpr.is_empty() {
            String::new()
        } else {
            format!("<w:rPr>{}</w:rPr>", rpr)
        };

        if inc.image {
            // Image: render as inline drawing
            if let Some((url, alt)) = &inc.link {
                let img_rid = format!("rIdImg{}", images.len() + 1);
                let alt_text = escape_xml(alt);
                images.push((img_rid.clone(), url.clone()));
                // Default 1-inch square (914400 EMUs)
                xml.push_str(&format!(
                    r#"<w:r><w:rPr/><w:drawing><wp:inline distT="0" distB="0" distL="0" distR="0">
<wp:extent cx="914400" cy="914400"/>
<wp:effectExtent l="0" t="0" r="0" b="0"/>
<wp:docPr id="{}" name="Image {}" descr="{}"/>
<wp:cNvGraphicFramePr><a:graphicFrameLocks xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" noChangeAspect="1"/></wp:cNvGraphicFramePr>
<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/picture">
<pic:pic xmlns:pic="http://schemas.openxmlformats.org/drawingml/2006/picture">
<pic:nvPicPr><pic:cNvPr id="{}" name="image{}.png" descr="{}"/><pic:cNvPicPr/></pic:nvPicPr>
<pic:blipFill><a:blip r:embed="{}"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill>
<pic:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></pic:spPr>
</pic:pic>
</a:graphicData>
</a:graphic>
</wp:inline></w:drawing></w:r>"#,
                    images.len(), images.len(), alt_text,
                    images.len(), images.len(), alt_text,
                    img_rid
                ));
            }
        } else if let Some((url, _label)) = &inc.link {
            let rid = format!("rIdH{}", hyperlinks.len() + 1);
            hyperlinks.push((rid.clone(), url.clone()));
            xml.push_str(&format!(
                r#"<w:hyperlink r:id="{}" w:history="1"><w:r>{}<w:t xml:space="preserve">{}</w:t></w:r></w:hyperlink>"#,
                rid, rpr_tag, text
            ));
        } else {
            xml.push_str(&format!("<w:r>{}<w:t xml:space=\"preserve\">{}</w:t></w:r>", rpr_tag, text));
        }
    }
    xml
}

fn build_doc_rels_xml(hyperlinks: &[(String, String)], images: &[(String, String)]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>"#,
    );
    for (rid, url) in hyperlinks {
        xml.push_str(&format!(
            r#"
<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{url}" TargetMode="External"/>"#,
        ));
    }
    for (i, (rid, _path)) in images.iter().enumerate() {
        let ext = std::path::Path::new(_path).extension().and_then(|e| e.to_str()).unwrap_or("png");
        xml.push_str(&format!(
            r#"
<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image{}.{}"/>"#,
            i + 1, ext
        ));
    }
    xml.push_str("\n</Relationships>");
    xml
}

fn build_styles_xml() -> String {
    String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:docDefaults>
<w:pPrDefault><w:pPr><w:spacing w:after="200" w:line="276" w:lineRule="auto"/></w:pPr></w:pPrDefault>
<w:rPrDefault><w:rPr><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr></w:rPrDefault>
</w:docDefaults>
<w:style w:type="paragraph" w:default="1" w:styleId="Normal">
<w:name w:val="Normal"/>
<w:rPr><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr>
</w:style>
<w:style w:type="paragraph" w:styleId="Heading1">
<w:name w:val="heading 1"/>
<w:basedOn w:val="Normal"/>
<w:next w:val="Normal"/>
<w:pPr><w:spacing w:before="360" w:after="120"/><w:outlineLvl w:val="0"/></w:pPr>
<w:rPr><w:b/><w:sz w:val="36"/><w:szCs w:val="36"/></w:rPr>
</w:style>
<w:style w:type="paragraph" w:styleId="Heading2">
<w:name w:val="heading 2"/>
<w:basedOn w:val="Normal"/>
<w:next w:val="Normal"/>
<w:pPr><w:spacing w:before="280" w:after="80"/><w:outlineLvl w:val="1"/></w:pPr>
<w:rPr><w:b/><w:sz w:val="30"/><w:szCs w:val="30"/></w:rPr>
</w:style>
<w:style w:type="paragraph" w:styleId="Heading3">
<w:name w:val="heading 3"/>
<w:basedOn w:val="Normal"/>
<w:next w:val="Normal"/>
<w:pPr><w:spacing w:before="200" w:after="60"/><w:outlineLvl w:val="2"/></w:pPr>
<w:rPr><w:b/><w:sz w:val="26"/><w:szCs w:val="26"/></w:rPr>
</w:style>
<w:style w:type="paragraph" w:styleId="TOC1">
<w:name w:val="toc 1"/>
<w:basedOn w:val="Normal"/>
<w:pPr><w:spacing w:before="120" w:after="60"/></w:pPr>
</w:style>
<w:style w:type="paragraph" w:styleId="TOC2">
<w:name w:val="toc 2"/>
<w:basedOn w:val="Normal"/>
<w:pPr><w:ind w:left="360"/><w:spacing w:before="60" w:after="60"/></w:pPr>
</w:style>
<w:style w:type="paragraph" w:styleId="TOC3">
<w:name w:val="toc 3"/>
<w:basedOn w:val="Normal"/>
<w:pPr><w:ind w:left="720"/><w:spacing w:before="60" w:after="60"/></w:pPr>
</w:style>
</w:styles>"#,
    )
}

fn docx_from_elements(elements: &[MdElement], output: &Path) -> Result<(), String> {
    let mut hyperlinks = Vec::new();
    let mut images: Vec<(String, String)> = Vec::new();

    // Collect per-section titles (splits at SectionBreak markers)
    let section_titles = collect_section_titles(elements);
    let section_count = section_titles.len().max(1);

    let document_xml = build_docx_body(elements, &mut hyperlinks, &mut images, &section_titles);
    let mut doc_rels_xml = build_doc_rels_xml(&hyperlinks, &images);

    // Add per-section header/footer relationships
    if let Some(pos) = doc_rels_xml.rfind("</Relationships>") {
        let mut hdr_ftr_rels = String::new();
        for i in 1..=section_count {
            hdr_ftr_rels.push_str(&format!(
                r#"
<Relationship Id="rIdHdr{i}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/header" Target="header{i}.xml"/>
<Relationship Id="rIdFtr{i}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer" Target="footer{i}.xml"/>"#,
            ));
        }
        doc_rels_xml.insert_str(pos, &hdr_ftr_rels);
    }

    let file = std::fs::File::create(output).map_err(|e| format!("create {:?}: {}", output, e))?;
    let mut zip = ZipWriter::new(file);
    let opts: FileOptions<'_, ()> = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.add_directory("_rels/", opts).map_err(|e| format!("_rels: {}", e))?;
    zip.add_directory("word/", opts).map_err(|e| format!("word: {}", e))?;
    zip.add_directory("word/_rels/", opts).map_err(|e| format!("word/_rels: {}", e))?;
    zip.add_directory("word/media/", opts).map_err(|e| format!("word/media: {}", e))?;
    zip.add_directory("docProps/", opts).map_err(|e| format!("docProps: {}", e))?;

    // Read image files and add to ZIP
    let mut image_types = String::new();
    for (i, (_rid, path)) in images.iter().enumerate() {
        let img_path = std::path::Path::new(path);
        let ext = img_path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
        let content_type = match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            "tiff" | "tif" => "image/tiff",
            "webp" => "image/webp",
            _ => "image/png",
        };
        let img_bytes = std::fs::read(img_path).map_err(|e| format!("read image {}: {}", path, e))?;
        let filename = format!("image{}.{}", i + 1, ext);
        zip.start_file(format!("word/media/{}", filename), opts).map_err(|e| format!("media {}: {}", filename, e))?;
        use std::io::Write;
        zip.write_all(&img_bytes).map_err(|e| format!("write image: {}", e))?;
        image_types.push_str(&format!(
            r#"<Override PartName="/word/media/{}" ContentType="{}"/>"#,
            filename, content_type
        ));
    }

    // Build content types with per-section headers/footers
    let mut hdr_ftr_types = String::new();
    for i in 1..=section_count {
        hdr_ftr_types.push_str(&format!(
            r#"<Override PartName="/word/header{i}.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.header+xml"/>
<Override PartName="/word/footer{i}.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml"/>
"#,
        ));
    }
    let content_types = CONTENT_TYPES_DOCX.replace(
        "</Types>",
        &format!("{}{}</Types>", hdr_ftr_types, image_types),
    );
    zip.start_file("[Content_Types].xml", opts).map_err(|e| format!("[Content_Types]: {}", e))?;
    write!(zip, "{}", content_types).map_err(|e| format!("write content types: {}", e))?;

    zip.start_file("_rels/.rels", opts).map_err(|e| format!("_rels: {}", e))?;
    write!(zip, "{}", RELS_DOCX).map_err(|e| format!("write rels: {}", e))?;

    zip.start_file("word/_rels/document.xml.rels", opts).map_err(|e| format!("word/_rels: {}", e))?;
    write!(zip, "{}", doc_rels_xml).map_err(|e| format!("write doc rels: {}", e))?;

    zip.start_file("docProps/core.xml", opts).map_err(|e| format!("core.xml: {}", e))?;
    write!(zip, "{}", CORE_PROPS).map_err(|e| format!("write core: {}", e))?;

    zip.start_file("word/document.xml", opts).map_err(|e| format!("document.xml: {}", e))?;
    write!(zip, "{}", document_xml).map_err(|e| format!("write doc: {}", e))?;

    zip.start_file("word/styles.xml", opts).map_err(|e| format!("styles.xml: {}", e))?;
    write!(zip, "{}", build_styles_xml()).map_err(|e| format!("write styles: {}", e))?;

    // Write per-section header and footer XML files
    for (i, title) in section_titles.iter().enumerate() {
        let idx = i + 1;
        let hdr_content = if title.is_empty() {
            build_header_xml("Document")
        } else {
            build_header_xml(title)
        };
        zip.start_file(format!("word/header{}.xml", idx), opts).map_err(|e| format!("header{}.xml: {}", idx, e))?;
        write!(zip, "{}", hdr_content).map_err(|e| format!("write header{}: {}", idx, e))?;

        zip.start_file(format!("word/footer{}.xml", idx), opts).map_err(|e| format!("footer{}.xml: {}", idx, e))?;
        write!(zip, "{}", build_footer_xml()).map_err(|e| format!("write footer{}: {}", idx, e))?;
    }

    zip.finish().map_err(|e| format!("zip finish: {}", e))?;
    Ok(())
}

fn alignment_xml(align: Alignment) -> &'static str {
    match align {
        Alignment::Left => "",
        Alignment::Center => r#"<w:jc w:val="center"/>"#,
        Alignment::Right => r#"<w:jc w:val="right"/>"#,
        Alignment::Justify => r#"<w:jc w:val="both"/>"#,
    }
}

fn build_docx_body(elements: &[MdElement], hyperlinks: &mut Vec<(String, String)>, images: &mut Vec<(String, String)>, section_titles: &[String]) -> String {
    let mut body = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<w:body>
"#
    );

    // Count sections to know which sectPr to emit
    let _section_count = section_titles.len().max(1);
    let mut section_idx = 0;

    for element in elements {
        match &element.kind {
            MdKind::Heading(level, inlines) => {
                let style = format!("Heading{}", level);
                let inline_xml = inline_to_docx_xml(inlines, hyperlinks, images);
                body.push_str(&format!("<w:p><w:pPr><w:pStyle w:val=\"{style}\"/><w:spacing w:before=\"360\" w:after=\"120\"/></w:pPr>{inline_xml}</w:p>\n"));
            }
            MdKind::Paragraph(inlines) => {
                let inline_xml = inline_to_docx_xml(inlines, hyperlinks, images);
                let align = alignment_xml(element.alignment);
                body.push_str(&format!("<w:p><w:pPr><w:spacing w:before=\"120\" w:after=\"120\" w:line=\"276\" w:lineRule=\"auto\"/>{}</w:pPr>{}</w:p>\n", align, inline_xml));
            }
            MdKind::Table { headers, rows } => {
                body.push_str("<w:tbl>\n");
                body.push_str(r#"<w:tblPr><w:tblStyle w:val="TableGrid"/><w:tblW w:w="5000" w:type="pct"/></w:tblPr>"#);
                if !headers.is_empty() && !headers.iter().all(|h| h.is_empty()) {
                    body.push_str("<w:tr>\n");
                    for h in headers {
                        body.push_str(&format!(
                            r#"<w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p><w:pPr><w:pStyle w:val="Heading3"/></w:pPr><w:r><w:rPr><w:b/><w:sz w:val="22"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>
"#, escape_xml(h)));
                    }
                    body.push_str("</w:tr>\n");
                }
                for row in rows {
                    body.push_str("<w:tr>\n");
                    for cell in row {
                        body.push_str(&format!(
                            r#"<w:tc><w:tcPr><w:tcW w:w="0" w:type="auto"/></w:tcPr><w:p><w:r><w:t>{}</w:t></w:r></w:p></w:tc>
"#, escape_xml(cell)));
                    }
                    body.push_str("</w:tr>\n");
                }
                body.push_str("</w:tbl>\n\n");
            }
            MdKind::List { ordered, items } => {
                let prefix_fn = if *ordered {
                    |i: usize| format!("{}. ", i + 1)
                } else {
                    |_: usize| "• ".to_string()
                };
                for (i, (level, item_inlines)) in items.iter().enumerate() {
                    let mut prefix_inlines = parse_inline(&prefix_fn(i));
                    let mut combined = Vec::new();
                    combined.append(&mut prefix_inlines);
                    combined.extend(item_inlines.iter().cloned());
                    let inline_xml = inline_to_docx_xml(&combined, hyperlinks, images);
                    let indent = 720 + *level as i32 * 360;
                    body.push_str(&format!(
                        r#"<w:p><w:pPr><w:spacing w:before="60" w:after="60"/><w:ind w:left="{}" w:hanging="360"/></w:pPr>{}</w:p>
"#, indent, inline_xml));
                }
            }
            MdKind::CodeBlock(code) => {
                let text = escape_xml(code);
                body.push_str(&format!(
                    r#"<w:p><w:pPr><w:shd w:val="clear" w:color="auto" w:fill="F2F2F2"/><w:ind w:left="360"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Courier New" w:hAnsi="Courier New"/><w:sz w:val="18"/></w:rPr><w:t xml:space="preserve">{text}</w:t></w:r></w:p>
"#));
            }
            MdKind::TableOfContents => {
                body.push_str(
                    r#"<w:p><w:pPr><w:pStyle w:val="TOC1"/></w:pPr><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> TOC \o "1-3" \h \z \u </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:rPr><w:sz w:val="20"/></w:rPr><w:t>Update Table of Contents</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>
"#);
            }
            MdKind::ThematicBreak => {
                body.push_str(r#"<w:p><w:pPr><w:pBdr><w:bottom w:val="single" w:sz="6" w:space="1" w:color="999999"/></w:pPr></w:p>"#);
            }
            MdKind::BlankLine => {
                body.push_str("<w:p><w:r><w:t></w:t></w:r></w:p>\n");
            }
            MdKind::PageBreak => {
                body.push_str(r#"<w:p><w:pPr><w:pageBreakBefore/></w:pPr><w:r><w:t></w:t></w:r></w:p>"#);
            }
            MdKind::SectionBreak => {
                // Emit sectPr to close current section, then increment
                let sect_idx = section_idx + 1; // Sections are 1-indexed in DOCX
                body.push_str(&format!(
                    r#"<w:p><w:pPr><w:sectPr>
<w:pgSz w:w="12240" w:h="15840"/>
<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="708" w:footer="708" w:gutter="0"/>
<w:headerReference w:type="default" r:id="rIdHdr{}"/>
<w:footerReference w:type="default" r:id="rIdFtr{}"/>
</w:sectPr></w:pPr></w:p>
"#, sect_idx, sect_idx));
                section_idx += 1;
            }
        }
    }

    // Final section properties for the last section
    let last_sect_idx = section_idx + 1;
    body.push_str(&format!(
        r#"<w:sectPr>
<w:pgSz w:w="12240" w:h="15840"/>
<w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440" w:header="708" w:footer="708" w:gutter="0"/>
<w:headerReference w:type="default" r:id="rIdHdr{}"/>
<w:footerReference w:type="default" r:id="rIdFtr{}"/>
</w:sectPr>
"#, last_sect_idx, last_sect_idx));

    body.push_str("</w:body>\n</w:document>\n");
    body
}

/// Collect section titles from elements, splitting at SectionBreak markers.
/// The first section title comes from the first H1/H2 before the first SectionBreak.
/// Subsequent section titles come from the first H1/H2 after each SectionBreak.
fn collect_section_titles(elements: &[MdElement]) -> Vec<String> {
    let mut titles = Vec::new();
    let mut current_title = String::new();

    for element in elements {
        match &element.kind {
            MdKind::SectionBreak => {
                titles.push(current_title.clone());
                current_title = String::new();
            }
            MdKind::Heading(level, inlines) if *level <= 2 => {
                if current_title.is_empty() {
                    current_title = inlines.iter().map(|i| i.text.as_str()).collect::<Vec<_>>().join(" ");
                }
            }
            _ => {}
        }
    }
    // Push the last section's title
    titles.push(current_title);
    titles
}

fn build_header_xml(title: &str) -> String {
    let text = escape_xml(title);
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:hdr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:p><w:pPr><w:pStyle w:val="Normal"/></w:pPr><w:r><w:t>{text}</w:t></w:r></w:p>
</w:hdr>"#,
    )
}

fn build_footer_xml() -> String {
    String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:ftr xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:p><w:pPr><w:pStyle w:val="Normal"/><w:jc w:val="center"/></w:pPr><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>1</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>
</w:ftr>"#,
    )
}

// ─── PPTX Writer ───

fn build_pptx_content_types(slide_count: usize) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/ppt/presentation.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"/>
<Override PartName="/ppt/slideMasters/slideMaster1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml"/>
<Override PartName="/ppt/slideLayouts/slideLayout1.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml"/>
"#,
    );
    for i in 1..=slide_count {
        xml.push_str(&format!(
            r#"<Override PartName="/ppt/slides/slide{}.xml" ContentType="application/vnd.openxmlformats-officedocument.presentationml.slide+xml"/>
"#, i));
    }
    xml.push_str(r#"<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#);
    xml
}

fn build_pptx_presentation_rels(slide_count: usize) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rIdMaster" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="slideMasters/slideMaster1.xml"/>
<Relationship Id="rIdLayout" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="slideLayouts/slideLayout1.xml"/>
"#,
    );
    for i in 1..=slide_count {
        xml.push_str(&format!(
            r#"<Relationship Id="rId{}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide" Target="slides/slide{}.xml"/>
"#, i + 1, i));
    }
    xml.push_str("</Relationships>\n");
    xml
}

fn build_pptx_presentation(slide_count: usize) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<p:sldMasterIdLst><p:sldMasterId id="2147483648" r:id="rIdMaster"/></p:sldMasterIdLst>
<p:sldIdLst>
"#,
    );
    for i in 0..slide_count {
        xml.push_str(&format!(r#"<p:sldId id="{}" r:id="rId{}"/>
"#, 256 + i, i + 1));
    }
    xml.push_str(
        r#"</p:sldIdLst>
<p:sldSz cx="9144000" cy="6858000"/>
<p:notesSz cx="6858000" cy="9144000"/>
</p:presentation>"#,
    );
    xml
}

fn build_pptx_table_xml(headers: &[String], rows: &[Vec<String>], _hyperlinks: &mut Vec<(String, String)>) -> String {
    let col_count = headers.len().max(rows.first().map(|r| r.len()).unwrap_or(0));
    let col_width = 8229600 / col_count.max(1);
    let mut xml = String::new();

    // grid column definitions
    xml.push_str("<a:tblGrid>");
    for _ in 0..col_count {
        xml.push_str(&format!("<a:gridCol w=\"{}\"/>", col_width));
    }
    xml.push_str("</a:tblGrid>");

    // header row
    if !headers.is_empty() {
        xml.push_str("<a:tr h=\"370840\">");
        for h in headers {
            let text = escape_xml(h);
            xml.push_str(&format!(
                r#"<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" b="1"/><a:t>{text}</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc>"#
            ));
        }
        xml.push_str("</a:tr>");
    }

    // data rows
    for row in rows {
        xml.push_str("<a:tr h=\"370840\">");
        for cell in row {
            let text = escape_xml(cell);
            xml.push_str(&format!(
                r#"<a:tc><a:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>{text}</a:t></a:r></a:p></a:txBody><a:tcPr/></a:tc>"#
            ));
        }
        xml.push_str("</a:tr>");
    }
    xml
}

fn build_pptx_lst_style() -> String {
    let mut xml = String::from("<a:lstStyle>");
    for lvl in 1..=9 {
        let indent = lvl * 285600; // ~1/4 inch per level in EMUs
        xml.push_str(&format!(
            r#"<a:lvl{}pPr marL="{}" indent="{}" algn="l"/>
"#,
            lvl, indent, -285600i32
        ));
    }
    xml.push_str("</a:lstStyle>");
    xml
}

fn build_pptx_slide_xml(title: &str, items: &[SlideItem], hyperlinks: &mut Vec<(String, String)>, images: &mut Vec<(String, String)>) -> String {
    let title_text = escape_xml(title);

    // build body shapes: text boxes + tables + images
    let mut body_shapes = String::new();
    let mut y_offset = 2880000i32; // start below title
    let shape_height = 360000;
    let body_width = 8229600i32;

    for item in items {
        match item {
            SlideItem::Text(inlines, level) => {
                let body_xml = inline_to_pptx_xml(inlines, hyperlinks);
                let ppr = if *level > 0 {
                    format!(r#"<a:pPr lvl="{}"/>"#, level)
                } else {
                    String::new()
                };
                let lst_style = build_pptx_lst_style();
                body_shapes.push_str(&format!(
                    r#"<p:sp>
<p:nvSpPr><p:cNvPr id="{}" name="Body"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="457200" y="{}"/><a:ext cx="{}" cy="{}"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></a:spPr>
<p:txBody><a:bodyPr/>{lst_style}<a:p>{}{}</a:p></p:txBody>
</p:sp>
"#,
                    hyperlinks.len() as u32 + 4,
                    y_offset,
                    body_width,
                    shape_height,
                    ppr,
                    body_xml
                ));
            }
            SlideItem::Image { path, alt } => {
                let img_rid = format!("rIdImg{}", images.len() + 1);
                images.push((img_rid.clone(), path.clone()));
                let alt_text = escape_xml(alt);
                body_shapes.push_str(&format!(
                    r#"<p:pic>
<p:nvPicPr><p:cNvPr id="{}" name="{}" descr="{}"/><p:cNvPicPr><a:picLocks noChangeAspect="1"/></p:cNvPicPr><p:nvPr/></p:nvPicPr>
<p:blipFill><a:blip r:embed="{}"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>
<p:spPr><a:xfrm><a:off x="457200" y="{}"/><a:ext cx="4114800" cy="3086100"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></a:spPr>
</p:pic>
"#,
                    hyperlinks.len() as u32 + 10,
                    alt_text, alt_text,
                    img_rid,
                    y_offset,
                ));
            }
            SlideItem::Table { headers, rows } => {
                let table_xml = build_pptx_table_xml(headers, rows, hyperlinks);
                let table_height = (headers.len() as i32 + rows.len() as i32) * 370840;
                body_shapes.push_str(&format!(
                    r#"<p:graphicFrame>
<p:nvGraphicFramePr><p:cNvPr id="{}" name="Table"/><p:cNvGraphicFramePr/><p:nvPr/></p:nvGraphicFramePr>
<p:xfrm><a:off x="457200" y="{}"/><a:ext cx="{}" cy="{}"/></a:xfrm>
<a:graphic>
<a:graphicData uri="http://schemas.openxmlformats.org/drawingml/2006/table">
<a:tbl>
<a:tblPr firstRow="1" bandRow="1"/>
{}
</a:tbl>
</a:graphicData>
</a:graphic>
</p:graphicFrame>
"#,
                    hyperlinks.len() as u32 + 4,
                    y_offset,
                    body_width,
                    table_height,
                    table_xml
                ));
                y_offset += table_height + 72000;
                continue;
            }
        }
        y_offset += shape_height + 72000;
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<p:cSld>
<p:spTree>
<p:nvGrpSpPr><p:nvPr/><p:cNvPr id="1" name=""/><p:nvGrpSpPr/></p:nvGrpSpPr>
<p:grpSpPr/>
<p:sp>
<p:nvSpPr><p:cNvPr id="2" name="Title"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
<p:spPr><a:xfrm><a:off x="457200" y="2743200"/><a:ext cx="8229600" cy="1371600"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></a:spPr>
<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="4400" b="1"/><a:t>{title_text}</a:t></a:r></a:p></p:txBody>
</p:sp>
{body_shapes}
</p:spTree>
</p:cSld>
</p:sld>"#,
    )
}

fn inline_to_pptx_xml(inlines: &[MdInline], hyperlinks: &mut Vec<(String, String)>) -> String {
    let mut xml = String::new();
    for inc in inlines {
        let mut rpr = String::new();
        if inc.bold { rpr.push_str(r#"b="1" "#); }
        if inc.italic { rpr.push_str(r#"i="1" "#); }
        if inc.strike { rpr.push_str(r#"strike="sngStrike" "#); }
        if inc.underline { rpr.push_str(r#"u="sng" "#); }
        if inc.superscript || inc.subscript {
            rpr.push_str(&format!(r#"baseline="{}" "#, if inc.superscript { "30000" } else { "-25000" }));
        }
        if inc.code {
            rpr.push_str(r#"sz="1800" "#);
        }
        if let Some((url, _label)) = &inc.link {
            let rid = format!("rIdH{}", hyperlinks.len() + 1);
            hyperlinks.push((rid.clone(), url.clone()));
            rpr.push_str(&format!(r#"<a:hlinkClick r:id="{}"/>"#, rid));
            rpr.push(' ');
        }
        let text = escape_xml(&inc.text);
        let rpr_attr = if rpr.is_empty() { String::new() }
            else { format!(" {}", rpr.trim_end()) };
        xml.push_str(&format!("<a:r><a:rPr lang=\"en-US\"{rpr_attr}/><a:t>{text}</a:t></a:r>"));
    }
    if xml.is_empty() {
        xml = r#"<a:r><a:rPr lang="en-US"/><a:t> </a:t></a:r>"#.to_string();
    }
    xml
}

#[derive(Debug)]
enum SlideItem {
    Text(Vec<MdInline>, u8),
    Table { headers: Vec<String>, rows: Vec<Vec<String>> },
    Image { path: String, alt: String },
}

fn pptx_from_elements(elements: &[MdElement], output: &Path) -> Result<(), String> {
    // Extract slides: either explicit ## Slide N headings, or group content by H2
    let mut slides: Vec<(String, Vec<SlideItem>)> = Vec::new();
    let mut current_title = String::new();
    let mut current_items: Vec<SlideItem> = Vec::new();

    for element in elements {
        match &element.kind {
            MdKind::Heading(2, inlines) => {
                let title_text = inlines.iter().map(|i| i.text.as_str()).collect::<Vec<_>>().join(" ");
                if !current_title.is_empty() || !current_items.is_empty() {
                    slides.push((current_title.clone(), std::mem::take(&mut current_items)));
                }
                current_title = title_text;
            }
            MdKind::Heading(_, inlines) => {
                let t = inlines.iter().map(|i| i.text.as_str()).collect::<Vec<_>>().join(" ");
                if current_title.is_empty() {
                    current_title = t;
                } else {
                    current_items.push(SlideItem::Text(vec![MdInline::plain(&t)], 0));
                }
            }
            MdKind::Paragraph(inlines) => {
                // Extract images from inline content
                let text_inlines: Vec<MdInline> = inlines.iter().filter(|i| !i.image).cloned().collect();
                let image_inlines: Vec<&MdInline> = inlines.iter().filter(|i| i.image).collect();
                if !text_inlines.is_empty() {
                    current_items.push(SlideItem::Text(text_inlines, 0));
                }
                for img in image_inlines {
                    if let Some((url, alt)) = &img.link {
                        current_items.push(SlideItem::Image { path: url.clone(), alt: alt.clone() });
                    }
                }
            }
            MdKind::List { items, .. } => {
                for (level, item_inlines) in items {
                    let mut text = Vec::new();
                    text.push(MdInline::plain("• "));
                    text.extend(item_inlines.iter().cloned());
                    current_items.push(SlideItem::Text(text, *level));
                }
            }
            MdKind::CodeBlock(code) => {
                current_items.push(SlideItem::Text(vec![MdInline::new(code, InlineOpts { code: true, ..Default::default() })], 0));
            }
            MdKind::Table { headers, rows } => {
                current_items.push(SlideItem::Table { headers: headers.clone(), rows: rows.clone() });
            }
            _ => {}
        }
    }
    if !current_title.is_empty() || !current_items.is_empty() {
        slides.push((current_title, current_items));
    }

    // If no slides created, make a default one
    if slides.is_empty() {
        slides.push(("Document".to_string(), vec![SlideItem::Text(vec![MdInline::plain("Content")], 0)]));
    }

    let file = std::fs::File::create(output).map_err(|e| format!("create {:?}: {}", output, e))?;
    let mut zip = ZipWriter::new(file);
    let opts: FileOptions<'_, ()> = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.add_directory("_rels/", opts).map_err(|e| format!("_rels: {}", e))?;
    zip.add_directory("ppt/", opts).map_err(|e| format!("ppt: {}", e))?;
    zip.add_directory("ppt/slides/", opts).map_err(|e| format!("ppt/slides: {}", e))?;
    zip.add_directory("ppt/slideLayouts/", opts).map_err(|e| format!("ppt/slideLayouts: {}", e))?;
    zip.add_directory("ppt/slideMasters/", opts).map_err(|e| format!("ppt/slideMasters: {}", e))?;
    zip.add_directory("ppt/_rels/", opts).map_err(|e| format!("ppt/_rels: {}", e))?;
    zip.add_directory("ppt/slides/_rels/", opts).map_err(|e| format!("ppt/slides/_rels: {}", e))?;
    zip.add_directory("ppt/slideMasters/_rels/", opts).map_err(|e| format!("ppt/slideMasters/_rels: {}", e))?;
    zip.add_directory("ppt/slideLayouts/_rels/", opts).map_err(|e| format!("ppt/slideLayouts/_rels: {}", e))?;
    zip.add_directory("ppt/media/", opts).map_err(|e| format!("ppt/media: {}", e))?;
    zip.add_directory("docProps/", opts).map_err(|e| format!("docProps: {}", e))?;

    zip.start_file("_rels/.rels", opts).map_err(|e| format!("_rels: {}", e))?;
    write!(zip, "{}", PPTX_RELS).map_err(|e| format!("write rels: {}", e))?;

    zip.start_file("ppt/_rels/presentation.xml.rels", opts).map_err(|e| format!("ppt/_rels: {}", e))?;
    write!(zip, "{}", build_pptx_presentation_rels(slides.len())).map_err(|e| format!("write pres rels: {}", e))?;

    zip.start_file("ppt/presentation.xml", opts).map_err(|e| format!("presentation.xml: {}", e))?;
    write!(zip, "{}", build_pptx_presentation(slides.len())).map_err(|e| format!("write pres: {}", e))?;

    zip.start_file("ppt/slideMasters/slideMaster1.xml", opts).map_err(|e| format!("slideMaster: {}", e))?;
    write!(zip, "{}", SLIDE_MASTER).map_err(|e| format!("write slideMaster: {}", e))?;

    zip.start_file("ppt/slideMasters/_rels/slideMaster1.xml.rels", opts).map_err(|e| format!("slideMaster rels: {}", e))?;
    write!(zip, "{}", SLIDE_MASTER_RELS).map_err(|e| format!("write slideMaster rels: {}", e))?;

    zip.start_file("ppt/slideLayouts/slideLayout1.xml", opts).map_err(|e| format!("slideLayout: {}", e))?;
    write!(zip, "{}", SLIDE_LAYOUT).map_err(|e| format!("write slideLayout: {}", e))?;

    zip.start_file("ppt/slideLayouts/_rels/slideLayout1.xml.rels", opts).map_err(|e| format!("slideLayout rels: {}", e))?;
    write!(zip, "{}", SLIDE_LAYOUT_RELS).map_err(|e| format!("write slideLayout rels: {}", e))?;

    let mut all_images: Vec<(String, String)> = Vec::new(); // (rId, path) global

    for (i, (title, body)) in slides.iter().enumerate() {
        let slide_num = i + 1;
        let mut slide_hyperlinks: Vec<(String, String)> = Vec::new();
        let mut slide_images: Vec<(String, String)> = Vec::new(); // (local_rId, path)
        let entry = format!("ppt/slides/slide{}.xml", slide_num);
        zip.start_file(&entry, opts).map_err(|e| format!("{}: {}", entry, e))?;
        write!(zip, "{}", build_pptx_slide_xml(title, body, &mut slide_hyperlinks, &mut slide_images)).map_err(|e| format!("write slide {}: {}", slide_num, e))?;

        // Assign global IDs and write image files
        let mut global_image_mappings: Vec<(String, usize, String)> = Vec::new(); // (local_rId, global_idx, ext)
        for (_rid, path) in slide_images.iter() {
            let img_path = std::path::Path::new(path);
            let img_bytes = std::fs::read(img_path).map_err(|e| format!("read image {}: {}", path, e))?;
            let ext = img_path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
            let global_idx = all_images.len() + 1;
            let filename = format!("image{}.{}", global_idx, ext);
            zip.start_file(format!("ppt/media/{}", filename), opts).map_err(|e| format!("media {}: {}", filename, e))?;
            zip.write_all(&img_bytes).map_err(|e| format!("write image: {}", e))?;
            global_image_mappings.push((_rid.clone(), global_idx, ext.clone()));
            all_images.push((_rid.clone(), path.clone()));
        }

        let rel_entry = format!("ppt/slides/_rels/slide{}.xml.rels", slide_num);
        zip.start_file(&rel_entry, opts).map_err(|e| format!("{}: {}", rel_entry, e))?;
        write!(zip, "{}", build_slide_rels_xml(&slide_hyperlinks, &global_image_mappings, slide_num)).map_err(|e| format!("write slide rels: {}: {}", slide_num, e))?;
    }

    // Add image content types
    let mut image_overrides = String::new();
    for (i, (_rid, path)) in all_images.iter().enumerate() {
        let img_path = std::path::Path::new(path);
        let ext = img_path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
        let content_type = match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            _ => "image/png",
        };
        image_overrides.push_str(&format!(
            r#"<Override PartName="/ppt/media/image{}.{}" ContentType="{}"/>"#,
            i + 1, ext, content_type
        ));
    }
    let mut pptx_ct = build_pptx_content_types(slides.len());
    pptx_ct = pptx_ct.replace("</Types>", &format!("{}\n</Types>", image_overrides));
    zip.start_file("[Content_Types].xml", opts).map_err(|e| format!("[Content_Types]: {}", e))?;
    write!(zip, "{}", pptx_ct).map_err(|e| format!("write content types: {}", e))?;
    zip.start_file("docProps/core.xml", opts).map_err(|e| format!("core.xml: {}", e))?;
    write!(zip, "{}", CORE_PROPS).map_err(|e| format!("write core: {}", e))?;

    zip.finish().map_err(|e| format!("zip finish: {}", e))?;
    Ok(())
}

// ─── XLSX Writer ───

fn xlsx_from_elements(elements: &[MdElement], output: &Path) -> Result<(), String> {
    // Extract table elements, or treat paragraphs as single-cell rows
    let mut all_headers: Vec<String> = Vec::new();
    let mut all_rows: Vec<Vec<String>> = Vec::new();

    for element in elements {
        match &element.kind {
            MdKind::Table { headers, rows } => {
                all_headers = headers.clone();
                all_rows = rows.clone();
            }
            MdKind::Heading(_, inlines) => {
                let t = inlines.iter().map(|i| i.text.as_str()).collect::<Vec<_>>().join(" ");
                if all_headers.is_empty() && all_rows.is_empty() {
                    all_headers.push(t);
                } else {
                    all_rows.push(vec![t]);
                }
            }
            MdKind::Paragraph(inlines) => {
                let t = inlines.iter().map(|i| i.text.as_str()).collect::<Vec<_>>().join(" ");
                if all_rows.is_empty() && all_headers.is_empty() {
                    all_headers.push(t);
                } else {
                    all_rows.push(vec![t]);
                }
            }
            MdKind::List { items, .. } => {
                for (level, item_inlines) in items {
                    let prefix = "\t".repeat(*level as usize);
                    let line: String = prefix + &item_inlines.iter().map(|i| i.text.as_str()).collect::<String>();
                    all_rows.push(vec![line]);
                }
            }
            MdKind::CodeBlock(code) => {
                for line in code.lines() {
                    all_rows.push(vec![line.to_string()]);
                }
            }
            _ => {}
        }
    }

    if all_headers.is_empty() && all_rows.is_empty() {
        all_headers.push("Content".to_string());
    }

    let file = std::fs::File::create(output).map_err(|e| format!("create {:?}: {}", output, e))?;
    let mut zip = ZipWriter::new(file);
    let opts: FileOptions<'_, ()> = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.add_directory("_rels/", opts).map_err(|e| format!("_rels: {}", e))?;
    zip.add_directory("xl/", opts).map_err(|e| format!("xl: {}", e))?;
    zip.add_directory("xl/worksheets/", opts).map_err(|e| format!("xl/worksheets: {}", e))?;
    zip.add_directory("xl/_rels/", opts).map_err(|e| format!("xl/_rels: {}", e))?;
    zip.add_directory("xl/worksheets/_rels/", opts).map_err(|e| format!("xl/worksheets/_rels: {}", e))?;
    zip.add_directory("xl/theme/", opts).map_err(|e| format!("xl/theme: {}", e))?;
    zip.add_directory("docProps/", opts).map_err(|e| format!("docProps: {}", e))?;

    zip.start_file("[Content_Types].xml", opts).map_err(|e| format!("[Content_Types]: {}", e))?;
    write!(zip, "{}", XLSX_CONTENT_TYPES).map_err(|e| format!("write content types: {}", e))?;

    zip.start_file("_rels/.rels", opts).map_err(|e| format!("_rels: {}", e))?;
    write!(zip, "{}", XLSX_RELS).map_err(|e| format!("write rels: {}", e))?;

    zip.start_file("xl/_rels/workbook.xml.rels", opts).map_err(|e| format!("xl/_rels: {}", e))?;
    write!(zip, "{}", XL_WORKBOOK_RELS).map_err(|e| format!("write xl rels: {}", e))?;

    zip.start_file("xl/workbook.xml", opts).map_err(|e| format!("workbook: {}", e))?;
    write!(zip, "{}", XL_WORKBOOK).map_err(|e| format!("write workbook: {}", e))?;

    zip.start_file("xl/theme/theme1.xml", opts).map_err(|e| format!("theme: {}", e))?;
    write!(zip, "{}", XLSX_THEME).map_err(|e| format!("write theme: {}", e))?;

    let styles = build_xlsx_styles();
    zip.start_file("xl/styles.xml", opts).map_err(|e| format!("styles: {}", e))?;
    write!(zip, "{}", styles).map_err(|e| format!("write styles: {}", e))?;

    let (sheet, hyperlinks) = build_sheet_xml(&all_headers, &all_rows);
    zip.start_file("xl/worksheets/sheet1.xml", opts).map_err(|e| format!("sheet1: {}", e))?;
    write!(zip, "{}", sheet).map_err(|e| format!("write sheet: {}", e))?;

    // Build dynamic sheet rels with hyperlink relationships
    let sheet_rels = build_sheet_rels_xml(&hyperlinks);
    zip.start_file("xl/worksheets/_rels/sheet1.xml.rels", opts).map_err(|e| format!("sheet rels: {}", e))?;
    write!(zip, "{}", sheet_rels).map_err(|e| format!("write sheet rels: {}", e))?;

    zip.start_file("docProps/core.xml", opts).map_err(|e| format!("core.xml: {}", e))?;
    write!(zip, "{}", CORE_PROPS).map_err(|e| format!("write core: {}", e))?;

    zip.finish().map_err(|e| format!("zip finish: {}", e))?;
    Ok(())
}

fn build_sheet_rels_xml(hyperlinks: &[(String, String)]) -> String {
    if hyperlinks.is_empty() {
        return SHEET_RELS_EMPTY.to_string();
    }
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
"#,
    );
    for (i, (_cell_ref, url)) in hyperlinks.iter().enumerate() {
        let rid = format!("rId{}", i + 1);
        xml.push_str(&format!(
            r#"<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{url}" TargetMode="External"/>"#,
        ));
        xml.push('\n');
    }
    xml.push_str("</Relationships>");
    xml
}

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

fn build_xlsx_styles() -> String {
    String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<fonts count="2">
<font><sz val="11"/><name val="Calibri"/></font>
<font><b/><sz val="11"/><color rgb="FFFFFFFF"/><name val="Calibri"/></font>
</fonts>
<fills count="3">
<fill><patternFill patternType="none"/></fill>
<fill><patternFill patternType="gray125"/></fill>
<fill><patternFill patternType="solid"><fgColor rgb="FF4472C4"/></patternFill></fill>
</fills>
<border count="1"><border><left/><right/><top/><bottom/><diagonal/></border></border>
<cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
<cellXfs count="2">
<xf numFmtId="0" fontId="0" fillId="0" borderId="0"/>
<xf numFmtId="0" fontId="0" fillId="2" borderId="0" applyFont="1" applyFill="1">
<alignment horizontal="center"/>
</xf>
</cellXfs>
</styleSheet>"#,
    )
}

fn build_sheet_xml(headers: &[String], rows: &[Vec<String>]) -> (String, Vec<(String, String)>) {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"
           xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheetData>
"#,
    );
    let mut hyperlinks: Vec<(String, String)> = Vec::new();

    // Extract hyperlink from a cell string: "[text](url)" → text
    fn extract_cell_content_and_url(s: &str) -> (String, Option<String>) {
        if let Some(rest) = s.strip_prefix('[') {
            if let Some(end_bracket) = rest.find(']') {
                let text = &rest[..end_bracket];
                let after = &rest[end_bracket + 1..];
                if let Some(url_start) = after.strip_prefix("(") {
                    if let Some(url_end) = url_start.find(')') {
                        let url = &url_start[..url_end];
                        let remaining = &url_start[url_end + 1..];
                        // Only extract if remaining is empty (pure link cell)
                        if remaining.is_empty() {
                            return (text.to_string(), Some(url.to_string()));
                        }
                    }
                }
            }
        }
        (s.to_string(), None)
    }

    // Header row
    if !headers.is_empty() {
        xml.push_str(r#"<row r="1">"#);
        for (ci, h) in headers.iter().enumerate() {
            let cl = col_letter(ci);
            let cell_ref = format!("{}1", cl);
            let (content, url) = extract_cell_content_and_url(h);
            xml.push_str(&format!(
                r#"<c r="{cell_ref}" t="inlineStr" s="1"><is><t>{}</t></is></c>"#,
                escape_xml(&content)
            ));
            if let Some(u) = url {
                hyperlinks.push((cell_ref, u));
            }
        }
        xml.push_str("</row>\n");
    }

    for (ri, row) in rows.iter().enumerate() {
        let r = ri + 2; // +1 for header, +1 for 1-indexed
        xml.push_str(&format!(r#"<row r="{}">"#, r));
        for (ci, cell) in row.iter().enumerate() {
            let cl = col_letter(ci);
            let cell_ref = format!("{}{}", cl, r);
            let (content, url) = extract_cell_content_and_url(cell);
            xml.push_str(&format!(
                r#"<c r="{cell_ref}" t="inlineStr"><is><t>{}</t></is></c>"#,
                escape_xml(&content)
            ));
            if let Some(u) = url {
                hyperlinks.push((cell_ref, u));
            }
        }
        xml.push_str("</row>\n");
    }

    xml.push_str("</sheetData>\n");

    if !hyperlinks.is_empty() {
        xml.push_str("<hyperlinks>\n");
        for (i, (cell_ref, _url)) in hyperlinks.iter().enumerate() {
            let rid = format!("rId{}", i + 1);
            xml.push_str(&format!(
                r#"<hyperlink ref="{cell_ref}" r:id="{rid}"/>"#,
            ));
            xml.push('\n');
        }
        xml.push_str("</hyperlinks>\n");
    }

    // Merged cells: detect adjacent duplicate cell content in each row
    let merge_cells = detect_merge_cells(headers, rows);
    if !merge_cells.is_empty() {
        xml.push_str(&format!("<mergeCells count=\"{}\">\n", merge_cells.len()));
        for ref_range in &merge_cells {
            xml.push_str(&format!("<mergeCell ref=\"{}\"/>\n", ref_range));
        }
        xml.push_str("</mergeCells>\n");
    }

    xml.push_str("</worksheet>\n");
    (xml, hyperlinks)
}

fn detect_merge_cells(headers: &[String], rows: &[Vec<String>]) -> Vec<String> {
    let mut result = Vec::new();
    // Check header row
    if !headers.is_empty() {
        let mut start = 0;
        while start < headers.len() {
            let mut end = start + 1;
            while end < headers.len() && headers[end] == headers[start] {
                end += 1;
            }
            if end - start > 1 {
                let from = format!("{}{}", col_letter(start), 1);
                let to = format!("{}{}", col_letter(end - 1), 1);
                result.push(format!("{}:{}", from, to));
            }
            start = end;
        }
    }
    // Check data rows
    for (ri, row) in rows.iter().enumerate() {
        let r = ri + 2;
        let mut start = 0;
        while start < row.len() {
            let mut end = start + 1;
            while end < row.len() && row[end] == row[start] {
                end += 1;
            }
            if end - start > 1 {
                let from = format!("{}{}", col_letter(start), r);
                let to = format!("{}{}", col_letter(end - 1), r);
                result.push(format!("{}:{}", from, to));
            }
            start = end;
        }
    }
    result
}

// ─── Public API ───

pub struct OfficeRenderer;

impl OfficeRenderer {
    /// Markdown → DOCX
    pub fn docx_from_markdown(markdown: &str, output: &Path) -> Result<(), String> {
        let elements = parse_markdown(markdown);
        docx_from_elements(&elements, output)
    }

    /// Markdown → PPTX (H2 headings become slide boundaries)
    pub fn pptx_from_markdown(markdown: &str, output: &Path) -> Result<(), String> {
        let elements = parse_markdown(markdown);
        pptx_from_elements(&elements, output)
    }

    /// Markdown → XLSX (first table, or paragraphs as single-cell rows)
    pub fn xlsx_from_markdown(markdown: &str, output: &Path) -> Result<(), String> {
        let elements = parse_markdown(markdown);
        xlsx_from_elements(&elements, output)
    }

    /// Direct element → DOCX (for backend integration)
    pub fn docx_from_elements(elements: &[MdElement], output: &Path) -> Result<(), String> {
        docx_from_elements(elements, output)
    }

    /// Direct element → PPTX
    pub fn pptx_from_elements(elements: &[MdElement], output: &Path) -> Result<(), String> {
        pptx_from_elements(elements, output)
    }

    /// Direct element → XLSX
    pub fn xlsx_from_elements(elements: &[MdElement], output: &Path) -> Result<(), String> {
        xlsx_from_elements(elements, output)
    }
}

// ─── XML Constants ───

static CONTENT_TYPES_DOCX: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#;

static RELS_DOCX: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;

static CORE_PROPS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/">
<dc:creator>NeoTrix</dc:creator>
<dc:description>Generated by NeoTrix OfficeRenderer</dc:description>
</cp:coreProperties>"#;

static PPTX_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="ppt/presentation.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;

static SLIDE_MASTER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldMaster xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<p:cSld><p:spTree><p:nvGrpSpPr><p:nvPr/><p:cNvPr id="1" name=""/><p:nvGrpSpPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>
<p:sldLayoutIdLst><p:sldLayoutId id="2147483648" r:id="rId1"/></p:sldLayoutIdLst>
</p:sldMaster>"#;

static SLIDE_MASTER_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
</Relationships>"#;

static SLIDE_LAYOUT: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sldLayout xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" type="blank">
<p:cSld><p:spTree><p:nvGrpSpPr><p:nvPr/><p:cNvPr id="1" name=""/><p:nvGrpSpPr/></p:nvGrpSpPr><p:grpSpPr/></p:spTree></p:cSld>
</p:sldLayout>"#;

static SLIDE_LAYOUT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster" Target="../slideMasters/slideMaster1.xml"/>
</Relationships>"#;

fn build_slide_rels_xml(hyperlinks: &[(String, String)], images: &[(String, usize, String)], _slide_num: usize) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout" Target="../slideLayouts/slideLayout1.xml"/>
"#,
    );
    for (rid, url) in hyperlinks {
        xml.push_str(&format!(
            r#"<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="{url}" TargetMode="External"/>
"#,
        ));
    }
    for (rid, global_idx, ext) in images {
        xml.push_str(&format!(
            r#"<Relationship Id="{rid}" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image{global_idx}.{ext}"/>
"#,
        ));
    }
    xml.push_str("</Relationships>");
    xml
}

static SHEET_RELS_EMPTY: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;

static XLSX_CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
<Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
<Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
<Override PartName="/xl/sharedStrings.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sharedStrings+xml"/>
<Override PartName="/xl/theme/theme1.xml" ContentType="application/vnd.openxmlformats-officedocument.theme+xml"/>
<Override PartName="/docProps/core.xml" ContentType="application/vnd.openxmlformats-package.core-properties+xml"/>
</Types>"#;

static XLSX_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties" Target="docProps/core.xml"/>
</Relationships>"#;

static XL_WORKBOOK_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
<Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/sharedStrings" Target="sharedStrings.xml"/>
<Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme" Target="theme/theme1.xml"/>
</Relationships>"#;

static XL_WORKBOOK: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
<sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets>
</workbook>"#;

static XLSX_THEME: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<a:theme xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" name="Default">
<a:themeElements>
<a:clrScheme name="Office"><a:dk1><a:srgbClr val="000000"/></a:dk1><a:lt1><a:srgbClr val="FFFFFF"/></a:lt1><a:dk2><a:srgbClr val="44546A"/></a:dk2><a:lt2><a:srgbClr val="E7E6E6"/></a:lt2><a:accent1><a:srgbClr val="4472C4"/></a:accent1><a:accent2><a:srgbClr val="ED7D31"/></a:accent2><a:accent3><a:srgbClr val="A5A5A5"/></a:accent3><a:accent4><a:srgbClr val="FFC000"/></a:accent4><a:accent5><a:srgbClr val="5B9BD5"/></a:accent5><a:accent6><a:srgbClr val="70AD47"/></a:accent6><a:hlink><a:srgbClr val="0563C1"/></a:hlink><a:folHlink><a:srgbClr val="954F72"/></a:folHlink></a:clrScheme>
<a:fontScheme name="Office"><a:majorFont><a:latin typeface="Calibri Light"/><a:ea typeface=""/><a:cs typeface=""/></a:majorFont><a:minorFont><a:latin typeface="Calibri"/><a:ea typeface=""/><a:cs typeface=""/></a:minorFont></a:fontScheme>
<a:fmtScheme name="Office"><a:fillStyleLst><a:solidFill><a:schemeClr val="phClr"/></a:solidFill><a:gradFill rotWithShape="1"><a:gsLst><a:gs pos="0"><a:schemeClr val="phClr"/></a:gs><a:gs pos="50000"><a:schemeClr val="phClr"/></a:gs><a:gs pos="100000"><a:schemeClr val="phClr"/></a:gs></a:gsLst></a:gradFill></a:fillStyleLst><a:lnStyleLst><a:ln w="6350"><a:solidFill><a:schemeClr val="phClr"/></a:solidFill></a:ln></a:lnStyleLst></a:fmtScheme>
</a:themeElements></a:theme>"#;

// ─── Tests ───

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("a&b<c>d\"e'f"), "a&amp;b&lt;c&gt;d&quot;e&apos;f");
    }

    #[test]
    fn test_col_letter() {
        assert_eq!(col_letter(0), "A");
        assert_eq!(col_letter(1), "B");
        assert_eq!(col_letter(25), "Z");
        assert_eq!(col_letter(26), "AA");
        assert_eq!(col_letter(701), "ZZ");
    }

    #[test]
    fn test_parse_inline_bold() {
        let inlines = parse_inline("Hello **world** test");
        assert_eq!(inlines.len(), 3);
        assert_eq!(inlines[1].text, "world");
        assert!(inlines[1].bold);
    }

    #[test]
    fn test_parse_inline_italic() {
        let inlines = parse_inline("Hello *world* test");
        assert_eq!(inlines.len(), 3);
        assert_eq!(inlines[1].text, "world");
        assert!(inlines[1].italic);
    }

    #[test]
    fn test_parse_inline_strikethrough() {
        let inlines = parse_inline("Hello ~~world~~ test");
        assert_eq!(inlines.len(), 3);
        assert_eq!(inlines[1].text, "world");
        assert!(inlines[1].strike);
    }

    #[test]
    fn test_parse_inline_code() {
        let inlines = parse_inline("Use `foo()` here");
        assert_eq!(inlines.len(), 3);
        assert_eq!(inlines[1].text, "foo()");
        assert!(inlines[1].code);
    }

    #[test]
    fn test_parse_inline_link() {
        let inlines = parse_inline("See [docs](https://example.com)");
        assert_eq!(inlines.len(), 2);
        assert_eq!(inlines[1].text, "docs");
        assert!(inlines[1].link.is_some());
        let (url, label) = inlines[1].link.as_ref().unwrap();
        assert_eq!(url, "https://example.com");
        assert_eq!(label, "docs");
    }

    #[test]
    fn test_parse_inline_underline() {
        let inlines = parse_inline("Hello ++world++ test");
        assert_eq!(inlines.len(), 3);
        assert_eq!(inlines[1].text, "world");
        assert!(inlines[1].underline);
    }

    #[test]
    fn test_parse_inline_superscript() {
        let inlines = parse_inline("E=mc^2^");
        assert_eq!(inlines.len(), 2);
        assert_eq!(inlines[1].text, "2");
        assert!(inlines[1].superscript);
    }

    #[test]
    fn test_parse_inline_subscript() {
        let inlines = parse_inline("H~2~O");
        assert_eq!(inlines.len(), 3);
        assert_eq!(inlines[1].text, "2");
        assert!(inlines[1].subscript);
    }

    #[test]
    fn test_inline_to_docx_underline() {
        let md = "++underline++";
        let inlines = parse_inline(md);
        assert_eq!(inlines.len(), 1);
        assert!(inlines[0].underline);
        let mut hyperlinks = Vec::new();
        let xml = inline_to_docx_xml(&inlines, &mut hyperlinks, &mut Vec::new());
        assert!(xml.contains(r#"<w:u w:val="single"/>"#), "xml: {}", xml);
    }

    #[test]
    fn test_inline_to_docx_superscript() {
        let inlines = parse_inline("E=mc^2^");
        let mut hyperlinks = Vec::new();
        let xml = inline_to_docx_xml(&inlines, &mut hyperlinks, &mut Vec::new());
        assert!(xml.contains(r#"<w:vertAlign w:val="superscript"/>"#), "xml: {}", xml);
    }

    #[test]
    fn test_inline_to_docx_subscript() {
        let inlines = parse_inline("H~2~O");
        let mut hyperlinks = Vec::new();
        let xml = inline_to_docx_xml(&inlines, &mut hyperlinks, &mut Vec::new());
        assert!(xml.contains(r#"<w:vertAlign w:val="subscript"/>"#), "xml: {}", xml);
    }

    #[test]
    fn test_parse_markdown_headings() {
        let elements = parse_markdown("# Title\n\n## Sub\n\n### Subsub");
        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0].kind, MdKind::Heading(1, _)));
        assert!(matches!(elements[1].kind, MdKind::Heading(2, _)));
        assert!(matches!(elements[2].kind, MdKind::Heading(3, _)));
    }

    #[test]
    fn test_parse_markdown_table() {
        let md = "| A | B |\n| --- | --- |\n| 1 | 2 |";
        let elements = parse_markdown(md);
        assert_eq!(elements.len(), 1);
        if let MdKind::Table { headers, rows } = &elements[0].kind {
            assert_eq!(headers, &["A", "B"]);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0], ["1", "2"]);
        } else {
            panic!("expected table");
        }
    }

    #[test]
    fn test_parse_markdown_list() {
        let md = "- item1\n- item2\n- item3";
        let elements = parse_markdown(md);
        assert_eq!(elements.len(), 1);
        if let MdKind::List { ordered, items } = &elements[0].kind {
            assert!(!ordered);
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].1[0].text, "item1");
        } else {
            panic!("expected list");
        }
    }

    #[test]
    fn test_parse_toc_marker() {
        let md = "[toc]\n\n# Title\ncontent";
        let elements = parse_markdown(md);
        assert!(matches!(elements[0].kind, MdKind::TableOfContents));
    }

    #[test]
    fn test_parse_nested_list_levels() {
        let md = "- top\n  - nested\n    - deeper";
        let elements = parse_markdown(md);
        assert_eq!(elements.len(), 1);
        if let MdKind::List { items, .. } = &elements[0].kind {
            assert_eq!(items.len(), 3);
            assert_eq!(items[0].0, 0);
            assert_eq!(items[1].0, 1);
            assert_eq!(items[2].0, 2);
        } else {
            panic!("expected list");
        }
    }

    #[test]
    fn test_parse_markdown_code_block() {
        let md = "```\nfn foo() {}\n```";
        let elements = parse_markdown(md);
        assert_eq!(elements.len(), 1);
        if let MdKind::CodeBlock(code) = &elements[0].kind {
            assert!(code.contains("fn foo"));
        } else {
            panic!("expected code block");
        }
    }

    #[test]
    fn test_parse_markdown_thematic_break() {
        let md = "text\n\n---\n\nmore";
        let elements = parse_markdown(md);
        let brk = elements.iter().find(|e| matches!(e.kind, MdKind::ThematicBreak));
        assert!(brk.is_some(), "expected thematic break");
    }

    #[test]
    fn test_inline_to_docx_bold() {
        let inlines = vec![
            MdInline::plain("Hello "),
            MdInline::new("world", InlineOpts { bold: true, ..Default::default() }),
        ];
        let xml = inline_to_docx_xml(&inlines, &mut Vec::new(), &mut Vec::new());
        assert!(xml.contains("<w:b/>"), "bold marker missing: {}", xml);
        assert!(xml.contains("world"), "text missing: {}", xml);
    }

    #[test]
    fn test_docx_generation() {
        let md = "# Title\n\nHello **world**.\n\n## Sub\n\n- item1\n- item2\n\n| A | B |\n| --- | --- |\n| 1 | 2 |";
        let tmp = std::env::temp_dir().join("test_rw_docx.docx");
        let result = OfficeRenderer::docx_from_markdown(md, &tmp);
        assert!(result.is_ok(), "docx generation failed: {:?}", result);
        assert!(tmp.exists(), "output file does not exist");
        assert!(tmp.metadata().unwrap().len() > 100, "output file too small");
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_xlsx_generation() {
        let md = "| Name | Age | City |\n| --- | --- | --- |\n| Alice | 30 | NYC |\n| Bob | 25 | SF |";
        let tmp = std::env::temp_dir().join("test_rw_xlsx.xlsx");
        let result = OfficeRenderer::xlsx_from_markdown(md, &tmp);
        assert!(result.is_ok(), "xlsx generation failed: {:?}", result);
        assert!(tmp.exists());
        assert!(tmp.metadata().unwrap().len() > 100);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_pptx_generation() {
        let md = "## Slide 1\n\nWelcome content\n\n## Slide 2\n\nMore content";
        let tmp = std::env::temp_dir().join("test_rw_pptx.pptx");
        let result = OfficeRenderer::pptx_from_markdown(md, &tmp);
        assert!(result.is_ok(), "pptx generation failed: {:?}", result);
        assert!(tmp.exists());
        assert!(tmp.metadata().unwrap().len() > 100);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_docx_code_block() {
        let md = "```rust\nfn hello() { println!(\"hi\"); }\n```";
        let tmp = std::env::temp_dir().join("test_code.docx");
        let result = OfficeRenderer::docx_from_markdown(md, &tmp);
        assert!(result.is_ok(), "code block docx failed: {:?}", result);
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_docx_thematic_break() {
        let md = "before\n\n---\n\nafter";
        let tmp = std::env::temp_dir().join("test_hr.docx");
        let result = OfficeRenderer::docx_from_markdown(md, &tmp);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_xlsx_simple_list() {
        let md = "- Apples\n- Bananas\n- Cherries";
        let tmp = std::env::temp_dir().join("test_list.xlsx");
        let result = OfficeRenderer::xlsx_from_markdown(md, &tmp);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_pptx_single_slide() {
        let md = "Just some content without headings";
        let tmp = std::env::temp_dir().join("test_single.pptx");
        let result = OfficeRenderer::pptx_from_markdown(md, &tmp);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_empty_markdown_docx() {
        let md = "";
        let tmp = std::env::temp_dir().join("test_empty.docx");
        let result = OfficeRenderer::docx_from_markdown(md, &tmp);
        assert!(result.is_ok());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_xlsx_hyperlinks_roundtrip() {
        let md = "| [Click Here](https://example.com) | [Search](https://google.com) |\n| --- | --- |\n| 1 | 2 |";
        let tmp = std::env::temp_dir().join("test_xlsx_hyperlinks.xlsx");
        let result = OfficeRenderer::xlsx_from_markdown(md, &tmp);
        assert!(result.is_ok(), "xlsx generation: {:?}", result);

        // Parse back and verify hyperlinks
        use crate::neotrix::l2_world_impl::nt_world_parse::backends::office_oxide_backend::OfficeOxideBackend;
        use crate::neotrix::l2_world_impl::nt_world_parse::doc_parser::DocParser;
        let backend = OfficeOxideBackend;
        let parsed = backend.parse_office(&tmp).expect("parse should succeed");
        assert!(parsed.full_markdown.contains("[Click Here](https://example.com)"),
            "should preserve hyperlink, got: {}", parsed.full_markdown);
        assert!(parsed.full_markdown.contains("[Search](https://google.com)"),
            "should preserve second hyperlink, got: {}", parsed.full_markdown);

        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_parse_escaped_delimiters() {
        let inlines = parse_inline(r"Hello \*world\* test");
        assert_eq!(inlines.len(), 1, "escaped delimiters should be plain text");
        assert_eq!(inlines[0].text, "Hello *world* test");
        assert!(!inlines[0].italic);
    }

    #[test]
    fn test_parse_escaped_brackets() {
        let inlines = parse_inline(r"Not \[a link\]\(url\)");
        assert_eq!(inlines.len(), 1);
        assert_eq!(inlines[0].text, "Not [a link](url)");
        assert!(inlines[0].link.is_none());
    }

    #[test]
    fn test_pptx_inline_hyperlink() {
        let inlines = parse_inline("Hello [click](https://example.com)");
        let mut hyperlinks = Vec::new();
        let xml = inline_to_pptx_xml(&inlines, &mut hyperlinks);
        assert!(xml.contains(r#"<a:hlinkClick r:id="rIdH1"/>"#), "xml: {}", xml);
        assert_eq!(hyperlinks.len(), 1);
        assert_eq!(hyperlinks[0].1, "https://example.com");
    }

    #[test]
    fn test_pptx_slide_hyperlink() {
        let inlines = parse_inline("[link](https://example.com)");
        let items = vec![SlideItem::Text(inlines, 0)];
        let mut hyperlinks = Vec::new();
        let xml = build_pptx_slide_xml("Test", &items, &mut hyperlinks, &mut Vec::new());
        assert!(xml.contains(r#"<a:hlinkClick r:id="rIdH1"/>"#), "xml: {}", xml);
        assert_eq!(hyperlinks.len(), 1);
    }

    #[test]
    fn test_slide_rels_hyperlink() {
        let hyperlinks = vec![("rIdH1".to_string(), "https://example.com".to_string())];
        let xml = build_slide_rels_xml(&hyperlinks, &[], 1);
        assert!(xml.contains(r#"Relationship Id="rIdH1""#), "xml: {}", xml);
        assert!(xml.contains(r#"Target="https://example.com""#), "xml: {}", xml);
        assert!(xml.contains("TargetMode=\"External\""), "xml: {}", xml);
    }

    #[test]
    fn test_detect_merge_cells_header() {
        let headers = vec!["A".to_string(), "A".to_string(), "B".to_string()];
        let rows: Vec<Vec<String>> = vec![];
        let merges = detect_merge_cells(&headers, &rows);
        assert!(merges.contains(&"A1:B1".to_string()), "merges: {:?}", merges);
    }

    #[test]
    fn test_detect_merge_cells_data() {
        let headers = vec!["H".to_string()];
        let rows = vec![
            vec!["X".to_string(), "X".to_string(), "Y".to_string()],
        ];
        let merges = detect_merge_cells(&headers, &rows);
        assert!(merges.contains(&"A2:B2".to_string()), "merges: {:?}", merges);
    }

    #[test]
    fn test_docx_paragraph_spacing() {
        let inlines = parse_inline("Hello world");
        let mut hyperlinks = Vec::new();
        let _xml = inline_to_docx_xml(&inlines, &mut hyperlinks, &mut Vec::new());
        let md = "Hello world";
        let elements = parse_markdown(md);
        let mut h = Vec::new();
        let body = build_docx_body(&elements, &mut h, &mut Vec::new(), &["Test".to_string()]);
        assert!(body.contains(r#"<w:spacing w:before="120" w:after="120" w:line="276" w:lineRule="auto"/>"#),
            "paragraph spacing not found: {}", body);
    }

    #[test]
    fn test_docx_heading_spacing() {
        let elements = parse_markdown("# Title");
        let mut h = Vec::new();
        let body = build_docx_body(&elements, &mut h, &mut Vec::new(), &["Title".to_string()]);
        assert!(body.contains(r#"<w:spacing w:before="360" w:after="120"/>"#),
            "heading spacing not found: {}", body);
    }

    #[test]
    fn test_parse_image_markdown() {
        let inlines = parse_inline("![alt text](image.png)");
        assert_eq!(inlines.len(), 1, "expected 1 inline, got {}: {:?}", inlines.len(), inlines);
        assert!(inlines[0].image, "expected image=true");
        if let Some((url, alt)) = &inlines[0].link {
            assert_eq!(url, "image.png");
            assert_eq!(alt, "alt text");
        } else {
            panic!("expected link field with image url");
        }
    }

    #[test]
    fn test_parse_image_with_text() {
        let inlines = parse_inline("Hello ![img](pic.png) world");
        assert_eq!(inlines.len(), 3, "expected 3 inlines, got {}: {:?}", inlines.len(), inlines);
        assert_eq!(inlines[0].text, "Hello ");
        assert!(inlines[1].image);
        assert_eq!(inlines[2].text, " world");
    }

    #[test]
    fn test_inline_to_docx_image_xml() {
        let inlines = vec![MdInline::image("photo.jpg".to_string(), "A photo".to_string())];
        let mut hyperlinks = Vec::new();
        let mut images = Vec::new();
        let xml = inline_to_docx_xml(&inlines, &mut hyperlinks, &mut images);
        assert!(xml.contains(r#"<w:drawing>"#), "expected drawing: {}", xml);
        assert!(xml.contains(r#"<a:blip r:embed="rIdImg1"/>"#), "expected blip: {}", xml);
        assert_eq!(images.len(), 1, "expected 1 image ref");
        assert_eq!(images[0].1, "photo.jpg");
    }

    #[test]
    fn test_pptx_slide_item_image() {
        let items = vec![SlideItem::Image { path: "test.png".to_string(), alt: "Test Image".to_string() }];
        let mut hyperlinks = Vec::new();
        let mut images = Vec::new();
        let xml = build_pptx_slide_xml("Slide 1", &items, &mut hyperlinks, &mut images);
        assert!(xml.contains(r#"<p:pic>"#), "expected pic element: {}", xml);
        assert!(xml.contains(r#"<a:blip r:embed="rIdImg1"/>"#), "expected blip: {}", xml);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].1, "test.png");
    }

    #[test]
    fn test_parse_pagebreak_marker() {
        let elements = parse_markdown("Before\n<!--- pagebreak -->\nAfter");
        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[0].kind, MdKind::Paragraph(_)));
        assert!(matches!(elements[1].kind, MdKind::PageBreak));
        assert!(matches!(elements[2].kind, MdKind::Paragraph(_)));
    }

    #[test]
    fn test_parse_backslash_pagebreak() {
        let elements = parse_markdown("Text\n\\page\nMore");
        assert_eq!(elements.len(), 3);
        assert!(matches!(elements[1].kind, MdKind::PageBreak));
    }

    #[test]
    fn test_parse_alignment_center() {
        let elements = parse_markdown("-> Hello");
        assert_eq!(elements.len(), 1);
        assert!(matches!(elements[0].kind, MdKind::Paragraph(_)));
        assert_eq!(elements[0].alignment, Alignment::Center);
    }

    #[test]
    fn test_parse_alignment_right() {
        let elements = parse_markdown("<- Hello");
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].alignment, Alignment::Right);
    }

    #[test]
    fn test_parse_alignment_justify() {
        let elements = parse_markdown(">< Hello");
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].alignment, Alignment::Justify);
    }

    #[test]
    fn test_docx_pagebreak_in_body() {
        let elements = parse_markdown("<!--- pagebreak -->");
        let mut h = Vec::new();
        let body = build_docx_body(&elements, &mut h, &mut Vec::new(), &["Test".to_string()]);
        assert!(body.contains("w:pageBreakBefore"), "expected pageBreakBefore: {}", body);
    }

    #[test]
    fn test_docx_alignment_center() {
        let line = "-> Centered text";
        let (inlines, align) = parse_paragraph_with_alignment(line);
        assert_eq!(align, Alignment::Center);
        assert_eq!(inlines.len(), 1);
        assert_eq!(inlines[0].text, "Centered text");
        let elements = vec![MdElement { kind: MdKind::Paragraph(inlines), alignment: align }];
        let mut h = Vec::new();
        let body = build_docx_body(&elements, &mut h, &mut Vec::new(), &["Test".to_string()]);
        assert!(body.contains(r#"w:val="center""#), "expected center alignment: {}", body);
    }
}

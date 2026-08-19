// C3 benchmark 基线 — pdf_edit 能力节点 (C2→C3 晋升证据)
//
// 覆盖:
// - FileParser::edit_pdf_text redact-only  — span 清空 (保版式) 吞吐
// - FileParser::edit_pdf_text TTF 嵌入替换 — Type0/Identity-H 子集字体嵌入 + 原位替换吞吐
//
// 目的: 建立 NT-ACT pdf_edit 性能基线 (C3 = benchmark 基线建立 + 无回归),
// 对比历史 (cargo bench --bench pdf_c3)。基准 PDF 为合成生成 (页内容流 N 个 Tj span)。

use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use lopdf::{dictionary, content::Content, content::Operation, Document, Object, Stream};
use neotrix_types::core::file_parser::pdf::PdfTextEdit;
use neotrix_types::core::file_parser::FileParser;

/// 构造含 `n` 个独立 Tj span 的单页 PDF (WinAnsi 可解字体 F1)。
fn make_pdf(n: usize) -> Vec<u8> {
    let mut doc = Document::with_version("1.4");
    let pages_id = doc.new_object_id();
    let font_id = doc.add_object(dictionary! {
        "Type" => "Font",
        "Subtype" => "Type1",
        "BaseFont" => "Courier",
    });
    let resources_id = doc.add_object(dictionary! {
        "Font" => dictionary! { "F1" => font_id },
    });
    let mut ops: Vec<Operation> = Vec::with_capacity(n * 4 + 2);
    ops.push(Operation::new("BT", vec![]));
    for i in 0..n {
        ops.push(Operation::new("Tf", vec!["F1".into(), 12.into()]));
        ops.push(Operation::new("Td", vec![(i as f32 * 0.5).into(), 600.into()]));
        ops.push(Operation::new("Tj", vec![Object::string_literal(format!("span_{i:04}_text_here"))]));
    }
    ops.push(Operation::new("ET", vec![]));
    let content = Content { operations: ops };
    let content_id = doc.add_object(Stream::new(
        lopdf::Dictionary::new(),
        content.encode().expect("encode content"),
    ));
    let page = doc.add_object(dictionary! {
        "Type" => "Page",
        "Parent" => pages_id,
        "Contents" => content_id,
        "Resources" => resources_id,
    });
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => vec![page.into()],
            "Count" => 1,
        }),
    );
    let catalog_id = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog_id);
    let mut out = Vec::new();
    doc.save_to(&mut out).expect("save pdf");
    out
}

fn bench_redact(c: &mut Criterion) {
    let mut group = c.benchmark_group("pdf_edit_redact");
    for &n in &[50usize, 200] {
        let pdf = make_pdf(n);
        let edits = vec![PdfTextEdit {
            page: 1,
            find: "span_0007_text_here".into(),
            replace: None,
        }];
        group.bench_function(format!("redact_{n}spans"), |b| {
            b.iter_batched(
                || pdf.clone(),
                |p| {
                    let out = FileParser::edit_pdf_text(&p, &edits, None);
                    black_box(out.expect("redact ok").len());
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_replace_ttf(c: &mut Criterion) {
    let ttf = std::fs::read("/System/Library/Fonts/Supplemental/Arial.ttf")
        .or_else(|_| std::fs::read("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"))
        .ok();
    if ttf.is_none() {
        eprintln!("[pdf_c3] 无可用 TTF (跳过 TTF 替换基准)");
        return;
    }
    let ttf = ttf.unwrap();
    let mut group = c.benchmark_group("pdf_edit_replace_ttf");
    for &n in &[50usize, 200] {
        let pdf = make_pdf(n);
        let edits = vec![PdfTextEdit {
            page: 1,
            find: "span_0007_text_here".into(),
            replace: Some("ЗАДВИЖКА-тест".into()),
        }];
        let ttf_b = ttf.clone();
        group.bench_function(format!("replace_{n}spans"), |b| {
            b.iter_batched(
                || (pdf.clone(), ttf_b.clone()),
                |(p, f)| {
                    let out = FileParser::edit_pdf_text(&p, &edits, Some(&f));
                    black_box(out.expect("replace ok").len());
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, bench_redact, bench_replace_ttf);
criterion_main!(benches);
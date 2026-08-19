//! 通用表格读写 (D1/D2): XLSX 写/读 + CSV/TSV 读写 + 表格级编辑。
//! 对标: python-docx/openpyxl/csv。吸收此前 Python 价格表脚本的表格化逻辑为原生能力。

use std::path::Path;

use super::encoding::decode_bytes;
use super::types::{FileAbilityError, Result, TableData};

/// 写入 XLSX 表格 (D1) — 表头加粗+底色, 数值列带数字格式, 列宽自适应。
/// 对标 openpyxl: 支持字符串/数值/公式单元格。
pub fn write_xlsx_table(path: impl AsRef<Path>, table: &TableData) -> Result<()> {
    use office_oxide::xlsx::write::{CellData, CellStyle, HAlign, XlsxWriter};
    let mut wb = XlsxWriter::new();
    let sheet = wb.add_sheet_get_index(if table.name.is_empty() {
        "Sheet1"
    } else {
        &table.name
    });
    // 表头
    let header_style = CellStyle::new()
        .bold()
        .font_color("FFFFFF")
        .background("2F5496")
        .align(HAlign::Center)
        .wrap();
    for (col, h) in table.headers.iter().enumerate() {
        wb.sheet_set_cell_styled(sheet, 0, col, CellData::String(h.clone()), header_style.clone());
    }
    // 数据行
    for (r, row) in table.rows.iter().enumerate() {
        for (col, cell) in row.iter().enumerate() {
            let cdata = to_cell_data(cell);
            wb.sheet_set_cell(sheet, r + 1, col, cdata);
        }
    }
    // 列宽 (表头长度 + 内容最大长度, 上限 40)
    for col in 0..table.headers.len() {
        let mut w = table.headers.get(col).map(|h| h.chars().count()).unwrap_or(8);
        for row in &table.rows {
            if let Some(c) = row.get(col) {
                w = w.max(c.chars().count());
            }
        }
        wb.sheet_set_column_width(sheet, col, (w as f64).clamp(6.0, 40.0));
    }
    wb.save(path).map_err(|e| FileAbilityError::Office(office_oxide::OfficeError::from(e)))
}

/// 单元格编辑操作 (表格级编辑 API — 阶段1.5 基础能力补齐)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableEdit {
    /// 修改单元格值 (sheet 索引, 行号 0=表头后首行, 列号 0=首列)
    SetCell { sheet: usize, row: usize, col: usize, value: String },
    /// 在指定行前插入一行空值 (sheet 索引, 行号)
    InsertRow { sheet: usize, row: usize },
    /// 删除一行 (sheet 索引, 行号)
    RemoveRow { sheet: usize, row: usize },
}

/// 表格级编辑 — 读全表 → 应用编辑 → 重写回文件 (复用 L1 读写, 无独立实现)。
/// 返回编辑后的表格 (可用于链式编辑或快照存储)。
pub fn edit_xlsx_table(
    path: impl AsRef<Path>,
    edits: &[TableEdit],
) -> Result<Vec<TableData>> {
    let mut tables = read_xlsx_sheets_all(&path)?;
    for edit in edits {
        match edit {
            TableEdit::SetCell { sheet, row, col, value } => {
                let t = tables.get_mut(*sheet).ok_or_else(|| {
                    FileAbilityError::Parse(format!("sheet 越界: {sheet}"))
                })?;
                if *row >= t.rows.len() {
                    return Err(FileAbilityError::Parse(format!(
                        "row 越界: {row} (表 '{}' 共 {} 行)",
                        t.name,
                        t.rows.len()
                    )));
                }
                if *col >= t.headers.len() {
                    return Err(FileAbilityError::Parse(format!(
                        "col 越界: {col} (表 '{}' 共 {} 列)",
                        t.name,
                        t.headers.len()
                    )));
                }
                t.rows[*row][*col] = value.clone();
            }
            TableEdit::InsertRow { sheet, row } => {
                let t = tables.get_mut(*sheet).ok_or_else(|| {
                    FileAbilityError::Parse(format!("sheet 越界: {sheet}"))
                })?;
                if *row > t.rows.len() {
                    return Err(FileAbilityError::Parse(format!(
                        "InsertRow 越界: {row} (共 {} 行)",
                        t.rows.len()
                    )));
                }
                t.rows.insert(*row, vec![String::new(); t.headers.len()]);
            }
            TableEdit::RemoveRow { sheet, row } => {
                let t = tables.get_mut(*sheet).ok_or_else(|| {
                    FileAbilityError::Parse(format!("sheet 越界: {sheet}"))
                })?;
                if *row >= t.rows.len() {
                    return Err(FileAbilityError::Parse(format!(
                        "RemoveRow 越界: {row} (共 {} 行)",
                        t.rows.len()
                    )));
                }
                t.rows.remove(*row);
            }
        }
    }
    // 多 sheet 用 write_xlsx_sheets 重写
    write_xlsx_sheets(&path, &tables)?;
    Ok(tables)
}

/// 多 sheet 写入 — 每表一个 sheet (用于编辑回写)。
fn write_xlsx_sheets(path: impl AsRef<Path>, tables: &[TableData]) -> Result<()> {
    use office_oxide::xlsx::write::{CellData, CellStyle, HAlign, XlsxWriter};
    let mut wb = XlsxWriter::new();
    for t in tables {
        let sheet = wb.add_sheet_get_index(if t.name.is_empty() { "Sheet1" } else { &t.name });
        let header_style = CellStyle::new()
            .bold()
            .font_color("FFFFFF")
            .background("2F5496")
            .align(HAlign::Center)
            .wrap();
        for (col, h) in t.headers.iter().enumerate() {
            wb.sheet_set_cell_styled(sheet, 0, col, CellData::String(h.clone()), header_style.clone());
        }
        for (r, row) in t.rows.iter().enumerate() {
            for (col, cell) in row.iter().enumerate() {
                wb.sheet_set_cell(sheet, r + 1, col, to_cell_data(cell));
            }
        }
    }
    wb.save(path).map_err(|e| FileAbilityError::Office(office_oxide::OfficeError::from(e)))
}

/// 单元格文本 → CellData (纯数字→Number, 公式前缀→Formula, 其余→String)
fn to_cell_data(text: &str) -> office_oxide::xlsx::write::CellData {
    use office_oxide::xlsx::write::CellData;
    let t = text.trim();
    if let Some(f) = t.strip_prefix('=') {
        if !f.is_empty() {
            return CellData::Formula(f.to_string());
        }
    }
    if let Ok(n) = t.replace([',', '￥'], "").parse::<f64>() {
        return CellData::Number(n);
    }
    CellData::String(t.to_string())
}

/// 写入 CSV 文件 (UTF-8, BOM 可选) — 对标 Python csv.writer
pub fn write_csv(path: impl AsRef<Path>, table: &TableData, delimiter: char, with_bom: bool) -> Result<()> {
    use std::io::Write;
    let mut buf = Vec::new();
    if with_bom {
        buf.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    }
    let mut write_row = |cells: &[String]| -> std::io::Result<()> {
        let line: Vec<String> = cells
            .iter()
            .map(|c| {
                if c.contains(delimiter) || c.contains('"') || c.contains('\n') {
                    format!("\"{}\"", c.replace('"', "\"\""))
                } else {
                    c.clone()
                }
            })
            .collect();
        buf.extend_from_slice(line.join(&delimiter.to_string()).as_bytes());
        buf.push(b'\n');
        Ok(())
    };
    write_row(&table.headers).map_err(FileAbilityError::Io)?;
    for row in &table.rows {
        write_row(row).map_err(FileAbilityError::Io)?;
    }
    let mut f = std::fs::File::create(path.as_ref()).map_err(FileAbilityError::Io)?;
    f.write_all(&buf).map_err(FileAbilityError::Io)
}

/// 读取 CSV/TSV 文件 (自动编码检测: UTF-8 BOM / UTF-8 / GBK) (D2+D3)
pub fn read_csv(path: impl AsRef<Path>) -> Result<TableData> {
    let raw = std::fs::read(path.as_ref()).map_err(FileAbilityError::Io)?;
    let text = decode_bytes(&raw);
    let name = path
        .as_ref()
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    Ok(TableData::from_delimited_text(name, &text))
}

/// 读取 XLSX 所有工作表为表格 (calamine 驱动, 懒加载 per-sheet)。
///
/// 外部吸收 (R-P42): calamine 0.36 读 xlsx 比 openpyxl 快 ~9.4×, 支持
/// xls/xlsm/xlsb/ods, 纯 Rust 零 unsafe。对齐 E13 教训: 多 sheet XLSX 需全遍历,
/// 每个 sheet 都可能含独立数据。表头 = 每 sheet 第一个非空行。
pub fn read_xlsx_sheets_all(path: impl AsRef<Path>) -> Result<Vec<TableData>> {
    use calamine::{open_workbook, Reader, Xlsx};
    use std::io::BufReader;
    let mut wb: Xlsx<BufReader<std::fs::File>> = open_workbook(path.as_ref())
        .map_err(|e: calamine::XlsxError| FileAbilityError::Parse(e.to_string()))?;
    let names = wb.sheet_names().to_vec();
    let mut out = Vec::new();
    for name in names {
        let range = wb
            .worksheet_range(&name)
            .map_err(|e: calamine::XlsxError| FileAbilityError::Parse(e.to_string()))?;
        // 计算最大列 (cells() 产出 (row, col, &Data), c.1 为列索引)
        let max_col = range
            .cells()
            .map(|c| c.1)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        if max_col == 0 {
            continue;
        }
        // 按 (row, col) 填充
        let mut grid: Vec<Vec<String>> = Vec::new();
        for cell in range.cells() {
            let (r, c, v) = (cell.0, cell.1, cell.2);
            while grid.len() <= r {
                grid.push(vec![String::new(); max_col]);
            }
            grid[r][c] = data_to_text(v);
        }
        // 表头 = 第一个非空行
        let header_idx = grid
            .iter()
            .position(|row| row.iter().any(|c| !c.trim().is_empty()))
            .unwrap_or(0);
        let headers: Vec<String> = grid
            .get(header_idx)
            .cloned()
            .unwrap_or_else(|| vec![String::new(); max_col]);
        let rows: Vec<Vec<String>> = grid
            .iter()
            .enumerate()
            .filter(|(i, row)| *i > header_idx && row.iter().any(|c| !c.trim().is_empty()))
            .map(|(_, row)| row.clone())
            .collect();
        out.push(TableData {
            name: name.clone(),
            headers,
            rows,
        });
    }
    Ok(out)
}

/// calamine Data → 显示文本 (与 office_oxide 显示文本语义对齐)
pub(super) fn data_to_text(d: &calamine::Data) -> String {
    match d {
        calamine::Data::Int(i) => i.to_string(),
        calamine::Data::Float(f) => {
            if f.fract() == 0.0 && f.abs() < 1e15 {
                format!("{}", *f as i64)
            } else {
                format!("{}", f)
            }
        }
        calamine::Data::String(s) => s.clone(),
        calamine::Data::Bool(b) => b.to_string(),
        calamine::Data::DateTime(dt) => dt
            .as_datetime()
            .map(|d| d.to_string())
            .unwrap_or_else(|| dt.to_string()),
        calamine::Data::DateTimeIso(s) => s.clone(),
        calamine::Data::DurationIso(s) => s.clone(),
        calamine::Data::Error(_) => String::new(),
        calamine::Data::Empty => String::new(),
    }
}

/// 读取 XLSX 第一个非空工作表为表格 (calamine 驱动)。
/// 兼容原 office_oxide 语义; 多 sheet 场景用 [`read_xlsx_sheets_all`]。
pub fn read_xlsx_table(path: impl AsRef<Path>) -> Result<TableData> {
    let tables = read_xlsx_sheets_all(&path)?;
    tables
        .iter()
        .find(|t| !t.rows.is_empty())
        .cloned()
        .or_else(|| tables.first().cloned())
        .ok_or_else(|| FileAbilityError::Parse("XLSX 无工作表".into()))
}
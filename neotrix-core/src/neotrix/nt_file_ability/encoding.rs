//! 编码检测 (D3) — BOM 优先 / UTF-8 合法性 / GBK 启发式。

use super::types::TextEncoding;

/// 检测字节流编码 (BOM 优先, 其次 UTF-8 合法性, 再次 GBK 启发式)。
/// 对标 Python chardet — 覆盖中文场景 GBK/GB18030。
pub fn detect_encoding(data: &[u8]) -> TextEncoding {
    if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return TextEncoding::Utf8;
    }
    if data.starts_with(&[0xFF, 0xFE]) || data.starts_with(&[0xFE, 0xFF]) {
        return TextEncoding::Utf16;
    }
    if std::str::from_utf8(data).is_ok() {
        return TextEncoding::Utf8;
    }
    // GBK 启发: 高字节区段必须能成对构成合法双字节序列 (首字节 0x81-0xFE,
    // 次字节 0x40-0xFE 且非 0x7F)。以"高字节成对率"判定。
    let mut high = 0usize;
    let mut i = 0usize;
    while i < data.len() {
        let b = data[i];
        if b < 0x80 {
            i += 1;
            continue;
        }
        high += 1;
        // GBK 双字节: 首字节 0x81-0xFE, 次字节 0x40-0xFE (不含 0x7F)
        if (0x81..=0xFE).contains(&b) && i + 1 < data.len() {
            let b2 = data[i + 1];
            if (0x40..=0xFE).contains(&b2) && b2 != 0x7F {
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    if high > 0 {
        let valid_pairs = count_gbk_pairs(data);
        let pair_ratio = valid_pairs as f64 / high as f64;
        if pair_ratio > 0.9 && valid_pairs >= 2 {
            TextEncoding::Gbk
        } else {
            TextEncoding::Unknown
        }
    } else {
        TextEncoding::Unknown
    }
}

/// 统计合法 GBK 双字节对的个数
fn count_gbk_pairs(data: &[u8]) -> usize {
    let mut count = 0usize;
    let mut i = 0usize;
    while i + 1 < data.len() {
        let b = data[i];
        if (0x81..=0xFE).contains(&b) {
            let b2 = data[i + 1];
            if (0x40..=0xFE).contains(&b2) && b2 != 0x7F {
                count += 1;
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    count
}

/// 解码字节流为 UTF-8 字符串 (编码检测 + encoding_rs 转换)。
pub fn decode_bytes(data: &[u8]) -> String {
    let out = match detect_encoding(data) {
        TextEncoding::Gbk => {
            let (cow, _, _) = encoding_rs::GBK.decode(data);
            cow.into_owned()
        }
        TextEncoding::Utf16 => {
            // 自动去除 BOM
            let (cow, _, _) = encoding_rs::UTF_16LE.decode(data);
            cow.into_owned()
        }
        _ => String::from_utf8_lossy(data).into_owned(),
    };
    // 去除 BOM (UTF-8)
    out.trim_start_matches('\u{feff}').to_string()
}
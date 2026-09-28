//! `nt_vision` — 工作区图像取读：把磁盘上的一张图变成**多模态内容部件**。
//!
//! 三条硬规则，缺一不可：
//!
//! 1. **类型认魔数，不认扩展名。** `.png` 里装的是 zip 就不是 png —— 照扩展名
//!    填 `media_type`，等于把一颗不属于解码器的炸弹递到模型面前。认不出来就
//!    诚实拒，并**说清实际认成了什么**（"这是 zip 压缩包"），因为调用方往往是
//!    能据此改路的（换个文件、或者它压根不是图片）。
//! 2. **jail 与 `read_file` 同律。** 词法判定用 `nt_workspace::check_rel`
//!    （绝对路径/`~`/`..` 一律拒），落盘后仍由 `nt_agent` 的 `join_workspace`
//!    再拼一道 —— 双保险是这一层的既有惯例。
//! 3. **看不见就说看不见。** 本模块只负责「把图交出去」；能不能被模型看见由
//!    `EngineAdapter::vision_capable` 判定，见 [`crate::nt_engine`]。
//!
//! base64 是手写实现（约 25 行，见 [`base64_encode`]）：`base64` crate **不在**
//! 本 crate 的依赖里（只在 lock 图里被别人间接带着），接进来要动
//! `Cargo.toml`；RFC 4648 的标准字母表 + padding 只有十几行能背下来的规则，
//! 换来的是零新依赖。已知答案向量 + 往返测试在下面兜着。

use std::path::Path;

use crate::nt_error::NtBotError;
use crate::nt_types::ImagePart;

/// 单图上限 4 MiB。
///
/// 取值理由：base64 膨胀 4/3，4 MiB 原图 ≈ 5.6 MiB 请求体。再大的图应该先在
/// 本地缩，而不是让模型上下文被一张手机截图吃光。上限只管**编码后的字节**；
/// 解码侧的解压炸弹（几百像素的图展开成几百 MB）本模块不设防 —— 那要一个真
/// 解码器才谈得上，本模块只做到「不把超大文件读进内存」。
pub const IMAGE_SIZE_CAP: usize = 4 * 1024 * 1024;

/// 魔数探测只需文件头这么多字节（覆盖全部四种签名）。
const SNIFF_LEN: usize = 16;

/// 受支持的图像格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageKind {
    Png,
    Jpeg,
    Gif,
    Webp,
}

impl ImageKind {
    /// IANA media type —— 请求里 `image_url` 靠它告诉服务端怎么解。
    pub fn media_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Webp => "image/webp",
        }
    }
}

/// 魔数探测：文件头命中四族签名之一即认；否则 `None`（由调用方诚实拒）。
///
/// **只认头，不认名**：`extend` 一律不参与判定。
pub fn detect_kind(head: &[u8]) -> Option<ImageKind> {
    // PNG：8 字节完整签名（`\r\n\x1a\n` 收尾是规格的一部分，不是巧合）。
    if head.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some(ImageKind::Png);
    }
    // JPEG：SOI(FFD8) + 下一标记的 FF（合法 JPEG 第二个标记以 FF 起头）。
    if head.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some(ImageKind::Jpeg);
    }
    // GIF：`GIF87a` / `GIF89a`。
    if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        return Some(ImageKind::Gif);
    }
    // WebP：容器是 RIFF，格式名在**偏移 8**（`RIFF<4字节长度>WEBP`）。
    // 只看头 4 字节会把一切 RIFF 容器（AVI/WAV）当成 webp。
    if head.starts_with(b"RIFF") && head.get(8..12) == Some(b"WEBP") {
        return Some(ImageKind::Webp);
    }
    None
}

/// 说清「实际认成了什么」—— 拒的时候必须带上这句，否则模型只能瞎猜。
///
/// 存在的意义：`read_image` 失败的常见真相是「用户发来的根本不是图片」
/// （一个 zip、一个 PDF、一段纯文本）。只说 "unsupported image" 等于
/// 什么也没说；点名实际类型，模型（或人）才能立刻改路。
pub fn describe_unknown(head: &[u8]) -> String {
    if head.is_empty() {
        return "empty file".to_owned();
    }
    if head.starts_with(b"PK\x03\x04") || head.starts_with(b"PK\x05\x06") {
        return "zip archive".to_owned();
    }
    if head.starts_with(b"%PDF-") {
        return "pdf document".to_owned();
    }
    if head.starts_with(&[0x1F, 0x8B]) {
        return "gzip stream".to_owned();
    }
    if head.starts_with(b"RIFF") {
        // 认得出容器但认不出格式：把容器里自称的格式报出来。
        return match head.get(8..12) {
            Some(tag) => format!("riff container ({} not webp)", ascii_tag(tag)),
            None => "truncated riff header".to_owned(),
        };
    }
    if crate::nt_workspace::is_binary(head) {
        return "binary (no known image signature)".to_owned();
    }
    format!("text starting with {:?}", printable(head, 12))
}

/// 头部字节的可打印转义（控制字符用 `\xNN`），供错误串安全引用。
fn printable(head: &[u8], cap: usize) -> String {
    head.iter()
        .take(cap)
        .map(|byte| match byte {
            0x20..=0x7E => char::from(*byte).to_string(),
            b'\n' => "\\n".to_owned(),
            b'\t' => "\\t".to_owned(),
            other => format!("\\x{other:02x}"),
        })
        .collect()
}

fn ascii_tag(tag: &[u8]) -> String {
    tag.iter()
        .map(|byte| match byte {
            0x20..=0x7E => char::from(*byte).to_string(),
            other => format!("\\x{other:02x}"),
        })
        .collect()
}

/// 读一张图并封装成多模态部件。
///
/// 顺序是刻意的：**先 jail、再看大小、最后才读进内存** —— 越狱路径不该触发
/// 任何一次 `stat` 之外的动作，巨型文件也不该被读进来才发现超限。
///
/// 返回 `(部件, 格式)`；格式单独返回是为了让调用方在**给人看的**那行文本里
/// 写明「这是一张 jpeg」，而不是让模型自己去猜。
pub fn load_image(workspace: &Path, rel: &str) -> Result<(ImagePart, ImageKind), NtBotError> {
    let rel = rel.trim();
    // 第一道 jail：词法形状（`/etc/passwd`、`~/x`、`a/../../b`、超深路径）。
    crate::nt_workspace::check_rel(rel)?;
    let full = crate::nt_workspace::jail_join(workspace, rel)?;
    let meta = std::fs::metadata(&full)?;
    if !meta.is_file() {
        return Err(NtBotError::Invalid(format!("not a regular file: {rel}")));
    }
    if meta.len() > IMAGE_SIZE_CAP as u64 {
        return Err(NtBotError::Invalid(format!(
            "image too large: {} bytes > {IMAGE_SIZE_CAP} cap (shrink it locally first)",
            meta.len()
        )));
    }
    let bytes = std::fs::read(&full)?;
    // metadata 说多大就是多大不该被信任（并发改写、特殊文件）：
    // 实读后再核一次，宁可拒一张恰好越线的图，不可让内存先被撑开。
    if bytes.len() > IMAGE_SIZE_CAP {
        return Err(NtBotError::Invalid(format!(
            "image too large: {} bytes > {IMAGE_SIZE_CAP} cap (shrink it locally first)",
            bytes.len()
        )));
    }
    let head: &[u8] = match bytes.get(..SNIFF_LEN.min(bytes.len())) {
        Some(head) => head,
        None => &bytes,
    };
    let Some(kind) = detect_kind(head) else {
        return Err(NtBotError::Invalid(format!(
            "not a supported image ({}): supported types are png/jpeg/gif/webp",
            describe_unknown(head)
        )));
    };
    Ok((
        ImagePart {
            media_type: kind.media_type().to_owned(),
            base64: base64_encode(&bytes),
        },
        kind,
    ))
}

/// base64 标准字母表（RFC 4648 §4，带 `+` / `/` 与 `=` padding）。
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// base64 编码（RFC 4648 标准字母表 + padding；无换行）。
///
/// 手写而非引 crate：规则小到能手推验证，且不引入依赖换一堆编译期与审计面。
/// 3 字节一组 → 4 字符；末组不足 3 字节时缺的高位补 `=`。
pub fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3).saturating_mul(4));
    for chunk in bytes.chunks(3) {
        // 末组用 0 补齐成 3 字节（补出来的位随后被 padding 覆盖掉）。
        // 拼成**正常的 24 位值**再逐段取 6 位 —— 若这里已经预移位，取位时
        // 还要再移一次，位序就整个错开了（`"f"` 会被编成 `m` 而不是 `Z`）。
        let b0 = u32::from(chunk.first().copied().unwrap_or(0));
        let b1 = u32::from(chunk.get(1).copied().unwrap_or(0));
        let b2 = u32::from(chunk.get(2).copied().unwrap_or(0));
        let triple = (b0 << 16) | (b1 << 8) | b2;
        for position in 0..4usize {
            let sextet = ((triple >> (18 - 6 * position)) & 0x3F) as usize;
            // `chunk.len() + 1` 个字符有源数据，再往后的必须换成 padding ——
            // 1 字节 → `xx==`，2 字节 → `xxx=`，3 字节 → `xxxx`。
            if position < chunk.len() + 1 {
                if let Some(ch) = B64
                    .get(sextet)
                    .and_then(|byte| char::from_u32(u32::from(*byte)))
                {
                    out.push(ch);
                }
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{ImageKind, base64_encode, describe_unknown, detect_kind, load_image};

    /// 4 字节的最小 PNG 头（真 PNG 还有 IHDR/IDAT，这里只需过签名这一关）。
    const PNG_HEAD: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    const GIF89_HEAD: &[u8] = b"GIF89a";
    /// `RIFF` + 4 字节长度 + `WEBP`。
    fn webp_head() -> Vec<u8> {
        let mut head = b"RIFF".to_vec();
        head.extend_from_slice(&[0x1A, 0x00, 0x00, 0x00]);
        head.extend_from_slice(b"WEBPVP8 ");
        head
    }

    #[test]
    fn magic_bytes_decide_not_extension() {
        assert_eq!(detect_kind(&PNG_HEAD), Some(ImageKind::Png));
        assert_eq!(detect_kind(&[0xFF, 0xD8, 0xFF, 0xE0]), Some(ImageKind::Jpeg));
        assert_eq!(detect_kind(GIF89_HEAD), Some(ImageKind::Gif));
        assert_eq!(detect_kind(b"GIF87a..."), Some(ImageKind::Gif));
        assert_eq!(detect_kind(&webp_head()), Some(ImageKind::Webp));
        assert_eq!(ImageKind::Png.media_type(), "image/png");
        assert_eq!(ImageKind::Jpeg.media_type(), "image/jpeg");
        assert_eq!(ImageKind::Gif.media_type(), "image/gif");
        assert_eq!(ImageKind::Webp.media_type(), "image/webp");
    }

    #[test]
    fn riff_that_is_not_webp_is_not_an_image() {
        // 只有 RIFF 四字节 = AVI/WAV，不是 webp。
        let mut wav = b"RIFF".to_vec();
        wav.extend_from_slice(&[0x24, 0x00, 0x00, 0x00]);
        wav.extend_from_slice(b"WAVEfmt ");
        assert_eq!(detect_kind(&wav), None);
        assert!(describe_unknown(&wav).contains("riff"));
        assert_eq!(detect_kind(b""), None);
        assert_eq!(detect_kind(b"plain text file"), None);
    }

    #[test]
    fn unknown_detection_names_what_it_actually_found() {
        assert_eq!(describe_unknown(&[]), "empty file");
        assert_eq!(describe_unknown(b"PK\x03\x04rest"), "zip archive");
        assert_eq!(describe_unknown(b"%PDF-1.7"), "pdf document");
        assert!(describe_unknown(&[0x00, 0x01, 0x02]).contains("binary"));
        // 文本头部要可读地转义出来，不能把终端控制字符原样吐进错误串。
        assert!(describe_unknown(b"hello world").contains("hello world"));
        assert!(describe_unknown(b"a\nb\tc").contains("\\n"));
    }

    #[test]
    fn base64_matches_rfc4648_vectors() {
        // 已知答案向量：字母表 / 位序 / padding 三处最容易错，一处向量查一处。
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        // 全字节范围：确认 64 个字符的字母表没有错位。
        let all: Vec<u8> = (0..=255u8).collect();
        let encoded = base64_encode(&all);
        assert_eq!(encoded.len(), 344);
        assert!(encoded.is_ascii());
        assert!(!encoded.contains(' '));
    }

    /// 测试内解码器（标准字母表 + padding）——只为往返断言存在，生产代码不需要。
    fn base64_decode(text: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity(text.len() / 4 * 3);
        for group in text.as_bytes().chunks(4) {
            let mut triple: u32 = 0;
            let mut live = 0usize;
            for (position, byte) in group.iter().enumerate() {
                if *byte == b'=' {
                    break;
                }
                let index = super::B64
                    .iter()
                    .position(|candidate| candidate == byte)
                    .unwrap_or(0);
                triple |= (index as u32) << (18 - 6 * position);
                live += 1;
            }
            for position in 0..live.saturating_sub(1) {
                out.push(((triple >> (16 - 8 * position)) & 0xFF) as u8);
            }
        }
        out
    }

    #[test]
    fn base64_roundtrips_every_tail_length() {
        for len in 0..=300usize {
            let raw: Vec<u8> = (0..len).map(|i| (i % 251 + 1) as u8).collect();
            let encoded = base64_encode(&raw);
            assert_eq!(encoded.len() % 4, 0, "len {len} must be 4-aligned");
            assert_eq!(base64_decode(&encoded), raw, "len {len} roundtrip");
        }
    }

    fn workspace_with(name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let dir = crate::nt_testutil::temp_dir(&format!("neobot-vision-{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join(name), bytes).expect("write fixture");
        dir
    }

    #[test]
    fn mislabelled_file_is_refused_with_the_truth() {
        // 名字写着 png，内容是 zip —— 扩展名在这里就该被无视。
        let mut zip = b"PK\x03\x04".to_vec();
        zip.extend_from_slice(b"payload");
        let dir = workspace_with("photo.png", &zip);
        let err = load_image(&dir, "photo.png").expect_err("zip is not an image");
        let text = err.to_string();
        assert!(text.contains("zip archive"), "{text}");
        assert!(text.contains("png/jpeg/gif/webp"), "{text}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mislabelled_extension_yields_the_real_format() {
        // 真 jpeg 叫 .png：类型认魔数，media_type 必须是 image/jpeg。
        let mut jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F'];
        jpeg.extend_from_slice(b"IF\0");
        let dir = workspace_with("lying.png", &jpeg);
        let (part, kind) = load_image(&dir, "lying.png").expect("jpeg");
        assert_eq!(kind, ImageKind::Jpeg);
        assert_eq!(part.media_type, "image/jpeg");
        assert!(part.data_url().starts_with("data:image/jpeg;base64,"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn jail_refuses_paths_outside_the_workspace() {
        let dir = workspace_with("inside.png", &PNG_HEAD);
        for bad in ["../escape.png", "/etc/passwd", "~/key.png", "a/../../b.png", ""] {
            assert!(
                load_image(&dir, bad).is_err(),
                "must refuse '{bad}' (jail is the same law as read_file)"
            );
        }
        // 合法的相对路径读得到。
        assert!(load_image(&dir, "inside.png").is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn size_cap_refuses_before_reading_the_whole_thing() {
        let dir = crate::nt_testutil::temp_dir("vision-toobig");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let big = vec![0u8; super::IMAGE_SIZE_CAP + 1];
        std::fs::write(dir.join("huge.png"), &big).expect("write");
        let err = load_image(&dir, "huge.png").expect_err("over cap must refuse");
        let text = err.to_string();
        assert!(text.contains("too large"), "{text}");
        assert!(text.contains(&super::IMAGE_SIZE_CAP.to_string()), "{text}");
        // 恰好等于上限放行（边界是闭区间，不是 `<`）。
        std::fs::write(dir.join("at-cap.png"), vec![0u8; super::IMAGE_SIZE_CAP]).expect("write");
        // 内容全是 0 不是 png，失败原因必须是「不是图片」而不是「超限」。
        let at_cap = load_image(&dir, "at-cap.png").expect_err("zeros are not an image");
        assert!(at_cap.to_string().contains("not a supported image"), "{at_cap}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn directory_and_missing_file_fail_loudly() {
        let dir = workspace_with("f.png", &PNG_HEAD);
        std::fs::create_dir_all(dir.join("sub")).expect("mkdir");
        assert!(load_image(&dir, "sub").expect_err("dir is not an image").to_string().contains("not a regular file"));
        assert!(load_image(&dir, "nope.png").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

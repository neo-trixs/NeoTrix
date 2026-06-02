use crate::core::knowledge::TaskType;
use std::collections::HashMap;
use std::time::Instant;

// ── MediaFormat: precise format identification ──

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaFormat {
    // Images
    Jpeg, Png, Gif, Svg, Webp, Bmp, Ico, Tiff, Avif, Heic, Raw,
    // Video
    Mp4, Avi, Mov, Mkv, Wmv, Flv, Webm, Ogv, M4v, ThreeGp,
    // Audio
    Mp3, Wav, Flac, Ogg, Aac, Wma, M4a, Opus, Aiff, Amr, Mid,
    // Documents
    Pdf, Doc, Docx, Xls, Xlsx, Ppt, Pptx, Rtf, Epub, Mobi,
    // Archives
    Zip, Tar, Gz, Bz2, Xz, Zst, Rar, SevenZ, Lz4,
    // Code
    Rs, Py, Js, Ts, Tsx, Jsx, Go, Java, C, Cpp, H, Hpp, Swift, Kotlin,
    Scala, Rb, Php, Pl, Lua, R, Sh, Bash, Fish, Ps1, Bat, Ex, Exs,
    Dart, Wasm, Sol, Move, Cairo,
    // Web
    Html, Css, Scss, Less, Vue, Svelte,
    // Data
    Csv, Tsv, Json, Xml, Sql, Parquet, Avro, Proto,
    // Config
    Env, Ini, Cfg, Toml, Yaml, LdJson, Hcl, Terraform,
    // Markup
    Md, Rst, Adoc, Tex, Ltx, Org,
    // Font
    Ttf, Otf, Woff, Woff2, Eot,
    // Model
    Pt, Pth, Pkl, BinModel, Onnx, Safetensors,
    // Generic
    Bin, Txt, Log, Dump, Unknown,
}

impl MediaFormat {
    pub fn name(&self) -> &str {
        match self {
            Self::Jpeg => "jpeg", Self::Png => "png", Self::Gif => "gif",
            Self::Svg => "svg", Self::Webp => "webp", Self::Bmp => "bmp",
            Self::Ico => "ico", Self::Tiff => "tiff", Self::Avif => "avif",
            Self::Heic => "heic", Self::Raw => "raw",
            Self::Mp4 => "mp4", Self::Avi => "avi", Self::Mov => "mov",
            Self::Mkv => "mkv", Self::Wmv => "wmv", Self::Flv => "flv",
            Self::Webm => "webm", Self::Ogv => "ogv", Self::M4v => "m4v",
            Self::ThreeGp => "3gp",
            Self::Mp3 => "mp3", Self::Wav => "wav", Self::Flac => "flac",
            Self::Ogg => "ogg", Self::Aac => "aac", Self::Wma => "wma",
            Self::M4a => "m4a", Self::Opus => "opus", Self::Aiff => "aiff",
            Self::Amr => "amr", Self::Mid => "mid",
            Self::Pdf => "pdf", Self::Doc => "doc", Self::Docx => "docx",
            Self::Xls => "xls", Self::Xlsx => "xlsx", Self::Ppt => "ppt",
            Self::Pptx => "pptx", Self::Rtf => "rtf", Self::Epub => "epub",
            Self::Mobi => "mobi",
            Self::Zip => "zip", Self::Tar => "tar", Self::Gz => "gz",
            Self::Bz2 => "bz2", Self::Xz => "xz", Self::Zst => "zst",
            Self::Rar => "rar", Self::SevenZ => "7z", Self::Lz4 => "lz4",
            Self::Rs => "rs", Self::Py => "py", Self::Js => "js",
            Self::Ts => "ts", Self::Tsx => "tsx", Self::Jsx => "jsx",
            Self::Go => "go", Self::Java => "java", Self::C => "c",
            Self::Cpp => "cpp", Self::H => "h", Self::Hpp => "hpp",
            Self::Swift => "swift", Self::Kotlin => "kt",
            Self::Scala => "scala", Self::Rb => "rb", Self::Php => "php",
            Self::Pl => "pl", Self::Lua => "lua", Self::R => "r",
            Self::Sh => "sh", Self::Bash => "bash", Self::Fish => "fish",
            Self::Ps1 => "ps1", Self::Bat => "bat", Self::Ex => "ex",
            Self::Exs => "exs", Self::Dart => "dart", Self::Wasm => "wasm",
            Self::Sol => "sol", Self::Move => "move", Self::Cairo => "cairo",
            Self::Html => "html", Self::Css => "css", Self::Scss => "scss",
            Self::Less => "less", Self::Vue => "vue", Self::Svelte => "svelte",
            Self::Csv => "csv", Self::Tsv => "tsv", Self::Json => "json",
            Self::Xml => "xml", Self::Sql => "sql", Self::Parquet => "parquet",
            Self::Avro => "avro", Self::Proto => "proto",
            Self::Env => "env", Self::Ini => "ini", Self::Cfg => "cfg",
            Self::Toml => "toml", Self::Yaml => "yaml", Self::LdJson => "ld+json",
            Self::Hcl => "hcl", Self::Terraform => "tf",
            Self::Md => "md", Self::Rst => "rst", Self::Adoc => "adoc",
            Self::Tex => "tex", Self::Ltx => "ltx", Self::Org => "org",
            Self::Ttf => "ttf", Self::Otf => "otf", Self::Woff => "woff",
            Self::Woff2 => "woff2", Self::Eot => "eot",
            Self::Pt => "pt", Self::Pth => "pth", Self::Pkl => "pkl",
            Self::BinModel => "bin_model", Self::Onnx => "onnx",
            Self::Safetensors => "safetensors",
            Self::Bin => "bin", Self::Txt => "txt", Self::Log => "log",
            Self::Dump => "dump", Self::Unknown => "unknown",
        }
    }

    pub fn category(&self) -> &str {
        match self {
            Self::Jpeg | Self::Png | Self::Gif | Self::Svg | Self::Webp
            | Self::Bmp | Self::Ico | Self::Tiff | Self::Avif | Self::Heic
            | Self::Raw => "image",

            Self::Mp4 | Self::Avi | Self::Mov | Self::Mkv | Self::Wmv
            | Self::Flv | Self::Webm | Self::Ogv | Self::M4v | Self::ThreeGp => "video",

            Self::Mp3 | Self::Wav | Self::Flac | Self::Ogg | Self::Aac
            | Self::Wma | Self::M4a | Self::Opus | Self::Aiff | Self::Amr
            | Self::Mid => "audio",

            Self::Pdf | Self::Doc | Self::Docx | Self::Xls | Self::Xlsx
            | Self::Ppt | Self::Pptx | Self::Rtf | Self::Epub | Self::Mobi => "document",

            Self::Zip | Self::Tar | Self::Gz | Self::Bz2 | Self::Xz
            | Self::Zst | Self::Rar | Self::SevenZ | Self::Lz4 => "archive",

            Self::Rs | Self::Py | Self::Js | Self::Ts | Self::Tsx | Self::Jsx
            | Self::Go | Self::Java | Self::C | Self::Cpp | Self::H | Self::Hpp
            | Self::Swift | Self::Kotlin | Self::Scala | Self::Rb | Self::Php
            | Self::Pl | Self::Lua | Self::R | Self::Sh | Self::Bash | Self::Fish
            | Self::Ps1 | Self::Bat | Self::Ex | Self::Exs | Self::Dart
            | Self::Wasm | Self::Sol | Self::Move | Self::Cairo => "code",

            Self::Html | Self::Css | Self::Scss | Self::Less | Self::Vue
            | Self::Svelte => "web",

            Self::Csv | Self::Tsv | Self::Json | Self::Xml | Self::Sql
            | Self::Parquet | Self::Avro | Self::Proto => "data",

            Self::Env | Self::Ini | Self::Cfg | Self::Toml | Self::Yaml
            | Self::LdJson | Self::Hcl | Self::Terraform => "config",

            Self::Md | Self::Rst | Self::Adoc | Self::Tex | Self::Ltx
            | Self::Org => "markup",

            Self::Ttf | Self::Otf | Self::Woff | Self::Woff2 | Self::Eot => "font",

            Self::Pt | Self::Pth | Self::Pkl | Self::BinModel | Self::Onnx
            | Self::Safetensors => "model",

            Self::Bin | Self::Txt | Self::Log | Self::Dump | Self::Unknown => "other",
        }
    }
}

// ── InputType with enriched classification ──

#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Image(MediaFormat),
    Video(MediaFormat),
    Audio(MediaFormat),
    Document(MediaFormat),
    Archive(MediaFormat),
    Code(MediaFormat),
    Web(MediaFormat),
    Data(MediaFormat),
    Config(MediaFormat),
    Markup(MediaFormat),
    Font(MediaFormat),
    Model(MediaFormat),
    Binary,
    Text,
    Mixed,
    Unknown,
}

impl InputType {
    pub fn name(&self) -> &str {
        match self {
            Self::Image(f) => f.name(),
            Self::Video(f) => f.name(),
            Self::Audio(f) => f.name(),
            Self::Document(f) => f.name(),
            Self::Archive(f) => f.name(),
            Self::Code(f) => f.name(),
            Self::Web(f) => f.name(),
            Self::Data(f) => f.name(),
            Self::Config(f) => f.name(),
            Self::Markup(f) => f.name(),
            Self::Font(f) => f.name(),
            Self::Model(f) => f.name(),
            Self::Binary => "binary",
            Self::Text => "text",
            Self::Mixed => "mixed",
            Self::Unknown => "unknown",
        }
    }

    pub fn category(&self) -> &str {
        match self {
            Self::Image(_) => "image",
            Self::Video(_) => "video",
            Self::Audio(_) => "audio",
            Self::Document(_) => "document",
            Self::Archive(_) => "archive",
            Self::Code(_) => "code",
            Self::Web(_) => "web",
            Self::Data(_) => "data",
            Self::Config(_) => "config",
            Self::Markup(_) => "markup",
            Self::Font(_) => "font",
            Self::Model(_) => "model",
            Self::Binary => "binary",
            Self::Text => "text",
            Self::Mixed => "mixed",
            Self::Unknown => "unknown",
        }
    }

    pub fn format(&self) -> Option<MediaFormat> {
        match self {
            Self::Image(f) | Self::Video(f) | Self::Audio(f)
            | Self::Document(f) | Self::Archive(f) | Self::Code(f)
            | Self::Web(f) | Self::Data(f) | Self::Config(f)
            | Self::Markup(f) | Self::Font(f) | Self::Model(f) => Some(*f),
            _ => None,
        }
    }

    pub fn is_media(&self) -> bool {
        matches!(self, Self::Image(_) | Self::Video(_) | Self::Audio(_))
    }

    pub fn is_document(&self) -> bool {
        matches!(self, Self::Document(_))
    }

    /// Magic byte detection: identify format from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Self {
        // Images
        if data.starts_with(b"\xff\xd8\xff") { return Self::Image(MediaFormat::Jpeg); }
        if data.starts_with(b"\x89PNG\r\n\x1a\n") { return Self::Image(MediaFormat::Png); }
        if data.starts_with(b"GIF87a") || data.starts_with(b"GIF89a") { return Self::Image(MediaFormat::Gif); }
        if data.starts_with(b"RIFF") && data.windows(4).any(|w| w == b"WEBP") { return Self::Image(MediaFormat::Webp); }
        if data.starts_with(b"BM") { return Self::Image(MediaFormat::Bmp); }
        if data.starts_with(b"\x00\x00\x01\x00") || data.starts_with(b"\x00\x00\x02\x00") { return Self::Image(MediaFormat::Ico); }
        if data.starts_with(b"MM\x00*") || data.starts_with(b"II*\x00") { return Self::Image(MediaFormat::Tiff); }
        if data.starts_with(b"<?xml") || data.starts_with(b"<svg") { return Self::Image(MediaFormat::Svg); }
        if data.starts_with(b"\x00\x00\x00\x1cftyp") || data.starts_with(b"\x00\x00\x00\x20ftyp") { return Self::Image(MediaFormat::Avif); }
        if data.starts_with(b"\x00\x00\x00\x18ftyp") || data.len() > 8 && &data[4..8] == b"ftyp" {
            let ftyp = String::from_utf8_lossy(&data[8..12]);
            match ftyp.as_ref() {
                "isom" | "mp42" | "mp41" | "avc1" => return Self::Video(MediaFormat::Mp4),
                "M4V " | "M4V" | "m4v " => return Self::Video(MediaFormat::M4v),
                "3gp" | "3g2a" => return Self::Video(MediaFormat::ThreeGp),
                "M4A " | "M4A" => return Self::Audio(MediaFormat::M4a),
                _ => {}
            }
        }

        // Video
        if data.starts_with(b"RIFF") && data.len() > 8 && &data[8..12] == b"AVI " { return Self::Video(MediaFormat::Avi); }
        if data.starts_with(b"\x00\x00\x00\x14ftypqt") || data.starts_with(b"\x00\x00\x00\x1cftypqt") { return Self::Video(MediaFormat::Mov); }
        if data.starts_with(b"\x1aE\xdf\xa3") { return Self::Video(MediaFormat::Mkv); }
        if data.starts_with(b"\x30\x26\xb2\x75\x8e\x66\xcf\x11") { return Self::Video(MediaFormat::Wmv); }
        if data.starts_with(b"FLV\x01") { return Self::Video(MediaFormat::Flv); }
        if data.starts_with(b"\x1a\x45\xdf\xa3") && data.windows(4).any(|w| w == b"webm") { return Self::Video(MediaFormat::Webm); }
        if data.starts_with(b"OggS") && data.len() > 28 && data[28] == 0x01 { return Self::Video(MediaFormat::Ogv); }

        // Audio
        if data.starts_with(b"ID3") || (data.len() > 1 && data[0] == 0xff && (data[1] & 0xe0) == 0xe0) { return Self::Audio(MediaFormat::Mp3); }
        if data.starts_with(b"RIFF") && data.len() > 8 && &data[8..12] == b"WAVE" { return Self::Audio(MediaFormat::Wav); }
        if data.starts_with(b"fLaC") { return Self::Audio(MediaFormat::Flac); }
        if data.starts_with(b"OggS") && data.len() > 28 && data[28] == 0x02 { return Self::Audio(MediaFormat::Ogg); }
        if data.starts_with(b"\xff\xf1") || data.starts_with(b"\xff\xf9") { return Self::Audio(MediaFormat::Aac); }
        if data.starts_with(b"\x30\x26\xb2\x75\x8e\x66\xcf\x11\xa6\xd9\x00\xaa\x00\x62\xce\x6c") { return Self::Audio(MediaFormat::Wma); }
        if data.starts_with(b"OggS") && data.len() > 28 && data[28] == 0x03 { return Self::Audio(MediaFormat::Opus); }
        if data.starts_with(b"FORM") && data.len() > 8 && &data[8..12] == b"AIFF" { return Self::Audio(MediaFormat::Aiff); }
        if data.starts_with(b"#!AMR") || data.starts_with(b"\x23\x21\x41\x4d\x52\x0a") { return Self::Audio(MediaFormat::Amr); }
        if data.starts_with(b"MThd") { return Self::Audio(MediaFormat::Mid); }

        // Documents
        if data.starts_with(b"%PDF") { return Self::Document(MediaFormat::Pdf); }
        if data.starts_with(b"PK\x03\x04") && data.len() > 30 {
            let name_bytes = &data[30..data.len().min(38)];
            let name = String::from_utf8_lossy(name_bytes);
            if name.starts_with("word/") { return Self::Document(MediaFormat::Docx); }
            if name.starts_with("xl/") { return Self::Document(MediaFormat::Xlsx); }
            if name.starts_with("ppt/") { return Self::Document(MediaFormat::Pptx); }
            if name == "mimetype" || data.len() > 60 && data[30..58].windows(4).any(|w| w == b"epub") { return Self::Document(MediaFormat::Epub); }
        }
        if data.starts_with(b"\xd0\xcf\x11\xe0\xa1\xb1\x1a\xe1") {
            let clsid = &data[24..40];
            if clsid.starts_with(b"\xcf\x11\x89\x9a\xc8\x02\x00\x00\x00\x00\x00\x00\xe0\xc9\x00\x00") { return Self::Document(MediaFormat::Ppt); }
            if clsid.starts_with(b"\xcf\x11\x0a\x6b\x08\x02\x00\x00\x00\x00\x00\x00\xe0\xc9\x00\x00") { return Self::Document(MediaFormat::Xls); }
            if clsid.starts_with(b"\xcf\x11\xea\x79\x08\x02\x00\x00\x00\x00\x00\x00\xe0\xc9\x00\x00") { return Self::Document(MediaFormat::Doc); }
        }
        if data.starts_with(b"{\\rtf") { return Self::Document(MediaFormat::Rtf); }
        if data.starts_with(b"PK\x03\x04") && data.len() > 58 {
            let name_bytes = &data[30..data.len().min(80)];
            let name = String::from_utf8_lossy(name_bytes);
            if name.starts_with("mimetype") && data.windows(20).any(|w| w == b"application/epub+zip") { return Self::Document(MediaFormat::Epub); }
            if name.contains(".opf") { return Self::Document(MediaFormat::Epub); }
        }

        // Archives
        if data.starts_with(b"PK\x03\x04") { return Self::Archive(MediaFormat::Zip); }
        if data.starts_with(b"\x1f\x8b\x08") { return Self::Archive(MediaFormat::Gz); }
        if data.starts_with(b"BZh") { return Self::Archive(MediaFormat::Bz2); }
        if data.starts_with(b"\xfd7zXZ\x00") { return Self::Archive(MediaFormat::Xz); }
        if data.starts_with(b"\x28\xb5\x2f\xfd") { return Self::Archive(MediaFormat::Zst); }
        if data.starts_with(b"Rar!\x1a\x07\x01\x00") || data.starts_with(b"Rar!\x1a\x07\x00") { return Self::Archive(MediaFormat::Rar); }
        if data.starts_with(b"7z\xbc\xaf\x27\x1c") { return Self::Archive(MediaFormat::SevenZ); }
        if data.starts_with(b"\x04\x22\x4d\x18") { return Self::Archive(MediaFormat::Lz4); }

        // Fonts
        if data.starts_with(b"\x00\x01\x00\x00\x00") { return Self::Font(MediaFormat::Ttf); }
        if data.starts_with(b"OTTO") { return Self::Font(MediaFormat::Otf); }
        if data.starts_with(b"wOFF") { return Self::Font(MediaFormat::Woff); }
        if data.starts_with(b"wOF2") { return Self::Font(MediaFormat::Woff2); }

        // Models
        if data.len() > 4 && data[0..4].iter().all(|&b| b.is_ascii_alphanumeric()) {
            let header = String::from_utf8_lossy(&data[..data.len().min(64)]);
            if header.contains("pytorch") || header.contains("torch") { return Self::Model(MediaFormat::Pt); }
            if header.contains("onnx") { return Self::Model(MediaFormat::Onnx); }
            if header.contains("safetensors") { return Self::Model(MediaFormat::Safetensors); }
            if header.contains("pickle") || header.contains("PKL") { return Self::Model(MediaFormat::Pkl); }
        }
        if data.starts_with(b"\x80\x02") || data.starts_with(b"\x80\x03") || data.starts_with(b"\x80\x04") || data.starts_with(b"\x80\x05") { return Self::Model(MediaFormat::Pkl); }

        // Wasm
        if data.starts_with(b"\x00asm") { return Self::Code(MediaFormat::Wasm); }

        // Binary fallback
        if data.iter().any(|&b| b == 0x00) {
            return Self::Binary;
        }

        // Text — check for UTF-8 validity
        if std::str::from_utf8(data).is_ok() {
            return Self::Text;
        }

        Self::Binary
    }

    /// Infer input type from task description + has_image flag + optional metadata.
    pub fn infer(task: &str, has_image: bool, metadata: Option<&FileMetadata>) -> Self {
        if let Some(meta) = metadata {
            if meta.format != MediaFormat::Unknown {
                let media = match meta.format.category() {
                    "image" => Self::Image(meta.format),
                    "video" => Self::Video(meta.format),
                    "audio" => Self::Audio(meta.format),
                    "document" => Self::Document(meta.format),
                    "archive" => Self::Archive(meta.format),
                    "code" => Self::Code(meta.format),
                    "web" => Self::Web(meta.format),
                    "data" => Self::Data(meta.format),
                    "config" => Self::Config(meta.format),
                    "markup" => Self::Markup(meta.format),
                    "font" => Self::Font(meta.format),
                    "model" => Self::Model(meta.format),
                    _ => return Self::Unknown,
                };
                return media;
            }
        }

        if has_image {
            return Self::Image(MediaFormat::Unknown);
        }

        let lower = task.to_lowercase();

        // Code file extensions
        let code_extensions = &[
            ".rs", ".py", ".js", ".ts", ".tsx", ".jsx", ".go", ".java",
            ".c", ".cpp", ".cxx", ".cc", ".h", ".hpp", ".hxx", ".swift",
            ".kt", ".kts", ".scala", ".rb", ".php", ".php3", ".php4", ".php5",
            ".phtml", ".pl", ".pm", ".lua", ".r", ".R", ".sh", ".bash",
            ".zsh", ".fish", ".ps1", ".bat", ".cmd", ".ex", ".exs",
            ".dart", ".wasm", ".sol", ".move", ".cairo", ".zig", ".nim",
            ".cr", ".elm", ".erl", ".hrl", ".hs", ".lhs", ".clj", ".cljs",
            ".edn", ".ml", ".mli", ".fs", ".fsx", ".v", ".vhdl", ".vhd",
            ".s", ".asm", ".S", ".inc", ".m", ".mm",
        ];
        let config_extensions = &[
            ".env", ".envrc", ".ini", ".cfg", ".conf", ".toml", ".yaml",
            ".yml", ".hcl", ".tf", ".tfvars", ".editorconfig", ".gitconfig",
            ".npmrc", ".yarnrc", ".prettierrc", ".eslintrc", ".babelrc",
        ];
        let data_extensions = &[
            ".csv", ".tsv", ".json", ".xml", ".xsd", ".dtd", ".sql",
            ".sqlite", ".db", ".parquet", ".avro", ".proto",
        ];
        let web_extensions = &[".html", ".htm", ".css", ".scss", ".sass", ".less", ".vue", ".svelte"];
        let markup_extensions = &[".md", ".markdown", ".rst", ".adoc", ".asciidoc", ".tex", ".ltx", ".org", ".txt"];
        let image_extensions = &[".jpg", ".jpeg", ".png", ".gif", ".svg", ".webp", ".bmp", ".ico", ".tiff", ".tif", ".avif", ".heic", ".heif", ".raw", ".cr2", ".nef", ".arw"];
        let video_extensions = &[".mp4", ".avi", ".mov", ".mkv", ".wmv", ".flv", ".webm", ".ogv", ".m4v", ".3gp", ".mpeg", ".mpg"];
        let audio_extensions = &[".mp3", ".wav", ".flac", ".ogg", ".aac", ".wma", ".m4a", ".opus", ".aiff", ".amr", ".mid", ".midi"];
        let document_extensions = &[".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx", ".rtf", ".epub", ".mobi", ".azw3"];
        let archive_extensions = &[".zip", ".tar", ".gz", ".tgz", ".bz2", ".tbz2", ".xz", ".txz", ".zst", ".rar", ".7z", ".lz4", ".z", ".lz", ".arj", ".cab"];
        let font_extensions = &[".ttf", ".otf", ".woff", ".woff2", ".eot"];
        let model_extensions = &[".pt", ".pth", ".pkl", ".onnx", ".safetensors"];

        // Check if task contains a file reference
        let has_file_ref = lower.contains("file:") || lower.starts_with("file ")
            || lower.starts_with("open ") || lower.starts_with("read ")
            || (lower.contains('/') && lower.contains('.'));

        let any_word_ends_with = |exts: &[&str]| -> bool {
            lower.split_whitespace().any(|w| exts.iter().any(|e| w.ends_with(e)))
        };

        if has_file_ref || any_word_ends_with(image_extensions) || any_word_ends_with(video_extensions)
            || any_word_ends_with(audio_extensions) || any_word_ends_with(document_extensions)
            || any_word_ends_with(archive_extensions) || any_word_ends_with(code_extensions)
            || any_word_ends_with(config_extensions) || any_word_ends_with(data_extensions)
            || any_word_ends_with(web_extensions) || any_word_ends_with(markup_extensions)
            || any_word_ends_with(font_extensions) || any_word_ends_with(model_extensions)
        {
            let ext = lower.split_whitespace()
                .flat_map(|w| {
                    image_extensions.iter().chain(video_extensions).chain(audio_extensions)
                        .chain(document_extensions).chain(archive_extensions)
                        .chain(code_extensions).chain(config_extensions)
                        .chain(data_extensions).chain(web_extensions).chain(markup_extensions)
                        .chain(font_extensions).chain(model_extensions)
                        .find(|e| w.ends_with(**e))
                })
                .next();
            if let Some(found) = ext {
                return match to_media_format(found).category() {
                    "image" => Self::Image(to_media_format(found)),
                    "video" => Self::Video(to_media_format(found)),
                    "audio" => Self::Audio(to_media_format(found)),
                    "document" => Self::Document(to_media_format(found)),
                    "archive" => Self::Archive(to_media_format(found)),
                    "code" => Self::Code(to_media_format(found)),
                    "web" => Self::Web(to_media_format(found)),
                    "data" => Self::Data(to_media_format(found)),
                    "config" => Self::Config(to_media_format(found)),
                    "markup" => Self::Markup(to_media_format(found)),
                    "font" => Self::Font(to_media_format(found)),
                    "model" => Self::Model(to_media_format(found)),
                    _ => Self::Binary,
                };
            }
            return Self::Binary;
        }

        // Code blocks in task text
        if lower.contains("```") || lower.contains("fn ") || lower.contains("impl ")
            || lower.contains("def ") || lower.contains("const ") || lower.contains("let ")
            || lower.contains("import ") || lower.contains("package ")
        {
            return Self::Code(MediaFormat::Unknown);
        }

        // Task describes media
        if lower.contains("image") || lower.contains("picture") || lower.contains("photo")
            || lower.contains("screenshot") || lower.contains("diagram")
            || lower.contains("chart") || lower.contains("graph")
            || lower.contains("architecture")
        {
            return Self::Image(MediaFormat::Unknown);
        }
        if lower.contains("video") || lower.contains("movie") || lower.contains("clip")
            || lower.contains("animation") || lower.contains("footage")
        {
            return Self::Video(MediaFormat::Unknown);
        }
        if lower.contains("audio") || lower.contains("sound") || lower.contains("music")
            || lower.contains("speech") || lower.contains("recording")
            || lower.contains("podcast") || lower.contains("nt_act_voice")
        {
            return Self::Audio(MediaFormat::Unknown);
        }
        if lower.contains("pdf") || lower.contains("document") || lower.contains("spreadsheet")
            || lower.contains("presentation") || lower.contains("slide")
            || lower.contains("report")
        {
            return Self::Document(MediaFormat::Unknown);
        }

        Self::Text
    }
}

fn to_media_format(ext: &str) -> MediaFormat {
    match ext {
        ".rs" => MediaFormat::Rs, ".py" => MediaFormat::Py,
        ".js" => MediaFormat::Js, ".ts" => MediaFormat::Ts,
        ".tsx" => MediaFormat::Tsx, ".jsx" => MediaFormat::Jsx,
        ".go" => MediaFormat::Go, ".java" => MediaFormat::Java,
        ".c" => MediaFormat::C, ".cpp" | ".cxx" | ".cc" => MediaFormat::Cpp,
        ".h" => MediaFormat::H, ".hpp" | ".hxx" => MediaFormat::Hpp,
        ".swift" => MediaFormat::Swift, ".kt" | ".kts" => MediaFormat::Kotlin,
        ".scala" => MediaFormat::Scala, ".rb" => MediaFormat::Rb,
        ".php" | ".php3" | ".php4" | ".php5" | ".phtml" => MediaFormat::Php,
        ".pl" | ".pm" => MediaFormat::Pl, ".lua" => MediaFormat::Lua,
        ".r" | ".R" => MediaFormat::R, ".sh" | ".bash" | ".zsh" => MediaFormat::Sh,
        ".fish" => MediaFormat::Fish, ".ps1" => MediaFormat::Ps1,
        ".bat" | ".cmd" => MediaFormat::Bat, ".ex" => MediaFormat::Ex,
        ".exs" => MediaFormat::Exs, ".dart" => MediaFormat::Dart,
        ".wasm" => MediaFormat::Wasm, ".sol" => MediaFormat::Sol,
        ".move" => MediaFormat::Move, ".cairo" => MediaFormat::Cairo,
        ".html" | ".htm" => MediaFormat::Html, ".css" => MediaFormat::Css,
        ".scss" | ".sass" => MediaFormat::Scss, ".less" => MediaFormat::Less,
        ".vue" => MediaFormat::Vue, ".svelte" => MediaFormat::Svelte,
        ".csv" => MediaFormat::Csv, ".tsv" => MediaFormat::Tsv,
        ".json" => MediaFormat::Json, ".xml" => MediaFormat::Xml,
        ".sql" => MediaFormat::Sql, ".parquet" => MediaFormat::Parquet,
        ".avro" => MediaFormat::Avro, ".proto" => MediaFormat::Proto,
        ".env" | ".envrc" => MediaFormat::Env, ".ini" => MediaFormat::Ini,
        ".cfg" | ".conf" => MediaFormat::Cfg, ".toml" => MediaFormat::Toml,
        ".yaml" | ".yml" => MediaFormat::Yaml, ".hcl" => MediaFormat::Hcl,
        ".tf" | ".tfvars" => MediaFormat::Terraform,
        ".md" | ".markdown" => MediaFormat::Md, ".rst" => MediaFormat::Rst,
        ".adoc" | ".asciidoc" => MediaFormat::Adoc,
        ".tex" => MediaFormat::Tex, ".ltx" => MediaFormat::Ltx,
        ".org" => MediaFormat::Org,
        ".jpg" | ".jpeg" => MediaFormat::Jpeg, ".png" => MediaFormat::Png,
        ".gif" => MediaFormat::Gif, ".svg" => MediaFormat::Svg,
        ".webp" => MediaFormat::Webp, ".bmp" => MediaFormat::Bmp,
        ".ico" => MediaFormat::Ico, ".tiff" | ".tif" => MediaFormat::Tiff,
        ".avif" => MediaFormat::Avif, ".heic" | ".heif" => MediaFormat::Heic,
        ".raw" | ".cr2" | ".nef" | ".arw" => MediaFormat::Raw,
        ".mp4" => MediaFormat::Mp4, ".avi" => MediaFormat::Avi,
        ".mov" => MediaFormat::Mov, ".mkv" => MediaFormat::Mkv,
        ".wmv" => MediaFormat::Wmv, ".flv" => MediaFormat::Flv,
        ".webm" => MediaFormat::Webm, ".ogv" => MediaFormat::Ogv,
        ".m4v" => MediaFormat::M4v, ".3gp" => MediaFormat::ThreeGp,
        ".mpeg" | ".mpg" | ".mts" => MediaFormat::Mp4,
        ".mp3" => MediaFormat::Mp3, ".wav" => MediaFormat::Wav,
        ".flac" => MediaFormat::Flac, ".ogg" => MediaFormat::Ogg,
        ".aac" => MediaFormat::Aac, ".wma" => MediaFormat::Wma,
        ".m4a" => MediaFormat::M4a, ".opus" => MediaFormat::Opus,
        ".aiff" => MediaFormat::Aiff, ".amr" => MediaFormat::Amr,
        ".mid" | ".midi" => MediaFormat::Mid,
        ".pdf" => MediaFormat::Pdf, ".doc" => MediaFormat::Doc,
        ".docx" => MediaFormat::Docx, ".xls" => MediaFormat::Xls,
        ".xlsx" => MediaFormat::Xlsx, ".ppt" => MediaFormat::Ppt,
        ".pptx" => MediaFormat::Pptx, ".rtf" => MediaFormat::Rtf,
        ".epub" => MediaFormat::Epub, ".mobi" | ".azw3" => MediaFormat::Mobi,
        ".zip" => MediaFormat::Zip, ".tar" => MediaFormat::Tar,
        ".gz" | ".tgz" => MediaFormat::Gz, ".bz2" | ".tbz2" => MediaFormat::Bz2,
        ".xz" | ".txz" => MediaFormat::Xz, ".zst" => MediaFormat::Zst,
        ".rar" => MediaFormat::Rar, ".7z" => MediaFormat::SevenZ,
        ".lz4" => MediaFormat::Lz4,
        ".ttf" => MediaFormat::Ttf, ".otf" => MediaFormat::Otf,
        ".woff" => MediaFormat::Woff, ".woff2" => MediaFormat::Woff2,
        ".eot" => MediaFormat::Eot,
        ".pt" | ".pth" => MediaFormat::Pt, ".pkl" => MediaFormat::Pkl,
        ".onnx" => MediaFormat::Onnx, ".safetensors" => MediaFormat::Safetensors,
        ".txt" | ".log" => MediaFormat::Txt,
        _ => MediaFormat::Unknown,
    }
}

// ── FileMetadata: rich file/stream metadata ──

#[derive(Debug, Clone)]
pub struct FileMetadata {
    pub format: MediaFormat,
    pub size_bytes: u64,
    pub dimensions: Option<(u32, u32)>,
    pub duration_ms: Option<u64>,
    pub codec: Option<String>,
    pub sample_rate: Option<u32>,
    pub channels: Option<u8>,
    pub bitrate: Option<u64>,
    pub has_alpha: bool,
    pub pages: Option<u32>,
    pub compression: Option<String>,
}

impl FileMetadata {
    pub fn empty() -> Self {
        Self {
            format: MediaFormat::Unknown,
            size_bytes: 0,
            dimensions: None,
            duration_ms: None,
            codec: None,
            sample_rate: None,
            channels: None,
            bitrate: None,
            has_alpha: false,
            pages: None,
            compression: None,
        }
    }

    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        parts.push(self.format.name().to_string());
        if self.size_bytes > 0 {
            if self.size_bytes > 1024 * 1024 {
                parts.push(format!("{:.1}MB", self.size_bytes as f64 / (1024.0 * 1024.0)));
            } else if self.size_bytes > 1024 {
                parts.push(format!("{:.1}KB", self.size_bytes as f64 / 1024.0));
            } else {
                parts.push(format!("{}B", self.size_bytes));
            }
        }
        if let Some((w, h)) = self.dimensions {
            parts.push(format!("{}x{}", w, h));
        }
        if let Some(ms) = self.duration_ms {
            if ms >= 60_000 {
                parts.push(format!("{}m{:02}s", ms / 60_000, (ms / 1000) % 60));
            } else {
                parts.push(format!("{:.1}s", ms as f64 / 1000.0));
            }
        }
        if let Some(sr) = self.sample_rate {
            parts.push(format!("{}Hz", sr));
        }
        if let Some(ch) = self.channels {
            parts.push(format!("{}ch", ch));
        }
        if self.has_alpha {
            parts.push("alpha".to_string());
        }
        parts.join(" ")
    }
}

impl Default for FileMetadata {
    fn default() -> Self { Self::empty() }
}

/// Extract metadata from raw bytes by inspecting the content.
/// This does basic header analysis — real metadata requires external tools.
#[allow(dead_code)]
pub fn extract_metadata(data: &[u8], input_type: &InputType) -> FileMetadata {
    let mut meta = FileMetadata::empty();
    if let Some(fmt) = input_type.format() {
        meta.format = fmt;
    }
    meta.size_bytes = data.len() as u64;

    match input_type {
        InputType::Image(f) => {
            match f {
                MediaFormat::Png => {
                    // PNG: IHDR chunk at byte 16: width (4 bytes), height (4 bytes)
                    if data.len() > 24 {
                        let w = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
                        let h = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
                        meta.dimensions = Some((w, h));
                        meta.has_alpha = data.len() > 32 && data[32] == 0x06; // RGBA color type
                    }
                }
                MediaFormat::Jpeg => {
                    // Parse SOF markers for dimensions
                    if let Some((w, h)) = parse_jpeg_dimensions(data) {
                        meta.dimensions = Some((w, h));
                    }
                }
                MediaFormat::Gif => {
                    if data.len() > 10 {
                        let w = u16::from_le_bytes([data[6], data[7]]);
                        let h = u16::from_le_bytes([data[8], data[9]]);
                        meta.dimensions = Some((w as u32, h as u32));
                    }
                }
                MediaFormat::Bmp => {
                    if data.len() > 24 {
                        let w = u32::from_le_bytes([data[18], data[19], data[20], data[21]]);
                        let h = u32::from_le_bytes([data[22], data[23], data[24], data[25]]);
                        meta.dimensions = Some((w, h));
                    }
                }
                MediaFormat::Webp => {
                    if data.len() > 30 {
                        let w = u32::from_le_bytes([data[24], data[25], data[26], data[27]]) & 0x3fff;
                        let h = u32::from_le_bytes([data[28], data[29], data[30], data[31]]) & 0x3fff;
                        meta.dimensions = Some((w, h));
                    }
                }
                MediaFormat::Tiff => {
                    if let Some((w, h)) = parse_tiff_dimensions(data) {
                        meta.dimensions = Some((w, h));
                    }
                }
                _ => {}
            }
        }
        InputType::Video(_) => {
            // Parse MP4/AVI for basic dimensions when possible
            if data.len() > 40 {
                if data[4..8] == [0x66, 0x74, 0x79, 0x70] {
                    // MP4 container — look for avcC or hvcC for dimensions
                    if let Some((w, h)) = parse_mp4_dimensions(data) {
                        meta.dimensions = Some((w, h));
                    }
                }
            }
        }
        InputType::Audio(f) => {
            match f {
                MediaFormat::Wav => {
                    if data.len() > 44 {
                        let sample_rate = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
                        let channels = data[22];
                        meta.sample_rate = Some(sample_rate);
                        meta.channels = Some(channels);
                        // Calculate duration from data chunk
                        let data_size = u32::from_le_bytes([data[40], data[41], data[42], data[43]]);
                        if sample_rate > 0 && channels > 0 {
                            let bytes_per_sec = sample_rate * channels as u32 * (data[34] as u32 / 8).max(1);
                            if bytes_per_sec > 0 {
                                meta.duration_ms = Some((data_size as u64 * 1000) / bytes_per_sec as u64);
                            }
                        }
                    }
                }
                MediaFormat::Mp3 => {
                    if data.len() > 6 && data[0] == 0x49 && data[1] == 0x44 && data[2] == 0x33 {
                        // ID3v2 header
                        let size = ((data[6] as u32) << 21) | ((data[7] as u32) << 14)
                            | ((data[8] as u32) << 7) | (data[9] as u32);
                        meta.duration_ms = Some(0); // placeholder — needs frame parsing
                        if data.len() > 10 + size as usize + 1 {
                            // Try to read bitrate from first frame header
                            let frame_start = 10 + size as usize;
                            if frame_start + 1 < data.len() {
                                let bitrate_idx = (data[frame_start + 2] >> 4) as usize;
                                // MPEG1 Layer3 bitrates in kbps
                                const BITRATES: [u32; 15] = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320];
                                if bitrate_idx < 15 && bitrate_idx > 0 {
                                    meta.bitrate = Some(BITRATES[bitrate_idx] as u64 * 1000);
                                }
                            }
                        }
                    }
                }
                MediaFormat::Flac => {
                    if data.len() > 30 {
                        let sample_rate = u32::from_be_bytes([0, data[22], data[23], data[24]]) >> 4;
                        let channels = ((data[24] >> 1) & 0x07) + 1;
                        let total_samples = u64::from_be_bytes([
                            0, 0, data[18], data[19], data[20], data[21],
                            data[22] & 0x0f, (data[23] >> 4) & 0x0f,
                        ]);
                        meta.sample_rate = Some(sample_rate);
                        meta.channels = Some(channels as u8);
                        if sample_rate > 0 {
                            meta.duration_ms = Some((total_samples * 1000) / sample_rate as u64);
                        }
                    }
                }
                _ => {}
            }
        }
        InputType::Document(f) => {
            if *f == MediaFormat::Pdf {
                // Count PDF pages
                if let Ok(text) = std::str::from_utf8(data) {
                    let count = text.split_whitespace()
                        .filter(|w| *w == "/Page" || *w == "/Page\n" || w.starts_with("/Page\n"))
                        .count();
                    if count > 0 {
                        meta.pages = Some(count as u32);
                    }
                }
            }
        }
        InputType::Archive(f) => {
            meta.compression = Some(match f {
                MediaFormat::Zip => "deflate",
                MediaFormat::Gz => "gzip",
                MediaFormat::Bz2 => "bzip2",
                MediaFormat::Xz => "lzma2",
                MediaFormat::Zst => "zstd",
                MediaFormat::Rar => "rar",
                MediaFormat::SevenZ => "lzma",
                MediaFormat::Lz4 => "lz4",
                _ => "unknown",
            }.to_string());
        }
        _ => {}
    }

    meta
}

#[allow(dead_code)]
fn parse_jpeg_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    let mut pos = 2;
    while pos + 4 < data.len() {
        if data[pos] == 0xff {
            let marker = data[pos + 1];
            if marker == 0xc0 || marker == 0xc1 || marker == 0xc2 {
                if pos + 9 < data.len() {
                    let h = u16::from_be_bytes([data[pos + 5], data[pos + 6]]);
                    let w = u16::from_be_bytes([data[pos + 7], data[pos + 8]]);
                    return Some((w as u32, h as u32));
                }
            }
            if marker != 0xd9 && marker != 0xda {
                let seg_size = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);
                pos += 2 + seg_size as usize;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    None
}

#[allow(dead_code)]
fn parse_tiff_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    if data.len() < 8 { return None; }
    let little_endian = data[0] == 0x49;
    let offset = if little_endian {
        u32::from_le_bytes([data[4], data[5], data[6], data[7]])
    } else {
        u32::from_be_bytes([data[4], data[5], data[6], data[7]])
    };
    if offset as usize + 2 > data.len() { return None; }
    let num_tags = if little_endian {
        u16::from_le_bytes([data[offset as usize], data[offset as usize + 1]])
    } else {
        u16::from_be_bytes([data[offset as usize], data[offset as usize + 1]])
    };
    let mut width = None;
    let mut height = None;
    for i in 0..num_tags.min(50) {
        let tag_pos = offset as usize + 2 + i as usize * 12;
        if tag_pos + 12 > data.len() { break; }
        let tag = if little_endian {
            u16::from_le_bytes([data[tag_pos], data[tag_pos + 1]])
        } else {
            u16::from_be_bytes([data[tag_pos], data[tag_pos + 1]])
        };
        let value = if little_endian {
            u32::from_le_bytes([data[tag_pos + 8], data[tag_pos + 9], data[tag_pos + 10], data[tag_pos + 11]])
        } else {
            u32::from_be_bytes([data[tag_pos + 8], data[tag_pos + 9], data[tag_pos + 10], data[tag_pos + 11]])
        };
        match tag {
            256 => width = Some(value),
            257 => height = Some(value),
            _ => {}
        }
    }
    match (width, height) {
        (Some(w), Some(h)) => Some((w, h)),
        _ => None,
    }
}

#[allow(dead_code)]
fn parse_mp4_dimensions(data: &[u8]) -> Option<(u32, u32)> {
    // Search for avcC or hvcC boxes which contain SPS with dimensions
    // Simplified: look for avc1/hvc1 box which stores width/height at fixed offsets
    let mut pos = 0;
    while pos + 8 < data.len() {
        let box_size = u32::from_be_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
        if box_size < 8 { break; }
        let box_type = &data[pos+4..pos+8];
        if box_type == b"avc1" || box_type == b"hvc1" || box_type == b"encv" {
            if pos + 78 + 2 <= data.len() {
                let w = u16::from_be_bytes([data[pos+78], data[pos+79]]);
                let h = u16::from_be_bytes([data[pos+80], data[pos+81]]);
                if w > 0 && h > 0 {
                    return Some((w as u32, h as u32));
                }
            }
        }
        if box_size == 1 && pos + 12 <= data.len() {
            let large_size = u64::from_be_bytes([
                data[pos+8], data[pos+9], data[pos+10], data[pos+11],
                data[pos+12], data[pos+13], data[pos+14], data[pos+15],
            ]);
            pos += large_size as usize;
        } else {
            pos += box_size;
        }
    }
    None
}

// ── LlmApproachType (unchanged) ──

#[derive(Debug, Clone, PartialEq)]
pub enum LlmApproachType {
    Direct,
    StepByStep,
    Comparative,
    Exploratory,
    Structured,
    CodeFirst,
    Explanatory,
    Mixed,
}

impl LlmApproachType {
    pub fn name(&self) -> &str {
        match self {
            Self::Direct => "direct",
            Self::StepByStep => "step_by_step",
            Self::Comparative => "comparative",
            Self::Exploratory => "exploratory",
            Self::Structured => "structured",
            Self::CodeFirst => "code_first",
            Self::Explanatory => "explanatory",
            Self::Mixed => "mixed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResponseStructure {
    pub has_code_blocks: bool,
    pub code_block_count: usize,
    pub has_bullet_points: bool,
    pub has_sections: bool,
    pub word_count: usize,
    pub approach_type: LlmApproachType,
}

impl ResponseStructure {
    pub fn analyze(response: &str) -> Self {
        let code_block_count = response.matches("```").count() / 2;
        let has_code_blocks = code_block_count > 0;
        let has_bullet_points = response.contains("- ") || response.contains("* ");
        let has_sections = response.contains("##") || response.contains("==");
        let word_count = response.split_whitespace().count();

        let approach_type = if has_code_blocks && has_bullet_points {
            LlmApproachType::CodeFirst
        } else if has_sections && word_count > 50 {
            LlmApproachType::Structured
        } else if has_bullet_points {
            LlmApproachType::Exploratory
        } else if word_count > 100 {
            LlmApproachType::Explanatory
        } else {
            LlmApproachType::Direct
        };

        Self {
            has_code_blocks,
            code_block_count,
            has_bullet_points,
            has_sections,
            word_count,
            approach_type,
        }
    }
}

// ── LlmReasoningPattern with metadata ──

#[derive(Debug, Clone)]
pub struct LlmReasoningPattern {
    pub id: String,
    pub e8_mode: u8,
    pub specialist_label: String,
    pub task_type: TaskType,
    pub input_type: InputType,
    pub approach_type: String,
    pub has_code_blocks: bool,
    pub has_bullets: bool,
    pub has_sections: bool,
    pub outcome_score: f64,
    pub metadata: Option<FileMetadata>,
    pub observation_time: Instant,
}

// ── ReasoningDistiller ──

#[derive(Debug, Clone)]
pub struct ReasoningDistiller {
    patterns: Vec<LlmReasoningPattern>,
    mode_success_rates: HashMap<u8, Vec<f64>>,
    approach_mode_counts: HashMap<String, HashMap<u8, u32>>,
    distillation_skill: f64,
    total_observations: u64,
    max_patterns: usize,
    #[allow(dead_code)]
    decay_rate: f64,
}

impl ReasoningDistiller {
    pub fn new() -> Self {
        Self {
            patterns: Vec::with_capacity(200),
            mode_success_rates: HashMap::new(),
            approach_mode_counts: HashMap::new(),
            distillation_skill: 0.1,
            total_observations: 0,
            max_patterns: 1000,
            decay_rate: 0.995,
        }
    }

    pub fn with_decay(decay: f64) -> Self {
        Self {
            patterns: Vec::with_capacity(200),
            mode_success_rates: HashMap::new(),
            approach_mode_counts: HashMap::new(),
            distillation_skill: 0.1,
            total_observations: 0,
            max_patterns: 1000,
            decay_rate: decay.max(0.9).min(1.0),
        }
    }

    pub fn total_observations(&self) -> u64 {
        self.total_observations
    }

    pub fn observe(
        &mut self,
        task: &str,
        response: &str,
        e8_mode: u8,
        specialist_label: &str,
        outcome_score: f64,
        has_image: bool,
        metadata: Option<FileMetadata>,
    ) {
        self.total_observations += 1;
        let structure = ResponseStructure::analyze(response);
        let task_type = Self::infer_task_type(task);
        let input_type = InputType::infer(task, has_image, metadata.as_ref());

        let pattern = LlmReasoningPattern {
            id: format!("rd_{}_{}", self.total_observations, e8_mode),
            e8_mode,
            specialist_label: specialist_label.to_string(),
            task_type,
            input_type,
            approach_type: structure.approach_type.name().to_string(),
            has_code_blocks: structure.has_code_blocks,
            has_bullets: structure.has_bullet_points,
            has_sections: structure.has_sections,
            outcome_score,
            metadata,
            observation_time: Instant::now(),
        };

        self.patterns.push(pattern);
        if self.patterns.len() > self.max_patterns {
            self.patterns.remove(0);
        }

        self.mode_success_rates.entry(e8_mode).or_default().push(outcome_score);
        if self.mode_success_rates[&e8_mode].len() > self.max_patterns {
            self.mode_success_rates.get_mut(&e8_mode).unwrap().remove(0);
        }
        self.approach_mode_counts
            .entry(structure.approach_type.name().to_string())
            .or_default()
            .entry(e8_mode)
            .and_modify(|c| *c += 1)
            .or_insert(1);

        if self.total_observations % 10 == 0 {
            self.distillation_skill = (self.distillation_skill + 0.01).min(1.0);
        }
    }

    /// N-gram embedding for task similarity matching.
    fn task_embedding(task: &str) -> Vec<u32> {
        let lower = task.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();
        let mut ngrams = std::collections::HashMap::new();
        for w in 2..=4 {
            for ng in chars.windows(w) {
                let key: String = ng.iter().collect();
                *ngrams.entry(key).or_insert(0u32) += 1;
            }
        }
        ngrams.into_values().collect()
    }

    /// Cosine similarity between two n-gram frequency vectors.
    fn embedding_similarity(a: &[u32], b: &[u32]) -> f64 {
        let min_len = a.len().min(b.len());
        if min_len == 0 {
            return 0.0;
        }
        let dot: u64 = a.iter().zip(b.iter()).take(min_len).map(|(x, y)| *x as u64 * *y as u64).sum();
        let na: f64 = (a.iter().map(|x| *x as u64 * *x as u64).sum::<u64>() as f64).sqrt();
        let nb: f64 = (b.iter().map(|x| *x as u64 * *x as u64).sum::<u64>() as f64).sqrt();
        if na < 1e-9 || nb < 1e-9 { 0.0 } else { dot as f64 / (na * nb) }
    }

    /// Decay weight for an observation based on its age.
    fn decay_weight(decay_rate: f64, elapsed_obs: u64) -> f64 {
        decay_rate.powf(elapsed_obs as f64)
    }

    /// Highest-weight average across all modes with temporal decay + confidence interval.
    /// Returns (best_mode, reason, top_approaches, confidence_interval).
    pub fn recommend_mode_weighted(&self, _task: &str, task_embed: &[u32]) -> Option<(u8, String, f64, Vec<String>)> {
        if self.patterns.is_empty() {
            return None;
        }

        let now = Instant::now();
        let mut mode_scores: HashMap<u8, Vec<(f64, f64)>> = HashMap::new(); // (weighted_score, weight)

        for p in &self.patterns {
            let elapsed = (now - p.observation_time).as_secs() as u64;
            let age_weight = Self::decay_weight(self.decay_rate, elapsed.max(1));
            let task_sim = Self::embedding_similarity(task_embed, &Self::task_embedding(&p.id.replace("rd_", "")));
            let sim_weight = 0.3 + 0.7 * task_sim; // blend: min 0.3 for dissimilar tasks
            let combined_weight = age_weight * sim_weight;
            mode_scores.entry(p.e8_mode)
                .or_default()
                .push((p.outcome_score * combined_weight, combined_weight));
        }

        let best = mode_scores.into_iter()
            .filter(|(_, scores)| {
                let total_w: f64 = scores.iter().map(|(_, w)| w).sum();
                total_w > 0.01
            })
            .map(|(mode, scores)| {
                let total_w: f64 = scores.iter().map(|(_, w)| w).sum();
                let weighted_avg: f64 = scores.iter().map(|(s, w)| s * w / total_w).sum();
                let n = scores.len() as f64;
                let variance: f64 = if n > 1.0 {
                    scores.iter().map(|(s, _)| (s / total_w * n - weighted_avg).powi(2)).sum::<f64>() / (n - 1.0)
                } else {
                    0.0
                };
                let std_err = (variance / n).sqrt();
                let ci = 1.96 * std_err; // 95% CI
                (mode, weighted_avg, ci, n as u32)
            })
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())?;

        let reason = format!(
            "基于 {} 条加权数据, 模式 {} avg={:.3}, 95%CI=±{:.3}, decay={:.3}",
            best.3, best.0, best.1, best.2, self.decay_rate
        );
        let top_approaches: Vec<String> = self.mode_profile(best.0).into_iter()
            .take(3)
            .collect();

        Some((best.0, reason, best.2, top_approaches))
    }

    pub fn recommend_mode(&self, task: &str) -> Option<(u8, String, Vec<String>)> {
        let embed = Self::task_embedding(task);
        self.recommend_mode_weighted(task, &embed)
            .map(|(mode, reason, _ci, approaches)| (mode, reason, approaches))
    }

    pub fn recommend_mode_full(&self, task: &str) -> Option<(u8, String, f64, Vec<String>)> {
        let embed = Self::task_embedding(task);
        self.recommend_mode_weighted(task, &embed)
    }

    pub fn recommend_mode_for_input(&self, task: &str, has_image: bool, metadata: Option<&FileMetadata>) -> Option<(u8, String, Vec<String>)> {
        let input_type = InputType::infer(task, has_image, metadata);
        let embed = Self::task_embedding(task);
        let patterns_by_input: Vec<&LlmReasoningPattern> = self.patterns.iter()
            .filter(|p| p.input_type.category() == input_type.category() && p.outcome_score > 0.5)
            .collect();

        if patterns_by_input.is_empty() {
            return self.recommend_mode(task);
        }

        let now = Instant::now();
        let mut mode_scores: HashMap<u8, (f64, f64, u32)> = HashMap::new();
        for p in &patterns_by_input {
            let elapsed = (now - p.observation_time).as_secs() as u64;
            let age_weight = Self::decay_weight(self.decay_rate, elapsed.max(1));
            let task_sim = Self::embedding_similarity(&embed, &Self::task_embedding(&p.id.replace("rd_", "")));
            let sim_weight = 0.3 + 0.7 * task_sim;
            let w = age_weight * sim_weight;
            let entry = mode_scores.entry(p.e8_mode).or_insert((0.0, 0.0, 0));
            entry.0 += p.outcome_score * w;
            entry.1 += w;
            entry.2 += 1;
        }

        let best = mode_scores.into_iter()
            .filter(|(_, (_, total_w, _))| *total_w > 0.01)
            .max_by(|(_, (sum_a, w_a, _)), (_, (sum_b, w_b, _))| {
                (sum_a / *w_a).partial_cmp(&(sum_b / *w_b)).unwrap()
            })?;

        let weighted_avg = best.1.0 / best.1.1;
        let reason = format!(
            "基于 {} 条 {} 数据, 模式 {} 加权avg={:.3}",
            best.1.2, input_type.category(), best.0, weighted_avg
        );
        let top_approaches: Vec<String> = self.mode_profile(best.0).into_iter()
            .take(3)
            .collect();

        Some((best.0, reason, top_approaches))
    }

    pub fn recommend_for_type(&self, input_type: &InputType) -> Option<(u8, String, Vec<String>)> {
        let patterns_by_input: Vec<&LlmReasoningPattern> = self.patterns.iter()
            .filter(|p| p.input_type.category() == input_type.category() && p.outcome_score > 0.5)
            .collect();

        if patterns_by_input.is_empty() {
            return None;
        }

        let now = Instant::now();
        let mut mode_scores: HashMap<u8, (f64, f64, u32)> = HashMap::new();
        for p in &patterns_by_input {
            let elapsed = (now - p.observation_time).as_secs() as u64;
            let age_weight = Self::decay_weight(self.decay_rate, elapsed.max(1));
            let entry = mode_scores.entry(p.e8_mode).or_insert((0.0, 0.0, 0));
            entry.0 += p.outcome_score * age_weight;
            entry.1 += age_weight;
            entry.2 += 1;
        }

        let best = mode_scores.into_iter()
            .filter(|(_, (_, total_w, _))| *total_w > 0.01)
            .max_by(|(_, (sum_a, w_a, _)), (_, (sum_b, w_b, _))| {
                (sum_a / *w_a).partial_cmp(&(sum_b / *w_b)).unwrap()
            })?;

        let weighted_avg = best.1.0 / best.1.1;
        let reason = format!(
            "类别 {}: 从 {} 条观察, 模式 {} 加权avg={:.3}",
            input_type.category(), best.1.2, best.0, weighted_avg
        );
        let top_approaches: Vec<String> = self.mode_profile(best.0).into_iter()
            .take(3)
            .collect();

        Some((best.0, reason, top_approaches))
    }

    pub fn distillation_insight(&self) -> Vec<String> {
        let mut insights = Vec::new();
        insights.push(format!(
            "蒸馏技能: {:.1}% ({} 次观察)",
            self.distillation_skill * 100.0, self.total_observations
        ));

        if self.total_observations == 0 {
            insights.push("尚无观察数据 — 继续积累中".to_string());
            return insights;
        }

        let top_mode = self.mode_success_rates.iter()
            .max_by(|(_, a), (_, b)| {
                let avg_a = a.iter().sum::<f64>() / a.len() as f64;
                let avg_b = b.iter().sum::<f64>() / b.len() as f64;
                avg_a.partial_cmp(&avg_b).unwrap()
            });

        if let Some((mode, scores)) = top_mode {
            let avg = scores.iter().sum::<f64>() / scores.len() as f64;
            insights.push(format!(
                "最佳模式: E8模式{} (avg: {:.2}, n={})",
                mode, avg, scores.len()
            ));
        }

        let total_approaches: usize = self.approach_mode_counts.values()
            .map(|m| m.values().sum::<u32>() as usize)
            .sum();
        insights.push(format!(
            "已覆盖 {:?} 种方法类型, {} 次使用",
            self.approach_mode_counts.len(),
            total_approaches
        ));

        insights
    }

    pub fn type_insights(&self) -> Vec<String> {
        if self.patterns.is_empty() {
            return vec!["尚无输入类型数据".to_string()];
        }

        let mut type_hits: HashMap<String, (u32, f64)> = HashMap::new();
        for p in &self.patterns {
            let entry = type_hits.entry(p.input_type.name().to_string()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += p.outcome_score;
        }

        let mut sorted: Vec<_> = type_hits.into_iter().collect();
        sorted.sort_by(|a, b| b.1.0.cmp(&a.1.0));

        let mut insights = Vec::new();
        insights.push(format!("已覆盖 {} 种输入类型", sorted.len()));
        for (name, (count, total_score)) in sorted.iter().take(10) {
            let avg = total_score / *count as f64;
            insights.push(format!("  {}: {}次, avg={:.2}", name, count, avg));
        }
        insights
    }

    pub fn mode_profile(&self, mode: u8) -> Vec<String> {
        let mode_patterns: Vec<&LlmReasoningPattern> = self.patterns.iter()
            .filter(|p| p.e8_mode == mode)
            .collect();

        if mode_patterns.is_empty() {
            return vec![format!("E8模式{} 无观察数据", mode)];
        }

        let mut profile = Vec::new();
        let total = mode_patterns.len() as f64;
        let avg_score = mode_patterns.iter().map(|p| p.outcome_score).sum::<f64>() / total;
        profile.push(format!(
            "E8模式{}: {} 次观察, avg={:.2}",
            mode, mode_patterns.len(), avg_score
        ));

        let code_ratio = mode_patterns.iter().filter(|p| p.has_code_blocks).count() as f64 / total;
        let bullet_ratio = mode_patterns.iter().filter(|p| p.has_bullets).count() as f64 / total;
        let section_ratio = mode_patterns.iter().filter(|p| p.has_sections).count() as f64 / total;
        profile.push(format!(
            "  结构: 代码 {:.0}% 要点 {:.0}% 章节 {:.0}%",
            code_ratio * 100.0, bullet_ratio * 100.0, section_ratio * 100.0
        ));

        let mut approach_counts: HashMap<&str, u32> = HashMap::new();
        for p in &mode_patterns {
            *approach_counts.entry(p.approach_type.as_str()).or_insert(0) += 1;
        }
        let mut sorted: Vec<_> = approach_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        for (approach, count) in sorted.iter().take(3) {
            profile.push(format!("  方法 {}: {} 次", approach, count));
        }

        let mut input_counts: HashMap<&str, u32> = HashMap::new();
        for p in &mode_patterns {
            *input_counts.entry(p.input_type.name()).or_insert(0) += 1;
        }
        if input_counts.len() > 1 {
            let mut sorted_input: Vec<_> = input_counts.into_iter().collect();
            sorted_input.sort_by(|a, b| b.1.cmp(&a.1));
            let input_str: Vec<String> = sorted_input.iter().map(|(t, c)| format!("{}:{}", t, c)).collect();
            profile.push(format!("  输入类型: {}", input_str.join(" ")));
        }

        profile
    }

    pub fn type_profile(&self, input_type: &InputType) -> Vec<String> {
        let type_patterns: Vec<&LlmReasoningPattern> = self.patterns.iter()
            .filter(|p| p.input_type == *input_type)
            .collect();

        if type_patterns.is_empty() {
            return vec![format!("类型 {} 无观察数据", input_type.name())];
        }

        let mut profile = Vec::new();
        let total = type_patterns.len() as f64;
        let avg_score = type_patterns.iter().map(|p| p.outcome_score).sum::<f64>() / total;
        profile.push(format!(
            "类型 {}: {} 次观察, avg={:.2}",
            input_type.name(), type_patterns.len(), avg_score
        ));
        profile.push(format!("  类别: {}", input_type.category()));

        let mut mode_hits: HashMap<u8, (u32, f64)> = HashMap::new();
        for p in &type_patterns {
            let entry = mode_hits.entry(p.e8_mode).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += p.outcome_score;
        }
        let mut sorted_modes: Vec<_> = mode_hits.into_iter().collect();
        sorted_modes.sort_by(|a, b| b.1.0.cmp(&a.1.0));
        for (mode, (count, total_score)) in sorted_modes.iter().take(5) {
            profile.push(format!("  E8模式{}: {}次, avg={:.2}", mode, count, total_score / *count as f64));
        }

        profile
    }

    pub fn all_approach_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.approach_mode_counts.keys().cloned().collect();
        types.sort();
        types
    }

    pub fn all_input_types(&self) -> Vec<String> {
        let mut types: Vec<String> = self.patterns.iter()
            .map(|p| p.input_type.name().to_string())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        types.sort();
        types
    }

    fn infer_task_type(task: &str) -> TaskType {
        let lower = task.to_lowercase();
        if lower.contains("bug") || lower.contains("error") || lower.contains("fix") || lower.contains("debug") {
            TaskType::CodeReview
        } else if lower.contains("test") || lower.contains("unit") || lower.contains("integration") {
            TaskType::CodeAnalysis
        } else if lower.contains("design") || lower.contains("ui") || lower.contains("architecture") {
            TaskType::Design
        } else if lower.contains("research") || lower.contains("explain") || lower.contains("what is") {
            TaskType::Research
        } else if lower.contains("refactor") || lower.contains("optimize") || lower.contains("improve") {
            TaskType::Planning
        } else if lower.contains("generate") || lower.contains("create") || lower.contains("write") {
            TaskType::CodeGeneration
        } else if lower.contains("secure") || lower.contains("vuln") || lower.contains("attack") {
            TaskType::Security
        } else if lower.contains("learn") || lower.contains("study") || lower.contains("understand") {
            TaskType::Learning
        } else if lower.contains("diagram") || lower.contains("chart") || lower.contains("graph")
            || lower.contains("image") || lower.contains("picture")
            || lower.contains("video") || lower.contains("audio")
            || lower.contains("music") || lower.contains("sound") {
            TaskType::Research
        } else {
            TaskType::General
        }
    }
}

impl Default for ReasoningDistiller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_format_name() {
        assert_eq!(MediaFormat::Jpeg.name(), "jpeg");
        assert_eq!(MediaFormat::Mp4.name(), "mp4");
        assert_eq!(MediaFormat::Flac.name(), "flac");
        assert_eq!(MediaFormat::Pdf.name(), "pdf");
        assert_eq!(MediaFormat::Zip.name(), "zip");
        assert_eq!(MediaFormat::Unknown.name(), "unknown");
    }

    #[test]
    fn test_media_format_category() {
        assert_eq!(MediaFormat::Png.category(), "image");
        assert_eq!(MediaFormat::Mkv.category(), "video");
        assert_eq!(MediaFormat::Wav.category(), "audio");
        assert_eq!(MediaFormat::Docx.category(), "document");
        assert_eq!(MediaFormat::Rar.category(), "archive");
        assert_eq!(MediaFormat::Rs.category(), "code");
        assert_eq!(MediaFormat::Html.category(), "web");
        assert_eq!(MediaFormat::Json.category(), "data");
        assert_eq!(MediaFormat::Toml.category(), "config");
        assert_eq!(MediaFormat::Md.category(), "markup");
        assert_eq!(MediaFormat::Ttf.category(), "font");
        assert_eq!(MediaFormat::Onnx.category(), "model");
    }

    #[test]
    fn test_input_type_name() {
        assert_eq!(InputType::Image(MediaFormat::Png).name(), "png");
        assert_eq!(InputType::Video(MediaFormat::Mp4).name(), "mp4");
        assert_eq!(InputType::Audio(MediaFormat::Wav).name(), "wav");
        assert_eq!(InputType::Text.name(), "text");
        assert_eq!(InputType::Binary.name(), "binary");
        assert_eq!(InputType::Unknown.name(), "unknown");
    }

    #[test]
    fn test_input_type_format() {
        assert_eq!(InputType::Image(MediaFormat::Jpeg).format(), Some(MediaFormat::Jpeg));
        assert_eq!(InputType::Text.format(), None);
    }

    #[test]
    fn test_input_type_is_media() {
        assert!(InputType::Image(MediaFormat::Png).is_media());
        assert!(InputType::Video(MediaFormat::Mp4).is_media());
        assert!(InputType::Audio(MediaFormat::Mp3).is_media());
        assert!(!InputType::Text.is_media());
        assert!(!InputType::Code(MediaFormat::Rs).is_media());
    }

    #[test]
    fn test_from_bytes_jpeg() {
        let bytes = [0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10];
        let input = InputType::from_bytes(&bytes);
        assert_eq!(input, InputType::Image(MediaFormat::Jpeg));
    }

    #[test]
    fn test_from_bytes_png() {
        let bytes = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
        let input = InputType::from_bytes(&bytes);
        assert_eq!(input, InputType::Image(MediaFormat::Png));
    }

    #[test]
    fn test_from_bytes_gif() {
        assert_eq!(InputType::from_bytes(b"GIF87a"), InputType::Image(MediaFormat::Gif));
        assert_eq!(InputType::from_bytes(b"GIF89a"), InputType::Image(MediaFormat::Gif));
    }

    #[test]
    fn test_from_bytes_pdf() {
        assert_eq!(InputType::from_bytes(b"%PDF-1.4"), InputType::Document(MediaFormat::Pdf));
    }

    #[test]
    fn test_from_bytes_zip() {
        assert_eq!(InputType::from_bytes(b"PK\x03\x04"), InputType::Archive(MediaFormat::Zip));
    }

    #[test]
    fn test_from_bytes_mp3() {
        assert_eq!(InputType::from_bytes(b"ID3"), InputType::Audio(MediaFormat::Mp3));
    }

    #[test]
    fn test_from_bytes_wasm() {
        assert_eq!(InputType::from_bytes(b"\x00asm\x01\x00\x00\x00"), InputType::Code(MediaFormat::Wasm));
    }

    #[test]
    fn test_from_bytes_empty() {
        assert_eq!(InputType::from_bytes(b""), InputType::Text);
    }

    #[test]
    fn test_from_bytes_tiff() {
        assert_eq!(InputType::from_bytes(b"MM\x00*"), InputType::Image(MediaFormat::Tiff));
        assert_eq!(InputType::from_bytes(b"II*\x00"), InputType::Image(MediaFormat::Tiff));
    }

    #[test]
    fn test_from_bytes_gzip() {
        assert_eq!(InputType::from_bytes(b"\x1f\x8b\x08"), InputType::Archive(MediaFormat::Gz));
    }

    #[test]
    fn test_infer_image_task() {
        let result = InputType::infer("analyze this architecture diagram", false, None);
        assert_eq!(result.category(), "image");
    }

    #[test]
    fn test_infer_video_task() {
        let result = InputType::infer("process this video file", false, None);
        assert_eq!(result.category(), "video");
    }

    #[test]
    fn test_infer_audio_task() {
        let result = InputType::infer("transcribe audio recording", false, None);
        assert_eq!(result.category(), "audio");
    }

    #[test]
    fn test_infer_file_extension_rs() {
        let result = InputType::infer("file: src/main.rs", false, None);
        assert_eq!(result, InputType::Code(MediaFormat::Rs));
    }

    #[test]
    fn test_infer_file_extension_py() {
        let result = InputType::infer("file: script.py", false, None);
        assert_eq!(result, InputType::Code(MediaFormat::Py));
    }

    #[test]
    fn test_infer_file_extension_csv() {
        let result = InputType::infer("file.csv contains data", false, None);
        assert_eq!(result, InputType::Data(MediaFormat::Csv));
    }

    #[test]
    fn test_infer_file_extension_json() {
        let result = InputType::infer("data.json", false, None);
        assert_eq!(result, InputType::Data(MediaFormat::Json));
    }

    #[test]
    fn test_infer_file_extension_md() {
        let result = InputType::infer("file: README.md", false, None);
        assert_eq!(result, InputType::Markup(MediaFormat::Md));
    }

    #[test]
    fn test_infer_file_extension_pdf() {
        let result = InputType::infer("open report.pdf", false, None);
        assert_eq!(result, InputType::Document(MediaFormat::Pdf));
    }

    #[test]
    fn test_infer_file_extension_png() {
        let result = InputType::infer("screenshot.png", false, None);
        assert_eq!(result, InputType::Image(MediaFormat::Png));
    }

    #[test]
    fn test_infer_file_extension_toml() {
        let result = InputType::infer("file: Cargo.toml", false, None);
        assert_eq!(result, InputType::Config(MediaFormat::Toml));
    }

    #[test]
    fn test_infer_with_metadata() {
        let meta = FileMetadata { format: MediaFormat::Pdf, ..Default::default() };
        let result = InputType::infer("some task", false, Some(&meta));
        assert_eq!(result, InputType::Document(MediaFormat::Pdf));
    }

    #[test]
    fn test_infer_code_in_task() {
        let result = InputType::infer("```let x = 42```", false, None);
        assert_eq!(result.category(), "code");
    }

    #[test]
    fn test_infer_text_fallback() {
        let result = InputType::infer("hello world", false, None);
        assert_eq!(result, InputType::Text);
    }

    fn make_png_header(w: u32, h: u32) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"\x89PNG\r\n\x1a\n"); // signature
        // IHDR chunk
        data.extend_from_slice(&(13u32).to_be_bytes()); // chunk length
        data.extend_from_slice(b"IHDR"); // chunk type
        data.extend_from_slice(&w.to_be_bytes());
        data.extend_from_slice(&h.to_be_bytes());
        data.extend_from_slice(&[8, 2, 0, 0, 0]); // bit depth, color type, compression, filter, interlace
        // CRC placeholder
        data.extend_from_slice(&[0u8; 4]);
        data
    }

    #[test]
    fn test_extract_metadata_png() {
        let data = make_png_header(800, 600);
        let input = InputType::Image(MediaFormat::Png);
        let meta = extract_metadata(&data, &input);
        assert_eq!(meta.dimensions, Some((800, 600)));
        assert_eq!(meta.size_bytes, data.len() as u64);
    }

    #[test]
    fn test_extract_metadata_jpeg() {
        // Minimal JPEG with SOF0 marker
        let data = vec![
            0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00,
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06,
            0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0a, 0x0c, 0x14, 0x0d,
            0x0c, 0x0b, 0x0b, 0x0c, 0x19, 0x12, 0x13, 0x0f, 0x14, 0x1d, 0x1a, 0x1f, 0x1e, 0x1d,
            0x1a, 0x1c, 0x1c, 0x20, 0x24, 0x2e, 0x27, 0x20, 0x22, 0x2c, 0x23, 0x1c, 0x1c, 0x28,
            0x37, 0x29, 0x2c, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1f, 0x27, 0x39, 0x3d, 0x38, 0x32,
            0x3c, 0x2e, 0x33, 0x34, 0x32, 0xff, 0xc0, 0x00, 0x0b, 0x08, 0x01, 0x2c, 0x01, 0xf4,
            0x01, 0x22, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01,
        ];
        let input = InputType::Image(MediaFormat::Jpeg);
        let meta = extract_metadata(&data, &input);
        assert_eq!(meta.dimensions, Some((500, 300)));
    }

    #[test]
    fn test_extract_metadata_gif() {
        let data: &[u8] = b"GIF89a\xf4\x01\xe8\x03\xf7\x00\x00";
        let input = InputType::Image(MediaFormat::Gif);
        let meta = extract_metadata(data, &input);
        assert_eq!(meta.dimensions, Some((500, 1000)));
    }

    #[test]
    fn test_extract_metadata_wav() {
        let mut data = Vec::new();
        data.extend_from_slice(b"RIFF");
        data.extend_from_slice(&[0u8; 4]);
        data.extend_from_slice(b"WAVE");
        data.extend_from_slice(b"fmt ");
        data.extend_from_slice(&16u32.to_le_bytes());
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&2u16.to_le_bytes());
        data.extend_from_slice(&44100u32.to_le_bytes());
        data.extend_from_slice(&(44100u32 * 2 * 2).to_le_bytes());
        data.extend_from_slice(&4u16.to_le_bytes());
        data.extend_from_slice(&16u16.to_le_bytes());
        data.extend_from_slice(b"data");
        data.extend_from_slice(&(44100 * 2 * 2 * 5u32).to_le_bytes());
        data.push(0); // at least 1 byte of audio data so len > 44
        let input = InputType::Audio(MediaFormat::Wav);
        let meta = extract_metadata(&data, &input);
        assert_eq!(meta.sample_rate, Some(44100));
        assert_eq!(meta.channels, Some(2));
        assert!(meta.duration_ms.unwrap_or(0) > 4000);
        assert!(meta.duration_ms.unwrap_or(0) < 6000);
    }

    #[test]
    fn test_file_metadata_summary_image() {
        let meta = FileMetadata {
            format: MediaFormat::Png,
            size_bytes: 2_500_000,
            dimensions: Some((1920, 1080)),
            has_alpha: true,
            ..Default::default()
        };
        let summary = meta.summary();
        assert!(summary.contains("png"));
        assert!(summary.contains("2.4MB"));
        assert!(summary.contains("1920x1080"));
        assert!(summary.contains("alpha"));
    }

    #[test]
    fn test_file_metadata_summary_audio() {
        let meta = FileMetadata {
            format: MediaFormat::Mp3,
            size_bytes: 8_000_000,
            duration_ms: Some(234_000),
            sample_rate: Some(44100),
            channels: Some(2),
            ..Default::default()
        };
        let summary = meta.summary();
        assert!(summary.contains("mp3"));
        assert!(summary.contains("7.6MB") || summary.contains("8.0MB") || summary.contains("8MB"));
        assert!(summary.contains("3m54s"));
        assert!(summary.contains("44100Hz"));
        assert!(summary.contains("2ch"));
    }

    #[test]
    fn test_file_metadata_summary_empty() {
        let meta = FileMetadata::empty();
        assert_eq!(meta.summary(), "unknown");
    }

    #[test]
    fn test_observe_with_metadata() {
        let mut distiller = ReasoningDistiller::new();
        let meta = FileMetadata {
            format: MediaFormat::Jpeg,
            size_bytes: 1024,
            dimensions: Some((800, 600)),
            ..Default::default()
        };
        distiller.observe("analyze image", "It shows a chart.", 5, "Vision", 0.85, true, Some(meta));
        assert_eq!(distiller.total_observations(), 1);
        let profile = distiller.type_profile(&InputType::Image(MediaFormat::Jpeg));
        assert!(profile.len() >= 2);
        assert!(profile[0].contains("jpeg"));
    }

    #[test]
    fn test_recommend_for_type() {
        let mut distiller = ReasoningDistiller::new();
        distiller.observe("fix src/main.rs bug", "```code``` fix", 5, "Code", 0.9, false, None);
        distiller.observe("fix src/lib.rs error", "```code``` fix2", 5, "Code", 0.8, false, None);
        let rec = distiller.recommend_for_type(&InputType::Code(MediaFormat::Rs));
        assert!(rec.is_some());
        assert_eq!(rec.unwrap().0, 5);
    }

    #[test]
    fn test_recommend_for_type_no_data() {
        let distiller = ReasoningDistiller::new();
        assert!(distiller.recommend_for_type(&InputType::Image(MediaFormat::Png)).is_none());
    }

    #[test]
    fn test_type_insights_empty() {
        let distiller = ReasoningDistiller::new();
        assert_eq!(distiller.type_insights()[0], "尚无输入类型数据");
    }

    #[test]
    fn test_type_insights_nonempty() {
        let mut distiller = ReasoningDistiller::new();
        distiller.observe("fix code", "```x```", 5, "Code", 0.9, false, None);
        distiller.observe("read pdf", "report", 3, "Doc", 0.7, false, None);
        let insights = distiller.type_insights();
        assert!(insights[0].contains("覆盖"));
    }

    #[test]
    fn test_all_input_types() {
        let mut distiller = ReasoningDistiller::new();
        distiller.observe("fix src/main.rs", "```rs```", 5, "Code", 0.9, false, None);
        distiller.observe("read report.pdf", "report", 3, "Doc", 0.7, false, None);
        let types = distiller.all_input_types();
        assert!(types.contains(&"rs".to_string()));
    }

    #[test]
    fn test_type_profile_no_data() {
        let distiller = ReasoningDistiller::new();
        let profile = distiller.type_profile(&InputType::Image(MediaFormat::Png));
        assert!(profile[0].contains("无观察数据"));
    }

    #[test]
    fn test_extract_metadata_pdf() {
        let data = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n3 0 obj\n<< /Type /Page /Parent 2 0 R >>\nendobj\n";
        let input = InputType::Document(MediaFormat::Pdf);
        let meta = extract_metadata(data, &input);
        assert_eq!(meta.pages, Some(1));
    }

    #[test]
    fn test_analyze_code_response() {
        let response = "Here's the fix:\n\n```rust\nfn foo() -> i32 { 42 }\n```\n\nThis resolves the issue.";
        let structure = ResponseStructure::analyze(response);
        assert!(structure.has_code_blocks);
        assert_eq!(structure.code_block_count, 1);
    }

    #[test]
    fn test_analyze_bullet_response() {
        let response = "Steps:\n- First do this\n- Then do that\n- Finally check\n\nDone.";
        let structure = ResponseStructure::analyze(response);
        assert!(structure.has_bullet_points);
    }

    #[test]
    fn test_analyze_structured_response() {
        let response = "# Title\n\n## Section 1\nContent here.\n\n## Section 2\nMore content.\n\n### Subsection\nDetails.";
        let structure = ResponseStructure::analyze(response);
        assert!(structure.has_sections);
    }

    #[test]
    fn test_distiller_observe_and_recommend() {
        let mut distiller = ReasoningDistiller::new();

        distiller.observe("fix bug in parser", "The issue is in the lexer.\n\n```rust\nlet x = 42;\n```\n\nFix applied.", 5, "Code", 0.9, false, None);
        distiller.observe("fix error handler", "Step 1: Check input\nStep 2: Validate\nStep 3: Return result", 5, "Code", 0.8, false, None);
        distiller.observe("debug memory leak", "Root cause: reference cycle.\n\n- Track allocations\n- Find leaks\n- Fix them", 3, "Analyze", 0.3, false, None);

        let recommendation = distiller.recommend_mode("fix crash in module");
        assert!(recommendation.is_some());
        let (mode, _, _) = recommendation.unwrap();
        assert_eq!(mode, 5);
    }

    #[test]
    fn test_distiller_insights() {
        let mut distiller = ReasoningDistiller::new();
        for i in 0..10 {
            distiller.observe(&format!("task {}", i), "Some response text here. Multiple sentences. With analysis. And findings.", 3, "Code", 0.7 + (i as f64 * 0.02), false, None);
        }
        let insights = distiller.distillation_insight();
        assert!(insights.len() >= 3);
        assert!(insights[0].contains("蒸馏技能"));
    }

    #[test]
    fn test_mode_profile() {
        let mut distiller = ReasoningDistiller::new();
        for _ in 0..5 {
            distiller.observe("test task", "```code```\n\n- bullet\n- bullet", 7, "Code", 0.85, false, None);
        }
        let profile = distiller.mode_profile(7);
        assert!(profile.len() >= 3);
        assert!(profile[0].contains("E8模式7"));
    }

    #[test]
    fn test_all_approach_types() {
        let mut distiller = ReasoningDistiller::new();
        distiller.observe("task", "short reply", 1, "Code", 0.5, false, None);
        distiller.observe("task", "# Big\n\n## Structured\n\nResponse here", 2, "Analyze", 0.7, false, None);
        let approaches = distiller.all_approach_types();
        assert!(!approaches.is_empty());
    }

    #[test]
    fn test_empty_distiller() {
        let distiller = ReasoningDistiller::new();
        assert!(distiller.recommend_mode("test").is_none());
        assert_eq!(distiller.distillation_insight().len(), 2);
    }

    #[test]
    fn test_recommend_mode_for_input_fallback() {
        let distiller = ReasoningDistiller::new();
        assert!(distiller.recommend_mode_for_input("fix bug", false, None).is_none());
    }

    #[test]
    fn test_infer_tiff_extensions() {
        assert_eq!(InputType::infer("image.tiff", false, None), InputType::Image(MediaFormat::Tiff));
        assert_eq!(InputType::infer("image.tif", false, None), InputType::Image(MediaFormat::Tiff));
    }

    #[test]
    fn test_infer_webp() {
        assert_eq!(InputType::infer("image.webp", false, None), InputType::Image(MediaFormat::Webp));
    }

    #[test]
    fn test_infer_heic() {
        assert_eq!(InputType::infer("photo.heic", false, None), InputType::Image(MediaFormat::Heic));
    }

    #[test]
    fn test_infer_mov() {
        assert_eq!(InputType::infer("video.mov", false, None), InputType::Video(MediaFormat::Mov));
    }

    #[test]
    fn test_infer_mkv() {
        assert_eq!(InputType::infer("video.mkv", false, None), InputType::Video(MediaFormat::Mkv));
    }

    #[test]
    fn test_infer_webm() {
        assert_eq!(InputType::infer("video.webm", false, None), InputType::Video(MediaFormat::Webm));
    }

    #[test]
    fn test_infer_ogg_video() {
        assert_eq!(InputType::infer("video.ogv", false, None), InputType::Video(MediaFormat::Ogv));
    }

    #[test]
    fn test_infer_flac() {
        assert_eq!(InputType::infer("song.flac", false, None), InputType::Audio(MediaFormat::Flac));
    }

    #[test]
    fn test_infer_opus() {
        assert_eq!(InputType::infer("audio.opus", false, None), InputType::Audio(MediaFormat::Opus));
    }

    #[test]
    fn test_infer_aiff() {
        assert_eq!(InputType::infer("sound.aiff", false, None), InputType::Audio(MediaFormat::Aiff));
    }

    #[test]
    fn test_infer_pptx() {
        assert_eq!(InputType::infer("slides.pptx", false, None), InputType::Document(MediaFormat::Pptx));
    }

    #[test]
    fn test_infer_xlsx() {
        assert_eq!(InputType::infer("data.xlsx", false, None), InputType::Document(MediaFormat::Xlsx));
    }

    #[test]
    fn test_infer_epub() {
        assert_eq!(InputType::infer("book.epub", false, None), InputType::Document(MediaFormat::Epub));
    }

    #[test]
    fn test_infer_rar() {
        assert_eq!(InputType::infer("archive.rar", false, None), InputType::Archive(MediaFormat::Rar));
    }

    #[test]
    fn test_infer_sevenz() {
        assert_eq!(InputType::infer("archive.7z", false, None), InputType::Archive(MediaFormat::SevenZ));
    }

    #[test]
    fn test_infer_svg() {
        assert_eq!(InputType::infer("icon.svg", false, None), InputType::Image(MediaFormat::Svg));
    }

    #[test]
    fn test_infer_bmp() {
        assert_eq!(InputType::infer("image.bmp", false, None), InputType::Image(MediaFormat::Bmp));
    }

    #[test]
    fn test_extract_metadata_bmp() {
        let data = [
            0x42, 0x4d, // BM
            0x36, 0x00, 0x00, 0x00, // file size
            0x00, 0x00, 0x00, 0x00, // reserved
            0x36, 0x00, 0x00, 0x00, // offset
            0x28, 0x00, 0x00, 0x00, // header size
            0x20, 0x03, 0x00, 0x00, // width = 800
            0x58, 0x02, 0x00, 0x00, // height = 600
            0x01, 0x00, // planes
            0x20, 0x00, // bit count
        ];
        let input = InputType::Image(MediaFormat::Bmp);
        let meta = extract_metadata(&data, &input);
        assert_eq!(meta.dimensions, Some((800, 600)));
    }

    #[test]
    fn test_infer_type_from_task_sql() {
        assert_eq!(InputType::infer("query.sql", false, None), InputType::Data(MediaFormat::Sql));
    }

    #[test]
    fn test_infer_type_from_task_html() {
        assert_eq!(InputType::infer("index.html", false, None), InputType::Web(MediaFormat::Html));
    }

    #[test]
    fn test_infer_type_from_task_yaml() {
        assert_eq!(InputType::infer("config.yaml", false, None), InputType::Config(MediaFormat::Yaml));
    }

    #[test]
    fn test_infer_type_from_task_wasm() {
        assert_eq!(InputType::infer("module.wasm", false, None), InputType::Code(MediaFormat::Wasm));
    }

    #[test]
    fn test_infer_type_from_task_onnx() {
        assert_eq!(InputType::infer("model.onnx", false, None), InputType::Model(MediaFormat::Onnx));
    }

    #[test]
    fn test_infer_type_from_task_ttf() {
        assert_eq!(InputType::infer("font.ttf", false, None), InputType::Font(MediaFormat::Ttf));
    }

    #[test]
    fn test_infer_type_from_task_otf() {
        assert_eq!(InputType::infer("font.otf", false, None), InputType::Font(MediaFormat::Otf));
    }
}

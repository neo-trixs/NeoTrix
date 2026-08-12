//! NT-Pack v1 — NeoTrix 高密度数据格式编解码器 (NT-MEMORY 域)。
//!
//! 以最小数据量表达最高数据信息密度。面向"大量小记录" (坐标 + 短字符串标签 + ID)。
//! 设计来源: Google Polyline (E6定点+delta+zigzag+varint) + OSM PBF (StringTable 字典)
//!          + Parquet 列式思想 + 可选 zstd 熵压缩。
//!
//! 文件布局: [魔数 "NTPACK01" 8B][版本 1B][flags 2B][记录数 4B]
//!           [列描述段: 每列 type+precision+dict_size+字典]
//!           [坐标段: lat/lng 交错 E6定点→delta→zigzag→varint]
//!           [字符串段: StringTable 条目]
//!           [索引段: 每记录各列字典索引 varint]
//!           [可选 zstd 整体压缩数据段]
//!
//! 规范详见: docs/nt-pack-format.md

use std::collections::HashMap;

/// NT-Pack 魔数
pub const MAGIC: &[u8; 8] = b"NTPACK01";
/// 当前版本
pub const VERSION: u8 = 1;

/// flags bit
pub const FLAG_ZSTD: u16 = 0b0000_0000_0000_0001;
pub const FLAG_DELTA: u16 = 0b0000_0000_0000_0010;
pub const FLAG_TRUNCATE: u16 = 0b0000_0000_0000_0100;
/// 文件尾带 CRC32 校验 (C5 自愈基础, 损坏检测)
pub const FLAG_CHECKSUM: u16 = 0b0000_0000_0000_1000;
/// 分块模式 (v2, A5): 块表定位 → 随机访问单块解码; 见 nt_memory_pack_chunked
pub const FLAG_CHUNKED: u16 = 0b0000_0000_0001_0000;

/// 列类型
pub const COL_COORD_LAT: u8 = 0;
pub const COL_COORD_LNG: u8 = 1;
pub const COL_STRING_DICT: u8 = 2;
pub const COL_U32: u8 = 3;
pub const COL_F64: u8 = 4;
/// node_id 专用列: 前缀入字典 + ident 原始字节流 (高基数 ID 不字典化, 省字典空间)
pub const COL_NODE_ID: u8 = 5;

/// 一条地理记录 (与 geo_index 表结构对应)
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GeoPoint {
    pub node_id: String,
    pub lat: f64,
    pub lng: f64,
    pub country: String,
    pub region: String,
    pub city: String,
    pub tags: String,
    pub source: String,
}

/// NT-Pack 编码器
#[derive(Debug)]
pub struct PackEncoder {
    /// 坐标精度: 5 = E5 (1.1m), 6 = E6 (11cm)
    pub precision: u8,
    /// 是否启用 zstd 压缩
    pub use_zstd: bool,
}

impl Default for PackEncoder {
    fn default() -> Self {
        Self {
            precision: 5,
            use_zstd: true,
        }
    }
}

impl PackEncoder {
    pub fn new(precision: u8, use_zstd: bool) -> Self {
        Self {
            precision: precision.max(1).min(7),
            use_zstd,
        }
    }

    /// 编码一组地理点为 NT-Pack 二进制
    pub fn encode(&self, points: &[GeoPoint]) -> Vec<u8> {
        let n = points.len();
        let scale = 10i64.pow(self.precision as u32);

        // ---- StringTable 字典: 所有字符串字段全局去重 ----
        let mut dict: Vec<String> = Vec::new();
        let mut dict_map: HashMap<String, u32> = HashMap::new();
        let intern = |s: &str, dict: &mut Vec<String>, dict_map: &mut HashMap<String, u32>| -> u32 {
            if s.is_empty() {
                return 0;
            }
            if let Some(&idx) = dict_map.get(s) {
                return idx;
            }
            let idx = dict.len() as u32 + 1;
            dict.push(s.to_string());
            dict_map.insert(s.to_string(), idx);
            idx
        };

        // 坐标 (定点化) + 字符串索引
        let mut lats: Vec<i64> = Vec::with_capacity(n);
        let mut lngs: Vec<i64> = Vec::with_capacity(n);
        let mut countries: Vec<u32> = Vec::with_capacity(n);
        let mut regions: Vec<u32> = Vec::with_capacity(n);
        let mut cities: Vec<u32> = Vec::with_capacity(n);
        let mut tags: Vec<u32> = Vec::with_capacity(n);
        let mut sources: Vec<u32> = Vec::with_capacity(n);

        // node_id: 最长公共前缀 (LCP) 入字典, ident 部分走原始字节流
        let mut lcp = points[0].node_id.clone();
        for p in &points[1..] {
            while !p.node_id.starts_with(&lcp) {
                lcp.pop();
                if lcp.is_empty() {
                    break;
                }
            }
            if lcp.is_empty() {
                break;
            }
        }
        let prefix_idx = intern(&lcp, &mut dict, &mut dict_map);
        let mut idents: Vec<&str> = Vec::with_capacity(n);
        for p in points {
            idents.push(&p.node_id[lcp.len()..]);
        }

        for p in points {
            lats.push((p.lat * scale as f64).round() as i64);
            lngs.push((p.lng * scale as f64).round() as i64);
            countries.push(intern(&p.country, &mut dict, &mut dict_map));
            regions.push(intern(&p.region, &mut dict, &mut dict_map));
            cities.push(intern(&p.city, &mut dict, &mut dict_map));
            tags.push(intern(&p.tags, &mut dict, &mut dict_map));
            sources.push(intern(&p.source, &mut dict, &mut dict_map));
        }

        // ---- 组装输出 ----
        let mut out = Vec::with_capacity(64 + n * 8);
        out.extend_from_slice(MAGIC);
        out.push(VERSION);

        let mut flags = FLAG_DELTA | FLAG_CHECKSUM;
        if self.use_zstd {
            flags |= FLAG_ZSTD;
        }
        out.extend_from_slice(&flags.to_le_bytes());
        out.extend_from_slice(&(n as u32).to_le_bytes());

        // 列描述段: 8 列 (node_id, lat, lng, country, region, city, tags, source)
        let cols: [(u8, u8); 8] = [
            (COL_NODE_ID, 0),
            (COL_COORD_LAT, self.precision),
            (COL_COORD_LNG, self.precision),
            (COL_STRING_DICT, 0),
            (COL_STRING_DICT, 0),
            (COL_STRING_DICT, 0),
            (COL_STRING_DICT, 0),
            (COL_STRING_DICT, 0),
        ];
        out.push(cols.len() as u8);
        for (t, prec) in &cols {
            out.push(*t);
            out.push(*prec);
        }

        // 字符串段: StringTable
        out.extend_from_slice(&(dict.len() as u32).to_le_bytes());
        for s in &dict {
            out.extend_from_slice(&(s.len() as u32).to_le_bytes());
            out.extend_from_slice(s.as_bytes());
        }

        // ---- 坐标段: delta + zigzag + varint ----
        let mut prev_lat = 0i64;
        let mut prev_lng = 0i64;
        for i in 0..n {
            let dlat = lats[i] - prev_lat;
            let dlng = lngs[i] - prev_lng;
            prev_lat = lats[i];
            prev_lng = lngs[i];
            push_zigzag_varint(&mut out, dlat);
            push_zigzag_varint(&mut out, dlng);
        }

        // ---- ident 段: [varint prefix_idx][n × (varint len + 原始字节)] ----
        push_varint(&mut out, prefix_idx as u64);
        for ident in &idents {
            push_varint(&mut out, ident.len() as u64);
            out.extend_from_slice(ident.as_bytes());
        }

        // ---- 索引段: 5 个字符串列 (node_id 已走 ident 段) ----
        for i in 0..n {
            push_varint(&mut out, countries[i] as u64);
        }
        for i in 0..n {
            push_varint(&mut out, regions[i] as u64);
        }
        for i in 0..n {
            push_varint(&mut out, cities[i] as u64);
        }
        for i in 0..n {
            push_varint(&mut out, tags[i] as u64);
        }
        for i in 0..n {
            push_varint(&mut out, sources[i] as u64);
        }

        // ---- 可选 zstd 压缩 (只压数据区: 坐标+ident+索引, 保留头部+字典可解析) ----
        if self.use_zstd {
            compress_data_section(&mut out);
        }

        // ---- 文件尾 CRC32 校验 (FLAG_CHECKSUM) ----
        let crc = crc32fast::hash(&out);
        out.extend_from_slice(&crc.to_le_bytes());

        out
    }
}

/// NT-Pack 解码器
#[derive(Debug)]
pub struct PackDecoder {
    pub precision: u8,
    pub use_zstd: bool,
}

impl PackDecoder {
    /// 解码 NT-Pack 二进制为地理点列表 (无损: 定点量化容差内)。
    /// 兼容 v2 分块文件 (A5): 检测到 FLAG_CHUNKED 自动路由到块解码。
    pub fn decode(bytes: &[u8]) -> Result<(Self, Vec<GeoPoint>), String> {
        if bytes.len() < 16 || &bytes[0..8] != MAGIC {
            return Err("invalid NT-Pack magic".into());
        }
        // v2 分块文件: 委托给 chunked 解码器 (全部块按序拼接)
        {
            let flags_probe = u16::from_le_bytes([bytes[9], bytes[10]]);
            if flags_probe & FLAG_CHUNKED != 0 {
                let all = super::nt_memory_pack_chunked::decode_chunked(bytes)?;
                return Ok((
                    Self {
                        precision: 5,
                        use_zstd: true,
                    },
                    all,
                ));
            }
        }
        let version = bytes[8];
        if version != VERSION {
            return Err(format!("unsupported NT-Pack version: {}", version));
        }
        let flags = u16::from_le_bytes([bytes[9], bytes[10]]);
        let n = u32::from_le_bytes([bytes[11], bytes[12], bytes[13], bytes[14]]) as usize;
        let use_zstd = flags & FLAG_ZSTD != 0;
        let _delta = flags & FLAG_DELTA != 0;

        // CRC32 校验 (FLAG_CHECKSUM): 文件尾 4B = 前面所有字节的 CRC32
        if flags & FLAG_CHECKSUM != 0 {
            if bytes.len() < 4 {
                return Err("truncated checksum".into());
            }
            let (body, tail) = bytes.split_at(bytes.len() - 4);
            let stored = u32::from_le_bytes([tail[0], tail[1], tail[2], tail[3]]);
            let actual = crc32fast::hash(body);
            if stored != actual {
                return Err(format!(
                    "NT-Pack checksum mismatch: stored={:#010x} actual={:#010x} (文件损坏)",
                    stored, actual
                ));
            }
        }

        // 列描述段
        let mut pos = 15usize;
        let ncols = bytes[pos] as usize;
        pos += 1;
        let mut cols = Vec::with_capacity(ncols);
        for _ in 0..ncols {
            let t = bytes[pos];
            let prec = bytes[pos + 1];
            pos += 2;
            cols.push((t, prec));
        }
        // 取 lat/lng 精度 + 检测 node_id 列
        let mut precision = 5u8;
        let mut has_node_id = false;
        for (t, prec) in &cols {
            if *t == COL_COORD_LAT && *prec != 0 {
                precision = *prec;
            }
            if *t == COL_NODE_ID {
                has_node_id = true;
            }
        }

        // 字符串段
        let dict_len = u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]) as usize;
        pos += 4;
        let mut dict: Vec<String> = Vec::with_capacity(dict_len);
        for _ in 0..dict_len {
            let slen = u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]) as usize;
            pos += 4;
            let s = String::from_utf8_lossy(&bytes[pos..pos + slen]).to_string();
            pos += slen;
            dict.push(s);
        }

        // 数据区 (坐标 + ident + 索引) — FLAG_ZSTD 时被 zstd 压缩
        let data: Vec<u8>;
        if use_zstd {
            // 压缩格式: [varint 压缩长度][zstd 数据]
            let (clen, p) = pop_varint(bytes, pos).ok_or("truncated compressed len")?;
            let clen = clen as usize;
            pos = p;
            if pos + clen <= bytes.len() {
                let compressed = &bytes[pos..pos + clen];
                use std::io::Read;
                let mut decoder = zstd::stream::read::Decoder::new(compressed)
                    .map_err(|e| format!("zstd decoder init: {}", e))?;
                let mut buf = Vec::new();
                decoder
                    .read_to_end(&mut buf)
                    .map_err(|e| format!("zstd decompress failed: {}", e))?;
                data = buf;
            } else {
                return Err("truncated compressed data".into());
            }
        } else {
            data = bytes[pos..].to_vec();
        }
        let data = data.as_slice();

        // ---- 坐标段: 解码 delta zigzag varint ----
        let scale = 10f64.powi(precision as i32);
        let mut lats = Vec::with_capacity(n);
        let mut lngs = Vec::with_capacity(n);
        let mut prev_lat = 0i64;
        let mut prev_lng = 0i64;
        let mut dp = 0usize;
        for _ in 0..n {
            let (dlat, p) = pop_zigzag_varint(data, dp).ok_or("truncated lat stream")?;
            dp = p;
            let (dlng, p2) = pop_zigzag_varint(data, dp).ok_or("truncated lng stream")?;
            dp = p2;
            prev_lat += dlat;
            prev_lng += dlng;
            lats.push(prev_lat);
            lngs.push(prev_lng);
        }

        // ---- ident 段 (node_id 列存在时): [varint prefix_idx][n × (varint len + 字节)] ----
        let mut node_id_prefix = String::new();
        let mut idents: Vec<String> = Vec::with_capacity(n);
        if has_node_id {
            let (pidx, p) = pop_varint(data, dp).ok_or("truncated prefix idx")?;
            dp = p;
            node_id_prefix = if pidx == 0 {
                String::new()
            } else {
                dict.get(pidx as usize - 1).cloned().unwrap_or_default()
            };
            for _ in 0..n {
                let (len, p2) = pop_varint(data, dp).ok_or("truncated ident len")?;
                dp = p2;
                let end = dp + len as usize;
                if end > data.len() {
                    return Err("truncated ident bytes".into());
                }
                idents.push(String::from_utf8_lossy(&data[dp..end]).to_string());
                dp = end;
            }
        }

        // ---- 索引段: 5 个字符串列 (node_id 走 ident 段) ----
        let mut idxs: Vec<Vec<u32>> = (0..5).map(|_| Vec::with_capacity(n)).collect();
        for col in 0..5 {
            for _ in 0..n {
                let (v, p) = pop_varint(data, dp).ok_or("truncated index stream")?;
                dp = p;
                idxs[col].push(v as u32);
            }
        }

        let str_of = |idx: u32| -> String {
            if idx == 0 {
                String::new()
            } else {
                dict.get(idx as usize - 1).cloned().unwrap_or_default()
            }
        };

        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let node_id = if has_node_id {
                format!("{}{}", node_id_prefix, idents[i])
            } else {
                str_of(idxs[0][i])
            };
            points.push(GeoPoint {
                node_id,
                lat: lats[i] as f64 / scale,
                lng: lngs[i] as f64 / scale,
                country: str_of(idxs[0][i]),
                region: str_of(idxs[1][i]),
                city: str_of(idxs[2][i]),
                tags: str_of(idxs[3][i]),
                source: str_of(idxs[4][i]),
            });
        }

        Ok((
            Self {
                precision,
                use_zstd,
            },
            points,
        ))
    }
}

// ---- 原语: varint / zigzag ----

/// 编码 varint (LEB128 无符号)
pub(crate) fn push_varint(out: &mut Vec<u8>, mut v: u64) {
    loop {
        let b = (v & 0x7F) as u8;
        v >>= 7;
        if v == 0 {
            out.push(b);
            break;
        }
        out.push(b | 0x80);
    }
}

/// 解码 varint
pub(crate) fn pop_varint(data: &[u8], mut pos: usize) -> Option<(u64, usize)> {
    let mut result = 0u64;
    let mut shift = 0u32;
    loop {
        let b = *data.get(pos)?;
        pos += 1;
        result |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            return Some((result, pos));
        }
        shift += 7;
        if shift > 63 {
            return None;
        }
    }
}

/// 编码 zigzag (i64 → u64)
fn zigzag(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)) as u64
}

/// 解码 zigzag (u64 → i64)
fn unzigzag(v: u64) -> i64 {
    ((v >> 1) as i64) ^ (-((v & 1) as i64))
}

pub(crate) fn push_zigzag_varint(out: &mut Vec<u8>, v: i64) {
    push_varint(out, zigzag(v));
}

pub(crate) fn pop_zigzag_varint(data: &[u8], pos: usize) -> Option<(i64, usize)> {
    let (v, p) = pop_varint(data, pos)?;
    Some((unzigzag(v), p))
}

/// 数据区压缩 (zstd, 级别 9 平衡压缩率/速度 — 实验: 9 比 3 省 4.5%, 解码速度无关)
fn compress_data_section(out: &mut Vec<u8>) {
    // 对坐标段+ident段+索引段 (即 dict 之后的所有字节) 用 zstd 压缩
    // 头部 (魔数+版本+flags+记录数+列描述+字典) 保持明文可解析
    use std::io::Write;
    let split = find_data_start(out);
    if split >= out.len() {
        return;
    }
    let payload = out.split_off(split);
    let mut encoder = zstd::stream::write::Encoder::new(Vec::new(), 9).expect("zstd encoder");
    if encoder.write_all(&payload).is_ok() {
        if let Ok(compressed) = encoder.finish() {
            // 标记: [varint 压缩长度][zstd 数据] (varint 支持 >64KB, 修复 u16 溢出)
            let mut new_data = Vec::with_capacity(4 + compressed.len());
            push_varint(&mut new_data, compressed.len() as u64);
            new_data.extend_from_slice(&compressed);
            out.extend_from_slice(&new_data);
        } else {
            out.extend_from_slice(&payload);
        }
    } else {
        out.extend_from_slice(&payload);
    }
}

/// 找数据区起点 (字典段结束位置 = 坐标段开始)
fn find_data_start(bytes: &[u8]) -> usize {
    // 魔数 8 + 版本 1 + flags 2 + 记录数 4 = 15
    let mut pos = 15usize;
    if pos >= bytes.len() {
        return bytes.len();
    }
    let ncols = bytes[pos] as usize;
    pos += 1 + ncols * 2;
    if pos + 4 > bytes.len() {
        return bytes.len();
    }
    let dict_len = u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]) as usize;
    pos += 4;
    for _ in 0..dict_len {
        if pos + 4 > bytes.len() {
            return bytes.len();
        }
        let slen = u32::from_le_bytes([bytes[pos], bytes[pos + 1], bytes[pos + 2], bytes[pos + 3]]) as usize;
        pos += 4 + slen;
    }
    pos.min(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_points() -> Vec<GeoPoint> {
        vec![
            GeoPoint {
                node_id: "geo:airport:KLAX".into(),
                lat: 33.9425,
                lng: -118.4081,
                country: "US".into(),
                region: "US-CA".into(),
                city: "Los Angeles".into(),
                tags: "机场,large_airport".into(),
                source: "ourairports".into(),
            },
            GeoPoint {
                node_id: "geo:airport:KJFK".into(),
                lat: 40.6413,
                lng: -73.7781,
                country: "US".into(),
                region: "US-NY".into(),
                city: "New York".into(),
                tags: "机场,large_airport".into(),
                source: "ourairports".into(),
            },
            GeoPoint {
                node_id: "geo:volcano:283030".into(),
                lat: 35.361,
                lng: 138.728,
                country: "Japan".into(),
                region: "".into(),
                city: "Fujisan".into(),
                tags: "火山,holocene".into(),
                source: "gvp-volcanoes".into(),
            },
            GeoPoint {
                node_id: "geo:city:beijing".into(),
                lat: 39.9042,
                lng: 116.4074,
                country: "China".into(),
                region: "CN-BJ".into(),
                city: "Beijing".into(),
                tags: "城市".into(),
                source: "geonames-cities".into(),
            },
        ]
    }

    #[test]
    fn test_roundtrip_e5() {
        let enc = PackEncoder::new(5, false);
        let pts = sample_points();
        let bytes = enc.encode(&pts);
        // 4 条样本固定开销 (魔数+列描述+字典) 占比高; 真实万条场景摊销后每记录 ~5-12B
        assert!(bytes.len() < 500, "E5 4点应 <500B, 实际 {}", bytes.len());

        let (dec, out) = PackDecoder::decode(&bytes).unwrap();
        assert_eq!(dec.precision, 5);
        assert_eq!(out.len(), pts.len());
        for (a, b) in pts.iter().zip(out.iter()) {
            assert!((a.lat - b.lat).abs() < 1e-4, "lat {} vs {}", a.lat, b.lat);
            assert!((a.lng - b.lng).abs() < 1e-4, "lng {} vs {}", a.lng, b.lng);
            assert_eq!(a.node_id, b.node_id);
            assert_eq!(a.country, b.country);
            assert_eq!(a.city, b.city);
            assert_eq!(a.tags, b.tags);
            assert_eq!(a.source, b.source);
        }
    }

    #[test]
    fn test_roundtrip_e6_zstd() {
        let enc = PackEncoder::new(6, true);
        let pts = sample_points();
        let bytes = enc.encode(&pts);
        let (dec, out) = PackDecoder::decode(&bytes).unwrap();
        assert_eq!(dec.precision, 6);
        assert_eq!(out.len(), pts.len());
        for (a, b) in pts.iter().zip(out.iter()) {
            assert!((a.lat - b.lat).abs() < 1e-5, "E6 lat {}", a.lat);
            assert!((a.lng - b.lng).abs() < 1e-5, "E6 lng {}", a.lng);
        }
    }

    #[test]
    fn test_varint_roundtrip() {
        for v in [0u64, 1, 127, 128, 300, 16383, 16384, 2_000_000_000] {
            let mut buf = Vec::new();
            push_varint(&mut buf, v);
            let (got, _) = pop_varint(&buf, 0).unwrap();
            assert_eq!(got, v, "varint {}", v);
        }
    }

    #[test]
    fn test_zigzag_roundtrip() {
        for v in [0i64, 1, -1, 2, -2, 100, -100, i64::MAX, i64::MIN] {
            assert_eq!(unzigzag(zigzag(v)), v, "zigzag {}", v);
        }
    }

    #[test]
    fn test_magic_reject() {
        assert!(PackDecoder::decode(b"NOTPACK00000000").is_err());
    }

    #[test]
    fn test_checksum_detects_tampering() {
        let enc = PackEncoder::new(5, true);
        let pts = sample_points();
        let mut bytes = enc.encode(&pts);
        // 正常解码通过
        assert!(PackDecoder::decode(&bytes).is_ok());
        // 篡改数据区一个字节 → 校验失败
        let mid = bytes.len() / 2;
        bytes[mid] ^= 0xFF;
        let err = PackDecoder::decode(&bytes).unwrap_err();
        assert!(err.contains("checksum"), "应报 checksum 错误, 实际: {}", err);
    }

    #[test]
    fn test_checksum_flag_present() {
        let enc = PackEncoder::new(5, false);
        let bytes = enc.encode(&sample_points());
        let flags = u16::from_le_bytes([bytes[9], bytes[10]]);
        assert!(flags & FLAG_CHECKSUM != 0, "编码器应设置 FLAG_CHECKSUM");
        // 文件尾 4B 是 CRC32
        assert!(bytes.len() >= 4);
    }

    /// 伪模糊 roundtrip: 随机数据 × 4 编解码组合 (E5/E6 × zstd/no-zstd)
    /// 覆盖混合 node_id 前缀 (LCP 退化)、空字段、边界坐标。
    #[test]
    fn test_roundtrip_fuzz_random() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let sources = ["ourairports", "gvp-volcanoes", "geonames-cities", ""];
        let kinds = ["airport", "volcano", "city", ""];

        for round in 0..100 {
            let n = rng.gen_range(1..200usize);
            let pts: Vec<GeoPoint> = (0..n)
                .map(|_| GeoPoint {
                    node_id: format!(
                        "geo:{}:{}",
                        kinds[rng.gen_range(0..kinds.len())],
                        rng.gen_range(0..100_000)
                    ),
                    lat: rng.gen_range(-90.0..90.0),
                    lng: rng.gen_range(-180.0..180.0),
                    country: if rng.gen_bool(0.1) { String::new() } else { format!("C{}", rng.gen_range(0..20)) },
                    region: if rng.gen_bool(0.1) { String::new() } else { format!("R{}", rng.gen_range(0..50)) },
                    city: if rng.gen_bool(0.1) { String::new() } else { format!("City{}", rng.gen_range(0..500)) },
                    tags: if rng.gen_bool(0.2) { String::new() } else { "机场,small_airport".into() },
                    source: sources[rng.gen_range(0..sources.len())].into(),
                })
                .collect();

            for precision in [5u8, 6u8] {
                for use_zstd in [false, true] {
                    let enc = PackEncoder::new(precision, use_zstd);
                    let bytes = enc.encode(&pts);
                    let (dec, out) = PackDecoder::decode(&bytes)
                        .unwrap_or_else(|e| panic!("round {} prec {} zstd {}: {}", round, precision, use_zstd, e));
                    assert_eq!(out.len(), n, "round {}: 条数", round);
                    assert_eq!(dec.precision, precision);
                    assert_eq!(dec.use_zstd, use_zstd);
                    let tol = 1.0 / 10f64.powi(precision as i32);
                    for (a, b) in pts.iter().zip(out.iter()) {
                        assert_eq!(a.node_id, b.node_id, "round {}: node_id", round);
                        assert_eq!(a.country, b.country, "round {}: country", round);
                        assert_eq!(a.region, b.region, "round {}: region", round);
                        assert_eq!(a.city, b.city, "round {}: city", round);
                        assert_eq!(a.tags, b.tags, "round {}: tags", round);
                        assert_eq!(a.source, b.source, "round {}: source", round);
                        assert!((a.lat - b.lat).abs() < tol, "round {}: lat", round);
                        assert!((a.lng - b.lng).abs() < tol, "round {}: lng", round);
                    }
                }
            }
        }
    }
}

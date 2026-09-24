//! BBCode 富文本解析（纯逻辑，色键字符串，渲染层映射颜色）.

/// 带色文本段（color 为色键：gold/red/blue/green/purple/orange/pink/aqua）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub color: Option<&'static str>,
}

fn tag_color(tag: &str) -> Option<&'static str> {
    match tag {
        "gold" => Some("gold"),
        "red" => Some("red"),
        "blue" => Some("blue"),
        "green" => Some("green"),
        "purple" => Some("purple"),
        "orange" => Some("orange"),
        "pink" => Some("pink"),
        "aqua" => Some("aqua"),
        _ => None,
    }
}

/// 解析 BBCode（含 StS 特化标签），未知标签脱签留字。
pub fn parse_bbcode(s: &str) -> Vec<Segment> {
    let mut out: Vec<Segment> = Vec::new();
    let mut cur = String::new();
    let mut color: Option<&'static str> = None;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let flush = |out: &mut Vec<Segment>, cur: &mut String, color: &Option<&'static str>| {
        if !cur.is_empty() {
            out.push(Segment { text: std::mem::take(cur), color: *color });
        }
    };
    while i < chars.len() {
        if chars[i] == '[' {
            if let Some(end) = chars[i..].iter().position(|&c| c == ']') {
                let tag: String = chars[i + 1..i + end].iter().collect();
                let tag_low = tag.to_lowercase();
                if tag_low.starts_with("energy:") {
                    flush(&mut out, &mut cur, &color);
                    let n = tag_low["energy:".len()..].trim();
                    cur.push_str(&format!("({}费)", n));
                } else if tag_low.starts_with("star:") {
                    flush(&mut out, &mut cur, &color);
                    let n = tag_low["star:".len()..].trim();
                    cur.push_str(&format!("({}星)", n));
                } else if tag_low == "card" {
                    cur.push_str("此牌");
                } else if tag_low == "relic" {
                    cur.push_str("遗物");
                } else if tag_low.starts_with('/') {
                    flush(&mut out, &mut cur, &color);
                    color = None;
                } else if let Some(c) = tag_color(&tag_low) {
                    flush(&mut out, &mut cur, &color);
                    color = Some(c);
                }
                // 未知[b]/[i]/[sine]/[jitter] 等：脱签留字（不进分支即跳过）
                i += end + 1;
                continue;
            }
        }
        cur.push(chars[i]);
        i += 1;
    }
    flush(&mut out, &mut cur, &color);
    out
}

/// 纯文本（脱签），供无需着色的旧调用点。
pub fn plain_text(s: &str) -> String {
    parse_bbcode(s).into_iter().map(|seg| seg.text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_segments() {
        let segs = parse_bbcode("Gain [gold]Dexterity[/gold].");
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0], Segment { text: "Gain ".to_string(), color: None });
        assert_eq!(segs[1].text, "Dexterity");
        assert_eq!(segs[1].color, tag_color("gold"));
        assert_eq!(segs[2].text, ".");
    }

    #[test]
    fn nested_and_modifiers_stripped() {
        let segs = parse_bbcode("[b][jitter]CLANG![/jitter][/b]");
        assert_eq!(plain_text("[b][jitter]CLANG![/jitter][/b]"), "CLANG!");
        assert_eq!(segs.len(), 1);
    }

    #[test]
    fn energy_and_placeholders() {
        assert_eq!(plain_text("[energy:2]"), "(2费)");
        assert_eq!(plain_text("[star:1]"), "(1星)");
        assert_eq!(plain_text("[Card]"), "此牌");
        assert_eq!(plain_text("获得[blue]2[/blue]点"), "获得2点");
    }

    #[test]
    fn unterminated_bracket_kept() {
        assert_eq!(plain_text("a[b"), "a[b");
    }
}

use super::{FileParser, SpatialBlock, BlockType};

impl FileParser {
    pub(super) fn extract_pdf_text(data: &[u8]) -> String {
        let blocks = Self::extract_pdf_spatial(data);
        blocks.iter().map(|b| b.text.as_str()).collect::<Vec<_>>().join(" ")
    }

    pub(super) fn extract_pdf_spatial(data: &[u8]) -> Vec<SpatialBlock> {
        let content = String::from_utf8_lossy(data);
        let mut blocks = Vec::new();
        let mut cur_x = 0.0f32;
        let mut cur_y = 0.0f32;
        let mut in_text = false;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("BT") {
                in_text = true;
                cur_x = 0.0;
                cur_y = 0.0;
                continue;
            }
            if line.starts_with("ET") {
                in_text = false;
                continue;
            }
            if !in_text {
                continue;
            }

            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }

            for (i, token) in tokens.iter().enumerate() {
                match *token {
                    "Tm" if i >= 6 => {
                        let e = tokens[i - 2].parse::<f32>().unwrap_or(cur_x);
                        let f = tokens[i - 1].parse::<f32>().unwrap_or(cur_y);
                        cur_x = e;
                        cur_y = f;
                    }
                    "Td" if i >= 2 => {
                        let tx = tokens[i - 2].parse::<f32>().unwrap_or(0.0);
                        let ty = tokens[i - 1].parse::<f32>().unwrap_or(0.0);
                        cur_x += tx;
                        cur_y += ty;
                    }
                    "T*" => {
                        cur_y -= 14.0;
                    }
                    _ => {}
                }
            }

            if let Some(start) = line.find('(') {
                if let Some(end) = line.rfind(')') {
                    if start < end && line[end..].contains("Tj") {
                        let text = &line[start + 1..end];
                        if !text.is_empty() && text.chars().any(|c| c.is_alphabetic()) {
                            blocks.push(SpatialBlock {
                                x: cur_x,
                                y: cur_y,
                                width: text.len() as f32 * 5.0,
                                height: 12.0,
                                text: text.to_string(),
                                block_type: BlockType::TextBlock,
                            });
                        }
                    }
                }
            }
        }

        if blocks.is_empty() {
            for line in content.lines() {
                let line = line.trim();
                if let Some(start) = line.find('(') {
                    if let Some(end) = line.rfind(')') {
                        if start < end && (start == 0 || !line[..start].contains('\\')) {
                            let text = &line[start + 1..end];
                            if text.len() > 3 && text.chars().filter(|&c| c.is_alphabetic()).count() > 3 {
                                blocks.push(SpatialBlock {
                                    x: 0.0,
                                    y: 0.0,
                                    width: text.len() as f32 * 5.0,
                                    height: 12.0,
                                    text: text.to_string(),
                                    block_type: BlockType::TextBlock,
                                });
                            }
                        }
                    }
                }
            }
        }

        blocks.sort_by(|a, b| {
            b.y.partial_cmp(&a.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    a.x.partial_cmp(&b.x)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        blocks
    }
}

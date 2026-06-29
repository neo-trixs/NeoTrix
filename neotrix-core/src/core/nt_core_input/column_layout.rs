#![allow(dead_code)]

/// Multi-column layout detection and read-order correction.
///
/// All algorithms are pure heuristic, zero external ML dependencies.
/// Column detection: cluster text segments by x-position.
/// Region classification: classify by position, size, and content patterns.
/// Table detection: detect aligned columns of short text fragments.
/// Heading detection: detect by position (top of page/column) and spacing.
use std::collections::HashMap;

/// Represents a positioned text segment on a page.
#[derive(Debug, Clone)]
pub struct TextSegment {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub text: String,
    pub page_num: usize,
}

/// Detected column information.
#[derive(Debug, Clone)]
pub struct ColumnLayout {
    pub column_count: usize,
    pub columns: Vec<Column>,
    pub layout_type: LayoutType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutType {
    SingleColumn,
    TwoColumn,
    ThreeColumn,
    Mixed,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub index: usize,
    pub x_start: f32,
    pub x_end: f32,
    pub segments: Vec<TextSegment>,
}

/// A detected layout region with semantic type.
#[derive(Debug, Clone)]
pub struct LayoutRegion {
    pub bbox: [f32; 4],
    pub region_type: LayoutRegionType,
    pub confidence: f32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LayoutRegionType {
    TextBlock,
    Heading,
    Title,
    Table,
    Figure,
    Formula,
    Caption,
    ListItem,
    PageHeader,
    PageFooter,
}

impl LayoutRegionType {
    pub fn name(&self) -> &'static str {
        match self {
            LayoutRegionType::TextBlock => "text",
            LayoutRegionType::Heading => "heading",
            LayoutRegionType::Title => "title",
            LayoutRegionType::Table => "table",
            LayoutRegionType::Figure => "figure",
            LayoutRegionType::Formula => "formula",
            LayoutRegionType::Caption => "caption",
            LayoutRegionType::ListItem => "list-item",
            LayoutRegionType::PageHeader => "page-header",
            LayoutRegionType::PageFooter => "page-footer",
        }
    }
}

/// Full page layout analysis combining columns and semantic regions.
#[derive(Debug, Clone)]
pub struct PageLayout {
    pub page_width: f32,
    pub page_height: f32,
    pub columns: Vec<Column>,
    pub layout_type: LayoutType,
    pub regions: Vec<LayoutRegion>,
}

/// Determine average line height from segments.
fn avg_line_height(segments: &[TextSegment]) -> f32 {
    if segments.is_empty() {
        return 14.0;
    }
    let sum: f32 = segments.iter().map(|s| s.height).sum();
    (sum / segments.len() as f32).max(6.0)
}

/// Compute page bounds from segments.
fn page_bounds(segments: &[TextSegment]) -> (f32, f32) {
    if segments.is_empty() {
        return (800.0, 600.0);
    }
    let max_x = segments
        .iter()
        .map(|s| s.x + s.width)
        .fold(0.0f32, f32::max);
    let max_y = segments
        .iter()
        .map(|s| s.y + s.height)
        .fold(0.0f32, f32::max);
    (max_x.max(600.0), max_y.max(800.0))
}

/// Cluster y-positions into heading groups.
/// Headings typically sit at distinct y positions with spacing above/below.
fn detect_headings_and_titles(segments: &[TextSegment], page_height: f32) -> Vec<LayoutRegion> {
    if segments.len() < 3 {
        return vec![];
    }

    let mut regions = Vec::new();
    let lh = avg_line_height(segments);

    // Sort segments top-to-bottom.
    let mut sorted: Vec<&TextSegment> = segments.iter().collect();
    sorted.sort_by(|a, b| b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal));

    // Detect potential title: text near top of page, short, no preceding segment close.
    if let Some(first) = sorted.first() {
        if first.y > page_height * 0.85 && first.text.len() < 60 && first.text.len() > 3 {
            regions.push(LayoutRegion {
                bbox: [
                    first.x,
                    first.y,
                    first.x + first.width,
                    first.y + first.height,
                ],
                region_type: LayoutRegionType::Title,
                confidence: 0.6,
                text: first.text.clone(),
            });
        }
    }

    // Detect headings by: short text (<50 chars), preceded by vertical gap (>1.5x line height).
    for i in 1..sorted.len() {
        let seg = sorted[i];
        let prev = sorted[i - 1];
        let gap = prev.y - (seg.y + seg.height);

        let is_short = seg.text.len() < 50;
        let is_capitalized = seg
            .text
            .chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        let has_gap = gap > lh * 1.5;

        if is_short && is_capitalized && has_gap && seg.text.len() > 2 {
            regions.push(LayoutRegion {
                bbox: [seg.x, seg.y, seg.x + seg.width, seg.y + seg.height],
                region_type: LayoutRegionType::Heading,
                confidence: 0.5 + (gap / (lh * 3.0)).min(0.4),
                text: seg.text.clone(),
            });
        }
    }

    regions
}

/// Detect running headers and footers by consistent y-position across segments.
/// Headers: near top of page. Footers: near bottom of page.
fn detect_header_footer(segments: &[TextSegment], page_height: f32) -> Vec<LayoutRegion> {
    let mut regions = Vec::new();

    // Collect y-positions with frequency.
    let mut y_counts: HashMap<u32, usize> = HashMap::new();
    for seg in segments {
        let y_key = (seg.y / 5.0).round() as u32;
        *y_counts.entry(y_key).or_insert(0) += 1;
    }

    // Threshold: a line appearing at >20% of segments at same y is likely header/footer.
    let threshold = (segments.len() / 5).max(2);

    for (&y_key, &count) in &y_counts {
        if count < threshold {
            continue;
        }
        let y_center = y_key as f32 * 5.0;
        let is_header = y_center > page_height * 0.9;
        let is_footer = y_center < page_height * 0.1;

        if is_header || is_footer {
            let rtype = if is_header {
                LayoutRegionType::PageHeader
            } else {
                LayoutRegionType::PageFooter
            };
            // Collect all segments at this y.
            let mut text = String::new();
            for seg in segments {
                let sy = (seg.y / 5.0).round() as u32;
                if sy == y_key {
                    text.push_str(&seg.text);
                    text.push(' ');
                }
            }
            if !text.is_empty() {
                regions.push(LayoutRegion {
                    bbox: [0.0, y_center - 10.0, 0.0, y_center + 10.0],
                    region_type: rtype,
                    confidence: 0.7,
                    text: text.trim().to_string(),
                });
            }
        }
    }

    regions
}

/// Detect tabular regions: segments where multiple lines have similarly spaced
/// x-positions (indicating aligned columns of data).
fn detect_tables(segments: &[TextSegment]) -> Vec<LayoutRegion> {
    if segments.len() < 6 {
        return vec![];
    }

    let mut regions = Vec::new();
    let lh = avg_line_height(segments);

    // Sort top-to-bottom.
    let mut sorted: Vec<&TextSegment> = segments.iter().collect();
    sorted.sort_by(|a, b| b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal));

    // Look for sequences of lines with similar x-patterns (multi-column data rows).
    let mut i = 0;
    while i < sorted.len() - 2 {
        let table_start = i;
        let mut table_end = i + 1;

        // Try to extend table: check if next line has multiple x-clusters (aligned columns).
        while table_end < sorted.len() {
            let row = sorted[table_end];
            let prev = sorted[table_end - 1];

            // Vertical gap too large → likely not same table.
            let gap = prev.y - (row.y + row.height);
            if gap > lh * 2.5 {
                break;
            }

            // Check if this row has short fragments suggesting table cells.
            let is_tabular = row.text.len() < 80 && row.text.len() > 2;
            if !is_tabular {
                break;
            }

            table_end += 1;
        }

        let row_count = table_end - table_start;
        if row_count >= 3 {
            let start_seg = sorted[table_start];
            let end_seg = sorted[table_end - 1];

            let x0 = start_seg.x;
            let y0 = start_seg.y + start_seg.height;
            let x1 = end_seg.x + end_seg.width;
            let y1 = end_seg.y;

            let mut text = String::new();
            for j in table_start..table_end {
                text.push_str(&sorted[j].text);
                text.push('\n');
            }

            regions.push(LayoutRegion {
                bbox: [x0, y1, x1, y0],
                region_type: LayoutRegionType::Table,
                confidence: 0.5 + (row_count as f32 * 0.05).min(0.4),
                text,
            });

            i = table_end;
            continue;
        }

        i += 1;
    }

    regions
}

/// Detect formula regions: segments containing LaTeX math markers or
/// isolated short centered text.
fn detect_formulas(segments: &[TextSegment], page_width: f32) -> Vec<LayoutRegion> {
    let mut regions = Vec::new();

    for seg in segments {
        let trimmed = seg.text.trim();
        let is_latex = trimmed.starts_with("$$")
            || trimmed.ends_with("$$")
            || (trimmed.starts_with('$') && trimmed.ends_with('$') && trimmed.len() > 3)
            || trimmed.contains("\\begin{equation")
            || trimmed.contains("\\begin{align")
            || trimmed.contains("\\frac");
        let is_centered =
            (seg.x - page_width / 2.0).abs() < page_width * 0.15 && seg.text.len() < 100;

        if is_latex || (is_centered && seg.text.len() >= 4) {
            regions.push(LayoutRegion {
                bbox: [seg.x, seg.y, seg.x + seg.width, seg.y + seg.height],
                region_type: LayoutRegionType::Formula,
                confidence: if is_latex { 0.8 } else { 0.4 },
                text: seg.text.clone(),
            });
        }
    }

    regions
}

/// Detect list items: segments starting with bullet markers (•, -, *, 1., a), etc.
fn detect_list_items(segments: &[TextSegment]) -> Vec<LayoutRegion> {
    let mut regions = Vec::new();

    // Group consecutive segments that look like list items.
    let mut sorted: Vec<&TextSegment> = segments.iter().collect();
    sorted.sort_by(|a, b| b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal));

    let mut i = 0;
    while i < sorted.len() {
        let seg = sorted[i];
        let trimmed = seg.text.trim();
        let is_list_item = trimmed.starts_with("•")
            || trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("◦")
            || trimmed.starts_with("▪")
            || (trimmed.len() > 2
                && trimmed.chars().next().map_or(false, |c| c.is_ascii_digit())
                && trimmed.contains("."));

        if is_list_item {
            // Collect consecutive list items.
            let mut j = i;
            let mut text = String::new();
            while j < sorted.len() {
                let t = sorted[j].text.trim().to_string();
                let is_item = t.starts_with("•")
                    || t.starts_with("- ")
                    || t.starts_with("* ")
                    || t.starts_with("◦")
                    || t.starts_with("▪")
                    || (t.len() > 2
                        && t.chars().next().map_or(false, |c| c.is_ascii_digit())
                        && t.contains("."));
                if !is_item {
                    break;
                }
                text.push_str(&t);
                text.push('\n');
                j += 1;
            }

            // Need at least 2 items.
            if j - i >= 2 {
                let first = sorted[i];
                let last = sorted[j - 1];
                regions.push(LayoutRegion {
                    bbox: [
                        first.x,
                        last.y,
                        first.x + first.width,
                        first.y + first.height,
                    ],
                    region_type: LayoutRegionType::ListItem,
                    confidence: 0.7,
                    text: text.trim().to_string(),
                });
            }
            i = j;
            continue;
        }
        i += 1;
    }

    regions
}

/// A block produced by recursive X-Y cut — a group of segments that are
/// spatially cohesive and form a single reading unit.
#[derive(Debug, Clone)]
pub struct XyCutBlock {
    pub bbox: [f32; 4],
    pub segments: Vec<TextSegment>,
    pub depth: usize,
    pub is_mask: bool,
}

impl XyCutBlock {
    fn from_segments(segments: &[TextSegment], depth: usize) -> Self {
        let min_x = segments.iter().map(|s| s.x).fold(f32::MAX, f32::min);
        let min_y = segments.iter().map(|s| s.y).fold(f32::MAX, f32::min);
        let max_x = segments
            .iter()
            .map(|s| s.x + s.width)
            .fold(f32::MIN, f32::max);
        let max_y = segments
            .iter()
            .map(|s| s.y + s.height)
            .fold(f32::MIN, f32::max);
        Self {
            bbox: [min_x, min_y, max_x, max_y],
            segments: segments.to_vec(),
            depth,
            is_mask: false,
        }
    }
}

/// A cross-layout element detected and temporarily masked before XY-Cut.
/// These are segments spanning most of the page width (titles, full-width headings,
/// headers, footers, wide figures/tables).
#[derive(Debug, Clone)]
pub struct CrossLayoutMask {
    pub bbox: [f32; 4],
    pub segments: Vec<TextSegment>,
    /// E.g., "title", "full_heading", "header", "footer", "wide_figure", "wide_table"
    pub mask_type: &'static str,
    /// Priority in reading order (0 = top of page)
    pub y_anchor: f32,
}

/// XY-Cut++ config extending the base XY-Cut with pre-mask processing and
/// density-aware splitting.
#[derive(Debug, Clone)]
pub struct XyCutPlusPlusConfig {
    pub base: XyCutConfig,
    /// Width ratio threshold for cross-layout detection (default: 0.70)
    pub cross_layout_width_ratio: f32,
    /// Minimum area ratio for figure/table masking (default: 0.15)
    pub large_element_area_ratio: f32,
    /// Density threshold: segments-per-vertical-px above which horizontal cuts
    /// are preferred (default: 0.05)
    pub density_threshold: f32,
    /// Bias weight for density-aware splitting (default: 1.3)
    pub density_bias: f32,
}

impl Default for XyCutPlusPlusConfig {
    fn default() -> Self {
        Self {
            base: XyCutConfig::default(),
            cross_layout_width_ratio: 0.70,
            large_element_area_ratio: 0.15,
            density_threshold: 0.05,
            density_bias: 1.3,
        }
    }
}

/// Detect cross-layout elements that span most of the page width and should be
/// masked before XY-Cut segmentation.
///
/// These include:
/// - Titles: near top of page, short text
/// - Full-width headings: span >70% page width
/// - Page headers/footers: consistent y-position
/// - Wide figures/tables: large area, span most of page width
fn detect_cross_layout_masks(
    segments: &[TextSegment],
    page_width: f32,
    page_height: f32,
    config: &XyCutPlusPlusConfig,
) -> Vec<CrossLayoutMask> {
    if segments.is_empty() {
        return vec![];
    }

    let mut masks = Vec::new();
    let width_threshold = page_width * config.cross_layout_width_ratio;
    let area_threshold = page_width * page_height * config.large_element_area_ratio;

    // Collect segments into candidate masks by y-proximity.
    let mut sorted: Vec<&TextSegment> = segments.iter().collect();
    sorted.sort_by(|a, b| b.y.partial_cmp(&a.y).unwrap_or(std::cmp::Ordering::Equal));

    let mut used = vec![false; sorted.len()];
    let lh = avg_line_height(segments);

    for i in 0..sorted.len() {
        if used[i] {
            continue;
        }
        let seg = sorted[i];
        let seg_width = seg.width;
        let seg_height = seg.height;
        let seg_area = seg_width * seg_height;

        // Check if this is a cross-layout element.
        let is_full_width = seg_width > width_threshold;
        let is_large = seg_area > area_threshold;

        if !is_full_width && !is_large {
            continue;
        }

        // Gather adjacent segments at roughly the same y-band.
        let mut mask_segs = Vec::new();
        mask_segs.push(seg.clone());
        used[i] = true;

        for j in (i + 1)..sorted.len() {
            if used[j] {
                continue;
            }
            let other = sorted[j];
            let other_width = other.width;
            let other_area = other_width * other.height;

            let y_gap = (mask_segs.last().unwrap().y - (other.y + other.height)).abs();
            if y_gap > lh * 3.0 {
                break;
            }

            // Merge if also cross-layout or close by.
            let other_full = other_width > width_threshold;
            let other_large = other_area > area_threshold * 0.5;
            if other_full || other_large || y_gap < lh * 0.5 {
                mask_segs.push(other.clone());
                used[j] = true;
            }
        }

        let min_x = mask_segs.iter().map(|s| s.x).fold(f32::MAX, f32::min);
        let min_y = mask_segs.iter().map(|s| s.y).fold(f32::MAX, f32::min);
        let max_x = mask_segs
            .iter()
            .map(|s| s.x + s.width)
            .fold(f32::MIN, f32::max);
        let max_y = mask_segs
            .iter()
            .map(|s| s.y + s.height)
            .fold(f32::MIN, f32::max);
        let bbox = [min_x, min_y, max_x, max_y];
        let avg_y = (min_y + max_y) / 2.0;

        // Classify mask type.
        let mask_type = if is_large && !is_full_width {
            // Large but not full-width → figure or table
            if seg_height > page_height * 0.3 {
                "wide_figure"
            } else {
                "wide_table"
            }
        } else if avg_y > page_height * 0.9 {
            "header"
        } else if avg_y < page_height * 0.1 {
            "footer"
        } else if (max_y - min_y) < lh * 2.5 && mask_segs.len() <= 2 {
            "title"
        } else {
            "full_heading"
        };

        masks.push(CrossLayoutMask {
            bbox,
            segments: mask_segs,
            mask_type,
            y_anchor: avg_y,
        });
    }

    masks
}

/// XY-Cut++ entry point: detect masks → XY-Cut on remaining → merge.
///
/// 1. Detect cross-layout elements and remove them as masks.
/// 2. Run density-aware recursive XY-Cut on the remaining segments.
/// 3. Sort masks into reading order and prepend/intersperse with cut blocks.
pub fn recursive_xy_cut_plusplus(
    segments: &[TextSegment],
    config: &XyCutPlusPlusConfig,
) -> Vec<XyCutBlock> {
    if segments.is_empty() {
        return vec![];
    }

    let (page_width, page_height) = page_bounds(segments);

    // Step 1: Detect cross-layout masks.
    let masks = detect_cross_layout_masks(segments, page_width, page_height, config);

    // Build set of masked segment indices.
    let masked_set: Vec<bool> = {
        let mut v = vec![false; segments.len()];
        for mask in &masks {
            for seg in &mask.segments {
                if let Some(idx) = segments.iter().position(|s| {
                    (s.x - seg.x).abs() < 1.0 && (s.y - seg.y).abs() < 1.0 && s.text == seg.text
                }) {
                    v[idx] = true;
                }
            }
        }
        v
    };

    // Step 2: Filter out masked segments.
    let unmasked: Vec<TextSegment> = segments
        .iter()
        .enumerate()
        .filter(|(i, _)| !masked_set[*i])
        .map(|(_, s)| s.clone())
        .collect();

    // Step 3: Run density-aware XY-Cut on unmasked segments.
    let mut blocks = if unmasked.is_empty() {
        Vec::new()
    } else {
        let avg_h = avg_line_height(&unmasked);
        let min_gap = avg_h * config.base.min_gap_ratio;
        let mut result = Vec::new();
        density_aware_xy_cut(&unmasked, &config.base, min_gap, 0, &mut result, config);
        result.sort_by(|a, b| {
            let y_cmp = b.bbox[1]
                .partial_cmp(&a.bbox[1])
                .unwrap_or(std::cmp::Ordering::Equal);
            if y_cmp != std::cmp::Ordering::Equal {
                return y_cmp;
            }
            a.bbox[0]
                .partial_cmp(&b.bbox[0])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        result
    };

    // Step 4: Convert masks to XyCutBlocks tagged is_mask=true.
    let mut mask_blocks: Vec<XyCutBlock> = masks
        .into_iter()
        .map(|m| {
            let mut block = XyCutBlock::from_segments(&m.segments, 0);
            block.is_mask = true;
            block
        })
        .collect();

    // Step 5: Merge masks into blocks at correct reading order.
    // Sort masks by y_anchor descending (top of page first).
    mask_blocks.sort_by(|a, b| {
        let a_y = a.segments.iter().map(|s| s.y).fold(f32::MAX, f32::min);
        let b_y = b.segments.iter().map(|s| s.y).fold(f32::MAX, f32::min);
        b_y.partial_cmp(&a_y).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Interleave: place each mask before the first block whose top is lower.
    for mask in mask_blocks {
        let mask_top_y = mask.segments.iter().map(|s| s.y).fold(f32::MAX, f32::min);
        let insert_pos = blocks
            .iter()
            .position(|b| {
                let b_top_y = b.segments.iter().map(|s| s.y).fold(f32::MAX, f32::min);
                b_top_y < mask_top_y
            })
            .unwrap_or(blocks.len());
        blocks.insert(insert_pos, mask);
    }

    blocks
}

/// Density-aware recursive XY cut: when both x and y gaps are viable, the
/// density of the region biases the cut choice. High density → prefer horizontal
/// cuts (reading flow). Low density → prefer vertical cuts (column separation).
fn density_aware_xy_cut(
    segments: &[TextSegment],
    config: &XyCutConfig,
    min_gap: f32,
    depth: usize,
    blocks: &mut Vec<XyCutBlock>,
    pp_config: &XyCutPlusPlusConfig,
) {
    if segments.is_empty() {
        return;
    }
    if segments.len() <= config.min_segments_per_block || depth >= config.max_depth {
        let mut block = XyCutBlock::from_segments(segments, depth);
        block.is_mask = false;
        blocks.push(block);
        return;
    }

    // Compute midpoints.
    let mut x_mids: Vec<f32> = segments.iter().map(|s| s.x + s.width / 2.0).collect();
    let mut y_mids: Vec<f32> = segments.iter().map(|s| s.y + s.height / 2.0).collect();
    x_mids.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    y_mids.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Find largest x-gap.
    let mut best_x_gap = 0.0f32;
    let mut best_x_pos = 0.0f32;
    for pair in x_mids.windows(2) {
        let gap = pair[1] - pair[0];
        if gap > best_x_gap {
            best_x_gap = gap;
            best_x_pos = (pair[0] + pair[1]) / 2.0;
        }
    }

    // Find largest y-gap.
    let mut best_y_gap = 0.0f32;
    let mut best_y_pos = 0.0f32;
    for pair in y_mids.windows(2) {
        let gap = pair[1] - pair[0];
        if gap > best_y_gap {
            best_y_gap = gap;
            best_y_pos = (pair[0] + pair[1]) / 2.0;
        }
    }

    // Compute density: segments per vertical pixel of bounding box.
    let min_y_all = segments.iter().map(|s| s.y).fold(f32::MAX, f32::min);
    let max_y_all = segments
        .iter()
        .map(|s| s.y + s.height)
        .fold(f32::MIN, f32::max);
    let height_span = (max_y_all - min_y_all).max(1.0);
    let density = segments.len() as f32 / height_span;

    // Density-aware decision: high density → favor horizontal (y) cuts.
    let prefer_horizontal = density > pp_config.density_threshold;

    let x_gap_ok = best_x_gap >= min_gap;
    let y_gap_ok = best_y_gap >= min_gap;

    if x_gap_ok && !y_gap_ok {
        // Only x-gap viable → vertical cut.
        let (left, right): (Vec<TextSegment>, Vec<TextSegment>) = segments
            .iter()
            .cloned()
            .partition(|s| s.x + s.width / 2.0 < best_x_pos);
        if !left.is_empty() && !right.is_empty() {
            density_aware_xy_cut(&left, config, min_gap, depth + 1, blocks, pp_config);
            density_aware_xy_cut(&right, config, min_gap, depth + 1, blocks, pp_config);
            return;
        }
    } else if y_gap_ok && !x_gap_ok {
        // Only y-gap viable → horizontal cut.
        let (bottom, top): (Vec<TextSegment>, Vec<TextSegment>) = segments
            .iter()
            .cloned()
            .partition(|s| s.y + s.height / 2.0 < best_y_pos);
        if !bottom.is_empty() && !top.is_empty() {
            density_aware_xy_cut(&bottom, config, min_gap, depth + 1, blocks, pp_config);
            density_aware_xy_cut(&top, config, min_gap, depth + 1, blocks, pp_config);
            return;
        }
    } else if x_gap_ok && y_gap_ok {
        // Both gaps viable — density bias decides.
        let x_biased_gap = if prefer_horizontal {
            best_x_gap / pp_config.density_bias
        } else {
            best_x_gap * pp_config.density_bias
        };
        let y_biased_gap = if !prefer_horizontal {
            best_y_gap / pp_config.density_bias
        } else {
            best_y_gap * pp_config.density_bias
        };

        if x_biased_gap >= y_biased_gap {
            let (left, right): (Vec<TextSegment>, Vec<TextSegment>) = segments
                .iter()
                .cloned()
                .partition(|s| s.x + s.width / 2.0 < best_x_pos);
            if !left.is_empty() && !right.is_empty() {
                density_aware_xy_cut(&left, config, min_gap, depth + 1, blocks, pp_config);
                density_aware_xy_cut(&right, config, min_gap, depth + 1, blocks, pp_config);
                return;
            }
        } else {
            let (bottom, top): (Vec<TextSegment>, Vec<TextSegment>) = segments
                .iter()
                .cloned()
                .partition(|s| s.y + s.height / 2.0 < best_y_pos);
            if !bottom.is_empty() && !top.is_empty() {
                density_aware_xy_cut(&bottom, config, min_gap, depth + 1, blocks, pp_config);
                density_aware_xy_cut(&top, config, min_gap, depth + 1, blocks, pp_config);
                return;
            }
        }
    }

    // Leaf block.
    let mut block = XyCutBlock::from_segments(segments, depth);
    block.is_mask = false;
    blocks.push(block);
}

/// Configuration for recursive X-Y cut.
#[derive(Debug, Clone)]
pub struct XyCutConfig {
    pub min_gap_ratio: f32,
    pub max_depth: usize,
    pub min_segments_per_block: usize,
}

impl Default for XyCutConfig {
    fn default() -> Self {
        Self {
            min_gap_ratio: 1.5,
            max_depth: 12,
            min_segments_per_block: 1,
        }
    }
}

/// Recursive X-Y cut page segmentation.
///
/// Projects segment bounding boxes onto X and Y axes, finds the largest
/// whitespace valley, and recursively partitions the page. The result is a
/// list of blocks in reading order: top-to-bottom then left-to-right.
///
/// Reference: Nagy et al., "A document image segmentation method using
/// recursive X-Y cut", 1992.
pub fn recursive_xy_cut(segments: &[TextSegment], config: &XyCutConfig) -> Vec<XyCutBlock> {
    if segments.is_empty() {
        return vec![];
    }

    let mut blocks = Vec::new();
    let avg_h = avg_line_height(segments);
    let min_gap = avg_h * config.min_gap_ratio;

    xy_cut_recursive(segments, config, min_gap, 0, &mut blocks);

    // Sort blocks in reading order: top-to-bottom (y descending), then
    // left-to-right (x ascending). This gives the natural LTR reading flow.
    blocks.sort_by(|a, b| {
        let y_cmp = b.bbox[1]
            .partial_cmp(&a.bbox[1])
            .unwrap_or(std::cmp::Ordering::Equal);
        if y_cmp != std::cmp::Ordering::Equal {
            return y_cmp;
        }
        a.bbox[0]
            .partial_cmp(&b.bbox[0])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    blocks
}

fn xy_cut_recursive(
    segments: &[TextSegment],
    config: &XyCutConfig,
    min_gap: f32,
    depth: usize,
    blocks: &mut Vec<XyCutBlock>,
) {
    if segments.is_empty() {
        return;
    }
    if segments.len() <= config.min_segments_per_block || depth >= config.max_depth {
        blocks.push(XyCutBlock::from_segments(segments, depth));
        return;
    }

    // Compute midpoints for projection.
    let mut x_mids: Vec<f32> = segments.iter().map(|s| s.x + s.width / 2.0).collect();
    let mut y_mids: Vec<f32> = segments.iter().map(|s| s.y + s.height / 2.0).collect();
    x_mids.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    y_mids.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Find largest gap in x-projection (vertical cut).
    let mut best_x_gap = 0.0f32;
    let mut best_x_pos = 0.0f32;
    for pair in x_mids.windows(2) {
        let gap = pair[1] - pair[0];
        if gap > best_x_gap {
            best_x_gap = gap;
            best_x_pos = (pair[0] + pair[1]) / 2.0;
        }
    }

    // Find largest gap in y-projection (horizontal cut).
    let mut best_y_gap = 0.0f32;
    let mut best_y_pos = 0.0f32;
    for pair in y_mids.windows(2) {
        let gap = pair[1] - pair[0];
        if gap > best_y_gap {
            best_y_gap = gap;
            best_y_pos = (pair[0] + pair[1]) / 2.0;
        }
    }

    // Choose the axis with the larger gap, provided it exceeds min_gap.
    if best_x_gap >= best_y_gap && best_x_gap >= min_gap {
        // Vertical cut: split into left and right groups.
        let (left, right): (Vec<TextSegment>, Vec<TextSegment>) = segments
            .iter()
            .cloned()
            .partition(|s| s.x + s.width / 2.0 < best_x_pos);
        if !left.is_empty() && !right.is_empty() {
            xy_cut_recursive(&left, config, min_gap, depth + 1, blocks);
            xy_cut_recursive(&right, config, min_gap, depth + 1, blocks);
            return;
        }
    } else if best_y_gap >= min_gap {
        // Horizontal cut: split into bottom and top groups.
        // Note: y increases downward, so "bottom" segments have larger y.
        let (bottom, top): (Vec<TextSegment>, Vec<TextSegment>) = segments
            .iter()
            .cloned()
            .partition(|s| s.y + s.height / 2.0 < best_y_pos);
        if !bottom.is_empty() && !top.is_empty() {
            xy_cut_recursive(&bottom, config, min_gap, depth + 1, blocks);
            xy_cut_recursive(&top, config, min_gap, depth + 1, blocks);
            return;
        }
    }

    // No viable cut found — this is a leaf block.
    blocks.push(XyCutBlock::from_segments(segments, depth));
}

/// Run XY-Cut++ on segments and return the resulting blocks.
/// Convenience wrapper with default XY-Cut++ config.
pub fn segment_blocks(segments: &[TextSegment]) -> Vec<XyCutBlock> {
    recursive_xy_cut_plusplus(segments, &XyCutPlusPlusConfig::default())
}

/// Classify a masked XY-Cut++ block into the best-fit LayoutRegionType.
/// Uses heuristics on segment content to determine if the mask is a
/// title, heading, figure, table, or general cross-layout element.
fn classify_mask_region_type(block: &XyCutBlock) -> LayoutRegionType {
    let text: String = block
        .segments
        .iter()
        .map(|s| s.text.as_str())
        .collect::<Vec<&str>>()
        .join(" ");

    // Check for formula patterns.
    let trimmed = text.trim();
    if trimmed.starts_with("$$")
        || trimmed.ends_with("$$")
        || trimmed.contains("\\begin{equation")
        || trimmed.contains("\\begin{align")
        || trimmed.contains("\\frac")
    {
        return LayoutRegionType::Formula;
    }

    let is_short = text.len() < 60;
    let first_char = text.chars().next();
    let is_cap = first_char.map(|c| c.is_uppercase()).unwrap_or(false);

    // Short capitalized text at top of page → Title.
    if is_short && is_cap && block.segments.len() <= 2 {
        let min_y = block.segments.iter().map(|s| s.y).fold(f32::MAX, f32::min);
        let max_y = block
            .segments
            .iter()
            .map(|s| s.y + s.height)
            .fold(f32::MIN, f32::max);
        if max_y - min_y < 30.0 {
            return LayoutRegionType::Title;
        }
        return LayoutRegionType::Heading;
    }

    // Check for list markers.
    let has_bullet =
        trimmed.starts_with("•") || trimmed.starts_with("- ") || trimmed.starts_with("* ");
    if has_bullet {
        return LayoutRegionType::ListItem;
    }

    // Digit-prefixed lines → numbered list or table.
    let has_numbered = trimmed.chars().next().map_or(false, |c| c.is_ascii_digit());
    if has_numbered && is_short {
        return LayoutRegionType::ListItem;
    }

    // Large blocks with many segments → figure or table.
    if block.segments.len() >= 6 {
        return LayoutRegionType::Table;
    }

    // Fallback: heading.
    if is_short {
        return LayoutRegionType::Heading;
    }

    LayoutRegionType::TextBlock
}

/// Top-level page layout analysis. Runs all heuristic detectors, recursive
/// X-Y cut segmentation, and produces a unified PageLayout with columns +
/// semantic regions.
pub fn analyze_page_layout(segments: &[TextSegment]) -> PageLayout {
    let (page_width, page_height) = page_bounds(segments);
    let layout = detect_columns(segments);

    let mut regions = Vec::new();

    // Run all semantic detectors.
    regions.extend(detect_headings_and_titles(segments, page_height));
    regions.extend(detect_header_footer(segments, page_height));
    regions.extend(detect_tables(segments));
    regions.extend(detect_formulas(segments, page_width));
    regions.extend(detect_list_items(segments));

    // Run XY-Cut++ segmentation (with pre-mask + density-aware splitting)
    // and add blocks as TextBlock regions. Masked blocks are tagged with their
    // type name for richer downstream consumption.
    let blocks = recursive_xy_cut_plusplus(segments, &XyCutPlusPlusConfig::default());
    for block in &blocks {
        // Only add blocks that span a reasonable area.
        let w = block.bbox[2] - block.bbox[0];
        let h = block.bbox[3] - block.bbox[1];
        if w > 10.0 && h > 10.0 {
            // Masks get richer type classification; normal blocks are TextBlock.
            let region_type = if block.is_mask {
                classify_mask_region_type(block)
            } else {
                LayoutRegionType::TextBlock
            };
            regions.push(LayoutRegion {
                bbox: block.bbox,
                region_type,
                confidence: if block.is_mask { 0.7 } else { 0.5 },
                text: block
                    .segments
                    .iter()
                    .map(|s| s.text.as_str())
                    .collect::<Vec<&str>>()
                    .join(" "),
            });
        }
    }

    PageLayout {
        page_width,
        page_height,
        columns: layout.columns,
        layout_type: layout.layout_type,
        regions,
    }
}

/// Generate a human-readable report of the page layout analysis.
pub fn layout_report(layout: &PageLayout) -> String {
    let mut report = String::new();
    report.push_str(&format!(
        "Page: {:.0}×{:.0} | {:?} ({} columns)\n",
        layout.page_width,
        layout.page_height,
        layout.layout_type,
        layout.columns.len()
    ));
    report.push_str(&format!("Regions detected: {}\n", layout.regions.len()));
    for r in &layout.regions {
        report.push_str(&format!(
            "  [{:>12}] conf={:.2} bbox=({:.0},{:.0},{:.0},{:.0}) text={:?}...\n",
            r.region_type.name(),
            r.confidence,
            r.bbox[0],
            r.bbox[1],
            r.bbox[2],
            r.bbox[3],
            &r.text[..r.text.len().min(40)],
        ));
    }
    report
}

/// Attempts to detect column layout from text segments on a page.
///
/// Algorithm:
/// 1. Collect all x-positions of text segments
/// 2. Cluster x-positions to detect column boundaries
/// 3. Assign each segment to a column based on its x position
/// 4. Sort within each column top-to-bottom
/// 5. Return columns in left-to-right order for correct read order
pub fn detect_columns(segments: &[TextSegment]) -> ColumnLayout {
    if segments.is_empty() {
        return ColumnLayout {
            column_count: 1,
            columns: vec![Column {
                index: 0,
                x_start: 0.0,
                x_end: 0.0,
                segments: vec![],
            }],
            layout_type: LayoutType::Unknown,
        };
    }

    let (page_width, _) = page_bounds(segments);

    let gap_threshold = page_width / 4.0;

    // Extract x-centers of all segments
    let x_centers: Vec<f32> = segments.iter().map(|s| s.x + s.width / 2.0).collect();

    // Sort unique x-centers
    let mut sorted_x: Vec<f32> = x_centers;
    sorted_x.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    sorted_x.dedup_by(|a, b| (*a - *b).abs() < 10.0);

    // Cluster by x-position gaps
    let mut clusters: Vec<Vec<f32>> = Vec::new();
    for &x in &sorted_x {
        if let Some(last_cluster) = clusters.last_mut() {
            let last_x = *last_cluster.last().unwrap_or(&x);
            if (x - last_x) < gap_threshold {
                last_cluster.push(x);
                continue;
            }
        }
        clusters.push(vec![x]);
    }

    let column_count = clusters.len();
    let layout_type = match column_count {
        1 => LayoutType::SingleColumn,
        2 => LayoutType::TwoColumn,
        3 => LayoutType::ThreeColumn,
        _ if column_count > 3 => LayoutType::Mixed,
        _ => LayoutType::Unknown,
    };

    // Assign segments to columns
    let mut columns: Vec<Column> = clusters
        .iter()
        .enumerate()
        .map(|(idx, x_positions)| {
            let x_start = x_positions.first().copied().unwrap_or(0.0) - 20.0;
            let x_end = x_positions.last().copied().unwrap_or(0.0) + 20.0;

            let mut segs: Vec<TextSegment> = segments
                .iter()
                .filter(|s| {
                    let center = s.x + s.width / 2.0;
                    center >= x_start && center <= x_end
                })
                .cloned()
                .collect();

            // Sort top-to-bottom within column
            segs.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));

            Column {
                index: idx,
                x_start,
                x_end,
                segments: segs,
            }
        })
        .collect();

    // Sort columns left-to-right
    columns.sort_by(|a, b| {
        a.x_start
            .partial_cmp(&b.x_start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ColumnLayout {
        column_count,
        columns,
        layout_type,
    }
}

/// Reconstruct text in correct read order from a column layout.
/// Returns (corrected_text, column_layout_report).
pub fn reconstruct_read_order(layout: &ColumnLayout) -> (String, String) {
    let mut text = String::new();
    let mut report = String::new();

    report.push_str(&format!(
        "Layout: {:?} ({} columns)\n",
        layout.layout_type, layout.column_count
    ));

    for (i, col) in layout.columns.iter().enumerate() {
        text.push_str(&format!("[Column {}]\n", i + 1));
        for seg in &col.segments {
            text.push_str(&seg.text);
            if !seg.text.ends_with('\n') {
                text.push(' ');
            }
        }
        text.push_str("\n\n");
    }

    (text.trim().to_string(), report)
}

/// Parse text segments from a raw PDF text dump.
/// This is a heuristic: assumes lines are separated by newlines,
/// attempts to detect x-position from indentation/whitespace.
/// Convenience: detect columns and reconstruct read order in one call.
pub fn correct_read_order(text: &str, page_num: usize) -> (String, String) {
    let segments = segments_from_text(text, page_num);
    let layout = detect_columns(&segments);
    reconstruct_read_order(&layout)
}

pub fn segments_from_text(text: &str, page_num: usize) -> Vec<TextSegment> {
    let mut segments = Vec::new();
    let mut y: f32 = 800.0; // start from top of page
    let line_height: f32 = 14.0;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            y -= line_height * 0.5;
            continue;
        }
        // Estimate x position from leading whitespace
        let leading_spaces = line.len() - trimmed.len();
        let x = leading_spaces as f32 * 8.0; // ~8px per space
        let width = trimmed.len().min(100) as f32 * 8.0;

        segments.push(TextSegment {
            x,
            y,
            width,
            height: line_height,
            text: trimmed.to_string(),
            page_num,
        });

        y -= line_height;
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_column() {
        let segs = vec![
            TextSegment {
                x: 50.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "Title".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 680.0,
                width: 400.0,
                height: 14.0,
                text: "Normal paragraph text here.".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 660.0,
                width: 300.0,
                height: 14.0,
                text: "More content.".into(),
                page_num: 1,
            },
        ];
        let layout = detect_columns(&segs);
        assert_eq!(layout.column_count, 1);
        assert_eq!(layout.layout_type, LayoutType::SingleColumn);
    }

    #[test]
    fn test_two_column() {
        let segs = vec![
            // Left column
            TextSegment {
                x: 30.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "Left col 1".into(),
                page_num: 1,
            },
            TextSegment {
                x: 30.0,
                y: 680.0,
                width: 200.0,
                height: 14.0,
                text: "Left col 2".into(),
                page_num: 1,
            },
            // Right column
            TextSegment {
                x: 350.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "Right col 1".into(),
                page_num: 1,
            },
            TextSegment {
                x: 350.0,
                y: 680.0,
                width: 200.0,
                height: 14.0,
                text: "Right col 2".into(),
                page_num: 1,
            },
        ];
        let layout = detect_columns(&segs);
        assert_eq!(layout.column_count, 2);
        assert_eq!(layout.layout_type, LayoutType::TwoColumn);
        let (text, _) = reconstruct_read_order(&layout);
        assert!(text.contains("Left col"));
        assert!(text.contains("Right col"));
        assert!(text.contains("[Column 1]"));
        assert!(text.contains("[Column 2]"));
    }

    #[test]
    fn test_empty() {
        let layout = detect_columns(&[]);
        assert_eq!(layout.column_count, 1);
        assert_eq!(layout.layout_type, LayoutType::Unknown);
    }

    #[test]
    fn test_reconstruct_empty() {
        let layout = ColumnLayout {
            column_count: 1,
            columns: vec![Column {
                index: 0,
                x_start: 0.0,
                x_end: 100.0,
                segments: vec![],
            }],
            layout_type: LayoutType::SingleColumn,
        };
        let (text, _) = reconstruct_read_order(&layout);
        assert!(text.is_empty());
    }

    #[test]
    fn test_segments_from_text() {
        let text = "Hello\n  Indented\nWorld";
        let segs = segments_from_text(text, 1);
        assert_eq!(segs.len(), 3);
        // Indented line should have larger x
        assert!(segs[1].x > segs[0].x);
    }

    #[test]
    fn test_correct_read_order_single() {
        let (text, report) = correct_read_order("Line 1\nLine 2\nLine 3", 1);
        assert!(text.contains("Line 1"));
        assert!(text.contains("Line 2"));
        assert!(report.contains("SingleColumn"));
    }

    // --- Heuristic layout tests ---

    #[test]
    fn test_analyze_page_layout_empty() {
        let layout = analyze_page_layout(&[]);
        assert_eq!(layout.regions.len(), 0);
        assert_eq!(layout.layout_type, LayoutType::Unknown);
    }

    #[test]
    fn test_analyze_page_layout_single_column() {
        let segs = vec![
            TextSegment {
                x: 50.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "Normal text here.".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 680.0,
                width: 180.0,
                height: 14.0,
                text: "More content.".into(),
                page_num: 1,
            },
        ];
        let layout = analyze_page_layout(&segs);
        assert_eq!(layout.layout_type, LayoutType::SingleColumn);
    }

    #[test]
    fn test_detect_heading_capitalized_with_gap() {
        let segs = vec![
            TextSegment {
                x: 50.0,
                y: 750.0,
                width: 200.0,
                height: 14.0,
                text: "Some preceding text.".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 700.0,
                width: 100.0,
                height: 16.0,
                text: "Introduction".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 670.0,
                width: 300.0,
                height: 14.0,
                text: "This is the body text after heading.".into(),
                page_num: 1,
            },
        ];
        let regions = detect_headings_and_titles(&segs, 800.0);
        let heading: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == LayoutRegionType::Heading)
            .collect();
        assert!(!heading.is_empty(), "should detect at least one heading");
        assert!(heading[0].text.contains("Introduction"));
    }

    #[test]
    fn test_detect_title_near_top() {
        let segs = vec![
            TextSegment {
                x: 100.0,
                y: 780.0,
                width: 300.0,
                height: 20.0,
                text: "My Research Paper Title".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 730.0,
                width: 400.0,
                height: 14.0,
                text: "Author Name".into(),
                page_num: 1,
            },
        ];
        let regions = detect_headings_and_titles(&segs, 800.0);
        let title: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == LayoutRegionType::Title)
            .collect();
        assert!(!title.is_empty());
        assert!(title[0].text.contains("Research Paper"));
    }

    #[test]
    fn test_detect_table_three_rows() {
        let segs = vec![
            TextSegment {
                x: 30.0,
                y: 700.0,
                width: 50.0,
                height: 14.0,
                text: "Name".into(),
                page_num: 1,
            },
            TextSegment {
                x: 150.0,
                y: 700.0,
                width: 50.0,
                height: 14.0,
                text: "Age".into(),
                page_num: 1,
            },
            TextSegment {
                x: 30.0,
                y: 680.0,
                width: 50.0,
                height: 14.0,
                text: "Alice".into(),
                page_num: 1,
            },
            TextSegment {
                x: 150.0,
                y: 680.0,
                width: 30.0,
                height: 14.0,
                text: "30".into(),
                page_num: 1,
            },
            TextSegment {
                x: 30.0,
                y: 660.0,
                width: 50.0,
                height: 14.0,
                text: "Bob".into(),
                page_num: 1,
            },
            TextSegment {
                x: 150.0,
                y: 660.0,
                width: 30.0,
                height: 14.0,
                text: "25".into(),
                page_num: 1,
            },
        ];
        let regions = detect_tables(&segs);
        assert!(!regions.is_empty(), "should detect table with 3+ rows");
        assert_eq!(regions[0].region_type, LayoutRegionType::Table);
    }

    #[test]
    fn test_detect_header_footer() {
        let mut segs = Vec::new();
        // 10 segments at header y=760 (near top of 800px page)
        for i in 0..10 {
            segs.push(TextSegment {
                x: 50.0,
                y: 760.0,
                width: 100.0,
                height: 14.0,
                text: format!("Header line {}", i),
                page_num: 1,
            });
        }
        // Body text
        segs.push(TextSegment {
            x: 50.0,
            y: 500.0,
            width: 200.0,
            height: 14.0,
            text: "Body".into(),
            page_num: 1,
        });
        // 10 segments at footer y=40 (near bottom)
        for i in 0..10 {
            segs.push(TextSegment {
                x: 50.0,
                y: 40.0,
                width: 100.0,
                height: 14.0,
                text: format!("Footer line {}", i),
                page_num: 1,
            });
        }
        let regions = detect_header_footer(&segs, 800.0);
        let headers: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == LayoutRegionType::PageHeader)
            .collect();
        let footers: Vec<_> = regions
            .iter()
            .filter(|r| r.region_type == LayoutRegionType::PageFooter)
            .collect();
        assert!(!headers.is_empty(), "should detect page header");
        assert!(!footers.is_empty(), "should detect page footer");
    }

    #[test]
    fn test_detect_formula_latex() {
        let segs = vec![
            TextSegment {
                x: 100.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: r"$\alpha + \beta = \gamma$".into(),
                page_num: 1,
            },
            TextSegment {
                x: 100.0,
                y: 680.0,
                width: 300.0,
                height: 14.0,
                text: r"\begin{equation} E = mc^2 \end{equation}".into(),
                page_num: 1,
            },
        ];
        let regions = detect_formulas(&segs, 600.0);
        assert_eq!(regions.len(), 2, "both lines are LaTeX formulas");
        for r in &regions {
            assert_eq!(r.region_type, LayoutRegionType::Formula);
        }
    }

    #[test]
    fn test_detect_list_items_bullets() {
        let segs = vec![
            TextSegment {
                x: 60.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "• First item".into(),
                page_num: 1,
            },
            TextSegment {
                x: 60.0,
                y: 680.0,
                width: 250.0,
                height: 14.0,
                text: "• Second item".into(),
                page_num: 1,
            },
            TextSegment {
                x: 60.0,
                y: 660.0,
                width: 180.0,
                height: 14.0,
                text: "• Third item".into(),
                page_num: 1,
            },
        ];
        let regions = detect_list_items(&segs);
        assert_eq!(
            regions.len(),
            1,
            "should group 3 list items into one region"
        );
        assert_eq!(regions[0].region_type, LayoutRegionType::ListItem);
    }

    #[test]
    fn test_detect_list_items_numbered() {
        let segs = vec![
            TextSegment {
                x: 50.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "1. First step".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 680.0,
                width: 200.0,
                height: 14.0,
                text: "2. Second step".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 660.0,
                width: 200.0,
                height: 14.0,
                text: "3. Third step".into(),
                page_num: 1,
            },
        ];
        let regions = detect_list_items(&segs);
        assert_eq!(regions.len(), 1);
    }

    #[test]
    fn test_layout_region_type_name() {
        assert_eq!(LayoutRegionType::TextBlock.name(), "text");
        assert_eq!(LayoutRegionType::Heading.name(), "heading");
        assert_eq!(LayoutRegionType::Table.name(), "table");
        assert_eq!(LayoutRegionType::Figure.name(), "figure");
        assert_eq!(LayoutRegionType::Formula.name(), "formula");
        assert_eq!(LayoutRegionType::PageHeader.name(), "page-header");
        assert_eq!(LayoutRegionType::PageFooter.name(), "page-footer");
        assert_eq!(LayoutRegionType::ListItem.name(), "list-item");
    }

    #[test]
    fn test_layout_report_format() {
        let segs = vec![
            TextSegment {
                x: 50.0,
                y: 750.0,
                width: 200.0,
                height: 14.0,
                text: "Header line".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 700.0,
                width: 300.0,
                height: 14.0,
                text: "Normal text paragraph.".into(),
                page_num: 1,
            },
        ];
        let layout = analyze_page_layout(&segs);
        let report = layout_report(&layout);
        assert!(report.contains("columns"));
        assert!(report.contains("Regions detected"));
    }

    // --- XY-Cut tests ---

    #[test]
    fn test_xy_cut_single_block() {
        let segs = vec![
            TextSegment {
                x: 50.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "Line 1".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 680.0,
                width: 200.0,
                height: 14.0,
                text: "Line 2".into(),
                page_num: 1,
            },
            TextSegment {
                x: 50.0,
                y: 660.0,
                width: 200.0,
                height: 14.0,
                text: "Line 3".into(),
                page_num: 1,
            },
        ];
        let config = XyCutConfig {
            min_gap_ratio: 3.0,
            ..Default::default()
        };
        let blocks = recursive_xy_cut(&segs, &config);
        // Single column of tight lines → no cut → 1 block
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].segments.len(), 3);
    }

    #[test]
    fn test_xy_cut_two_columns() {
        let segs = vec![
            // Left column
            TextSegment {
                x: 30.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "Left A".into(),
                page_num: 1,
            },
            TextSegment {
                x: 30.0,
                y: 680.0,
                width: 200.0,
                height: 14.0,
                text: "Left B".into(),
                page_num: 1,
            },
            // Right column — big x gap from left
            TextSegment {
                x: 400.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "Right A".into(),
                page_num: 1,
            },
            TextSegment {
                x: 400.0,
                y: 680.0,
                width: 200.0,
                height: 14.0,
                text: "Right B".into(),
                page_num: 1,
            },
        ];
        let config = XyCutConfig {
            min_gap_ratio: 1.5,
            ..Default::default()
        };
        let blocks = recursive_xy_cut(&segs, &config);
        // Two columns with gap 400-230=170 should split
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_xy_cut_vertical_split() {
        // Title at top, body below — should get a horizontal cut.
        let segs = vec![
            TextSegment {
                x: 100.0,
                y: 780.0,
                width: 300.0,
                height: 20.0,
                text: "Title".into(),
                page_num: 1,
            },
            TextSegment {
                x: 100.0,
                y: 400.0,
                width: 400.0,
                height: 14.0,
                text: "Body text here".into(),
                page_num: 1,
            },
            TextSegment {
                x: 110.0,
                y: 380.0,
                width: 400.0,
                height: 14.0,
                text: "More body".into(),
                page_num: 1,
            },
        ];
        let config = XyCutConfig {
            min_gap_ratio: 1.0,
            ..Default::default()
        };
        let blocks = recursive_xy_cut(&segs, &config);
        // Large y gap (780-414=366) should split title from body
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn test_xy_cut_empty() {
        let blocks = recursive_xy_cut(&[], &XyCutConfig::default());
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_xy_cut_single_segment() {
        let segs = vec![TextSegment {
            x: 50.0,
            y: 700.0,
            width: 200.0,
            height: 14.0,
            text: "Solo".into(),
            page_num: 1,
        }];
        let blocks = recursive_xy_cut(&segs, &XyCutConfig::default());
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].segments.len(), 1);
    }

    #[test]
    fn test_segment_blocks_convenience() {
        let segs = vec![
            TextSegment {
                x: 50.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "A".into(),
                page_num: 1,
            },
            TextSegment {
                x: 400.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "B".into(),
                page_num: 1,
            },
        ];
        let blocks = segment_blocks(&segs);
        assert!(!blocks.is_empty());
    }

    #[test]
    fn test_xy_cut_in_analyze_page_layout() {
        // Two-column layout: the analyze_page_layout should produce
        // at least the column blocks as TextBlock regions.
        let segs = vec![
            TextSegment {
                x: 30.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "Left".into(),
                page_num: 1,
            },
            TextSegment {
                x: 400.0,
                y: 700.0,
                width: 200.0,
                height: 14.0,
                text: "Right".into(),
                page_num: 1,
            },
        ];
        let layout = analyze_page_layout(&segs);
        let text_blocks: Vec<_> = layout
            .regions
            .iter()
            .filter(|r| r.region_type == LayoutRegionType::TextBlock)
            .collect();
        assert!(
            text_blocks.len() >= 2,
            "should produce xy-cut text blocks for 2-column layout"
        );
    }

    #[test]
    fn test_xy_cut_max_depth() {
        // Many segments with small gaps should be limited by max_depth.
        let mut segs = Vec::new();
        for i in 0..50 {
            segs.push(TextSegment {
                x: 50.0,
                y: 800.0 - i as f32 * 15.0,
                width: 200.0,
                height: 14.0,
                text: format!("Line {}", i),
                page_num: 1,
            });
        }
        let config = XyCutConfig {
            max_depth: 2,
            min_gap_ratio: 0.5,
            ..Default::default()
        };
        let blocks = recursive_xy_cut(&segs, &config);
        // With max_depth=2 we should get few blocks (cuts at top levels only).
        assert!(blocks.len() < segs.len());
        assert!(blocks.len() >= 1);
    }

    #[test]
    fn test_xy_cut_block_bbox() {
        let segs = vec![
            TextSegment {
                x: 50.0,
                y: 700.0,
                width: 100.0,
                height: 14.0,
                text: "A".into(),
                page_num: 1,
            },
            TextSegment {
                x: 60.0,
                y: 600.0,
                width: 100.0,
                height: 14.0,
                text: "B".into(),
                page_num: 1,
            },
        ];
        let blocks = recursive_xy_cut(&segs, &XyCutConfig::default());
        assert_eq!(blocks.len(), 1);
        let bbox = &blocks[0].bbox;
        assert!(bbox[0] <= 50.0); // min x
        assert!(bbox[2] >= 160.0); // max x (60+100)
    }
}

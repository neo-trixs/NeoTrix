use super::types::{
    DiffHunk, DiffLineType, RelocationRequest, ReviewComment, ReviewFileDiff, RELOCATION_THRESHOLD,
};

#[derive(Debug)]
pub struct CommentResolver;

impl CommentResolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_comments(&self, comments: &mut [ReviewComment], diffs: &[ReviewFileDiff]) {
        for comment in comments.iter_mut() {
            if comment.start_line.is_some()
                && comment.end_line.is_some()
                && comment.match_confidence >= RELOCATION_THRESHOLD
            {
                continue;
            }
            if let Some(diff) = diffs.iter().find(|d| d.file == comment.file) {
                self.resolve_single(comment, diff);
            }
        }
    }

    pub fn resolve_single(&self, comment: &mut ReviewComment, diff: &ReviewFileDiff) {
        let search = comment.existing_code.trim();
        if search.is_empty() {
            comment.match_confidence = 0.0;
            return;
        }
        let search_normalized = self.normalize(search);

        for hunk in &diff.hunks {
            let target_lines = self.build_target_lines(hunk);
            let search_lines: Vec<&str> = search_normalized.lines().collect();
            if search_lines.is_empty() {
                continue;
            }

            if let Some((match_new_start, match_new_end)) =
                self.sliding_window_match(&search_lines, &target_lines)
            {
                comment.start_line = Some(match_new_start);
                comment.end_line = Some(match_new_end);
                comment.match_confidence = self.compute_confidence(
                    &search_lines,
                    &target_lines,
                    match_new_start,
                    match_new_end,
                );
                comment.anchor_lines = self.extract_anchors(diff, match_new_start, match_new_end);
                comment.needs_relocation = comment.match_confidence < RELOCATION_THRESHOLD;
                return;
            }

            if let Some((confidence, lcs_start, lcs_end)) =
                self.lcs_match(&search_lines, &target_lines)
            {
                comment.start_line = Some(lcs_start);
                comment.end_line = Some(lcs_end);
                comment.match_confidence = confidence;
                comment.anchor_lines = self.extract_anchors(diff, lcs_start, lcs_end);
                comment.needs_relocation = confidence < RELOCATION_THRESHOLD;
                return;
            }
        }

        comment.match_confidence = 0.0;
        comment.needs_relocation = true;
    }

    /// Resolve a single comment and return a relocation request if confidence is too low.
    pub fn resolve_single_with_relocation(
        &self,
        comment: &mut ReviewComment,
        diff: &ReviewFileDiff,
    ) -> Option<RelocationRequest> {
        self.resolve_single(comment, diff);
        if comment.match_confidence < RELOCATION_THRESHOLD && comment.start_line.is_none() {
            Some(RelocationRequest {
                comment_id: comment.id.clone(),
                file: comment.file.clone(),
                existing_code: comment.existing_code.clone(),
                message: comment.message.clone(),
                current_confidence: comment.match_confidence,
            })
        } else {
            None
        }
    }

    /// Re-run resolution for re-review; resets stale anchors/confidence.
    pub fn relocate_comment(&self, comment: &mut ReviewComment, diff: &ReviewFileDiff) {
        comment.anchor_lines.clear();
        comment.match_confidence = 0.0;
        comment.start_line = None;
        comment.end_line = None;
        self.resolve_single(comment, diff);
    }

    /// Drift detection: check if anchor lines still exist in the current diff.
    pub fn check_anchors(&self, anchors: &[String], diff: &ReviewFileDiff) -> bool {
        if anchors.is_empty() {
            return false;
        }
        let mut match_count = 0;
        for anchor in anchors {
            let normalized_anchor = self.normalize(anchor);
            for hunk in &diff.hunks {
                let found = hunk.lines.iter().any(|dl| {
                    if dl.line_type == DiffLineType::Deletion {
                        return false;
                    }
                    let normalized_line = self.normalize(&dl.content);
                    normalized_line.contains(&normalized_anchor)
                        || normalized_anchor.contains(&normalized_line)
                });
                if found {
                    match_count += 1;
                    break;
                }
            }
        }
        match_count >= (anchors.len() / 2).max(1)
    }

    // ── Internal helpers ──

    fn build_target_lines(&self, hunk: &DiffHunk) -> Vec<(u32, u32, String)> {
        let mut target_lines = Vec::new();
        let mut new_line = hunk.new_start;
        for dl in &hunk.lines {
            match dl.line_type {
                DiffLineType::Addition | DiffLineType::Context => {
                    let normalized = self.normalize(&dl.content);
                    target_lines.push((dl.old_line.unwrap_or(0), new_line, normalized));
                    new_line += 1;
                }
                DiffLineType::Deletion => {
                    target_lines.push((dl.old_line.unwrap_or(0), 0, self.normalize(&dl.content)));
                }
            }
        }
        target_lines
    }

    fn normalize(&self, s: &str) -> String {
        s.trim().replace("\r\n", "\n").replace('\t', "    ")
    }

    fn sliding_window_match(
        &self,
        search_lines: &[&str],
        target: &[(u32, u32, String)],
    ) -> Option<(u32, u32)> {
        if search_lines.len() > target.len() {
            return None;
        }
        if search_lines.len() == 1 {
            let s = search_lines[0].trim();
            for (_, new_line, content) in target {
                if content.contains(s) || content.trim() == s {
                    return Some((*new_line, *new_line));
                }
            }
            return self.fuzzy_match_single(search_lines[0], target);
        }
        for window_start in 0..=(target.len() - search_lines.len()) {
            let mut match_count = 0;
            let required = search_lines.len();

            for (i, search_line) in search_lines.iter().enumerate() {
                let (_, _, target_line) = &target[window_start + i];
                if target_line.trim() == search_line.trim()
                    || target_line.contains(search_line.trim())
                {
                    match_count += 1;
                } else {
                    let s_clean: String = search_line
                        .trim()
                        .chars()
                        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                        .collect();
                    let t_clean: String = target_line
                        .trim()
                        .chars()
                        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                        .collect();
                    if s_clean == t_clean {
                        match_count += 1;
                    }
                }
            }

            if match_count >= required.saturating_sub(1) {
                let start = target[window_start].1;
                let end = target[window_start + search_lines.len() - 1].1;
                let end = if end == 0 { start } else { end };
                return Some((start, end));
            }
        }
        None
    }

    fn fuzzy_match_single(
        &self,
        search: &str,
        target: &[(u32, u32, String)],
    ) -> Option<(u32, u32)> {
        let s = search.trim();
        let s_alpha: String = s.chars().filter(|c| c.is_alphanumeric()).collect();
        if s_alpha.len() < 3 {
            return None;
        }
        for (_, new_line, content) in target {
            let t_alpha: String = content.chars().filter(|c| c.is_alphanumeric()).collect();
            if t_alpha.contains(&s_alpha) || s_alpha.contains(&t_alpha) {
                return Some((*new_line, *new_line));
            }
        }
        None
    }

    /// Compute confidence for a sliding-window match (fraction of exact line matches).
    fn compute_confidence(
        &self,
        search_lines: &[&str],
        target: &[(u32, u32, String)],
        match_start: u32,
        match_end: u32,
    ) -> f32 {
        let start_idx = target.iter().position(|(_, nl, _)| *nl == match_start);
        let end_idx = target.iter().rposition(|(_, nl, _)| *nl == match_end);

        let (s_idx, e_idx) = match (start_idx, end_idx) {
            (Some(s), Some(e)) if s <= e => (s, e),
            _ => return 0.5,
        };

        let target_slice = &target[s_idx..=e_idx];
        let exact_matches = search_lines
            .iter()
            .filter(|s| target_slice.iter().any(|(_, _, t)| t.trim() == s.trim()))
            .count();

        let confidence = exact_matches as f32 / search_lines.len() as f32;
        confidence.clamp(0.0, 1.0)
    }

    /// LCS-based matching as fallback when sliding window fails.
    fn lcs_match(
        &self,
        search_lines: &[&str],
        target: &[(u32, u32, String)],
    ) -> Option<(f32, u32, u32)> {
        if search_lines.is_empty() || target.is_empty() {
            return None;
        }

        let n = search_lines.len();
        let m = target.len();

        let mut dp = vec![vec![0usize; m + 1]; n + 1];
        for i in 1..=n {
            for j in 1..=m {
                let s_norm = search_lines[i - 1].trim();
                let t_norm = target[j - 1].2.trim();
                if s_norm == t_norm || target[j - 1].2.contains(s_norm) {
                    dp[i][j] = dp[i - 1][j - 1] + 1;
                } else {
                    dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
                }
            }
        }

        let lcs_len = dp[n][m];
        if lcs_len == 0 {
            return None;
        }

        let mut i = n;
        let mut j = m;
        let mut matched_indices: Vec<usize> = Vec::new();

        while i > 0 && j > 0 {
            let s_norm = search_lines[i - 1].trim();
            let t_norm = target[j - 1].2.trim();
            if s_norm == t_norm || target[j - 1].2.contains(s_norm) {
                matched_indices.push(j - 1);
                i -= 1;
                j -= 1;
            } else if dp[i - 1][j] >= dp[i][j - 1] {
                i -= 1;
            } else {
                j -= 1;
            }
        }

        matched_indices.reverse();
        if matched_indices.is_empty() {
            return None;
        }

        let start_idx = matched_indices[0];
        let end_idx = matched_indices[matched_indices.len() - 1];
        let start_line = target[start_idx].1;
        let end_line = target[end_idx].1;
        let end_line = if end_line == 0 { start_line } else { end_line };

        let matched_range = end_idx - start_idx + 1;
        let denominator = n.max(matched_range) as f32;
        let confidence = (lcs_len as f32 / denominator).min(1.0);

        Some((confidence, start_line, end_line))
    }

    /// Extract 3–5 anchor lines from context near the matched position.
    fn extract_anchors(
        &self,
        diff: &ReviewFileDiff,
        start_line: u32,
        end_line: u32,
    ) -> Vec<String> {
        let mut anchors = Vec::new();
        let margin = 5;

        let all_lines: Vec<&str> = diff
            .hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| matches!(l.line_type, DiffLineType::Context | DiffLineType::Addition))
            .filter_map(|l| {
                l.new_line
                    .filter(|&nl| {
                        nl >= start_line.saturating_sub(margin) && nl <= end_line + margin
                    })
                    .map(|_| l.content.trim())
            })
            .filter(|s| !s.is_empty() && s.len() > 3)
            .collect();

        for line in &all_lines {
            let trimmed = line.trim().to_string();
            if !anchors.contains(&trimmed) {
                anchors.push(trimmed);
                if anchors.len() >= 5 {
                    break;
                }
            }
        }

        if anchors.len() < 3 {
            let wider: Vec<&str> = diff
                .hunks
                .iter()
                .flat_map(|h| h.lines.iter())
                .filter(|l| matches!(l.line_type, DiffLineType::Context | DiffLineType::Addition))
                .filter_map(|l| {
                    l.new_line
                        .filter(|&nl| {
                            nl >= start_line.saturating_sub(margin * 2)
                                && nl <= end_line + margin * 2
                        })
                        .map(|_| l.content.as_str())
                })
                .filter(|s| !s.trim().is_empty() && s.trim().len() > 3)
                .collect();

            for line in wider {
                let trimmed = line.trim().to_string();
                if !anchors.contains(&trimmed) {
                    anchors.push(trimmed);
                    if anchors.len() >= 5 {
                        break;
                    }
                }
            }
        }

        anchors
    }

    pub fn normalize_code(&self, code: &str) -> String {
        let mut result = String::with_capacity(code.len());
        let mut prev_was_newline = false;
        for ch in code.chars() {
            if ch == '\n' {
                if prev_was_newline {
                    continue;
                }
                prev_was_newline = true;
            } else {
                prev_was_newline = false;
            }
            result.push(ch);
        }
        result.trim().to_string()
    }
}

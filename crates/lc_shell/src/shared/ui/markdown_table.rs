//! Custom GFM pipe-table parser and renderer for shared Markdown documents.
//!
//! Renders tables with explicit vertical column borders, horizontal row dividers,
//! alternating subtle stripes, and generous cell padding so columns never
//! look cramped.

use egui::{
    text::LayoutJob, vec2, Color32, CornerRadius, FontFamily, FontId, Pos2, Rect, Sense, Stroke,
    StrokeKind, TextFormat, Ui,
};
use lc_canvas::theme::Tokens;

const CELL_PAD_X: f32 = 12.0;
const CELL_PAD_Y: f32 = 7.0;
const MIN_COL_W: f32 = 56.0;

#[derive(Debug, PartialEq)]
pub(crate) struct ParsedTable {
    pub(crate) header: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum DocBlock<'a> {
    Markdown(&'a str),
    Table(ParsedTable),
}

/// Splits markdown into non-table slices and parsed GFM pipe-table blocks.
pub(crate) fn split_markdown_tables(md: &str) -> Vec<DocBlock<'_>> {
    let lines = collect_line_spans(md);
    let mut blocks = Vec::new();
    let mut cursor = 0usize;
    let mut i = 0usize;
    let mut in_fence = false;

    while i < lines.len() {
        let (start, _, line) = lines[i];
        let trimmed = line.trim();

        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            i += 1;
            continue;
        }

        if !in_fence && i + 1 < lines.len() {
            let next_line = lines[i + 1].2;
            if is_pipe_row(line) && is_separator_row(next_line) {
                if start > cursor {
                    let slice = &md[cursor..start];
                    if !slice.trim().is_empty() {
                        blocks.push(DocBlock::Markdown(slice));
                    }
                }

                let header = split_pipe_cells(line);
                let ncols = header.len().max(1);
                let mut rows = Vec::new();
                let mut j = i + 2;
                let mut table_end = lines[i + 1].1;

                while j < lines.len() && is_pipe_row(lines[j].2) {
                    let mut cells = split_pipe_cells(lines[j].2);
                    cells.resize(ncols, String::new());
                    rows.push(cells);
                    table_end = lines[j].1;
                    j += 1;
                }

                blocks.push(DocBlock::Table(ParsedTable { header, rows }));
                cursor = table_end;
                i = j;
                continue;
            }
        }

        i += 1;
    }

    if cursor < md.len() {
        let tail = &md[cursor..];
        if !tail.trim().is_empty() {
            blocks.push(DocBlock::Markdown(tail));
        }
    }

    blocks
}

fn collect_line_spans(text: &str) -> Vec<(usize, usize, &str)> {
    let mut out = Vec::new();
    let mut offset = 0usize;
    for chunk in text.split_inclusive('\n') {
        let end = offset + chunk.len();
        let line = chunk.trim_end_matches(['\r', '\n']);
        out.push((offset, end, line));
        offset = end;
    }
    if !text.is_empty() && !text.ends_with('\n') && offset < text.len() {
        out.push((offset, text.len(), &text[offset..]));
    }
    out
}

fn is_pipe_row(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('|') && trimmed.contains('|') && trimmed.len() > 1
}

fn is_separator_row(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return false;
    }
    let cells = split_pipe_cells(trimmed);
    !cells.is_empty()
        && cells.iter().all(|c| {
            let s = c.trim();
            !s.is_empty()
                && s.contains('-')
                && s.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')
        })
}

fn split_pipe_cells(line: &str) -> Vec<String> {
    let mut s = line.trim();
    if let Some(rest) = s.strip_prefix('|') {
        s = rest;
    }
    if let Some(rest) = s.strip_suffix('|') {
        s = rest;
    }
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek() == Some(&'|') {
            cur.push('|');
            chars.next();
        } else if ch == '|' {
            cells.push(cur.trim().to_string());
            cur.clear();
        } else {
            cur.push(ch);
        }
    }
    cells.push(cur.trim().to_string());
    cells
}

/// Renders a Markdown table with vertical column dividers, horizontal row rules, and padding.
pub(crate) fn render_markdown_table(ui: &mut Ui, t: &Tokens, table: &ParsedTable) {
    let ncols = table.header.len();
    if ncols == 0 {
        return;
    }

    // 1. Measure natural unwrapped width per column
    let mut natural_widths = vec![MIN_COL_W; ncols];
    for (c, cell) in table.header.iter().enumerate() {
        let job = build_cell_layout_job(cell, t, true, f32::INFINITY);
        let galley = ui.painter().layout_job(job);
        natural_widths[c] = natural_widths[c].max(galley.size().x + CELL_PAD_X * 2.0);
    }
    for row in &table.rows {
        for (c, cell) in row.iter().enumerate().take(ncols) {
            let job = build_cell_layout_job(cell, t, false, f32::INFINITY);
            let galley = ui.painter().layout_job(job);
            natural_widths[c] = natural_widths[c].max(galley.size().x + CELL_PAD_X * 2.0);
        }
    }

    // 2. Fit column widths into available width, wrapping widest columns first if needed
    let avail_w = ui.available_width().max(MIN_COL_W * ncols as f32);
    let col_widths = fit_column_widths(&natural_widths, avail_w);
    let total_w: f32 = col_widths.iter().sum();

    // 3. Layout galleys at final wrapped column widths
    let layout_row =
        |cells: &[String], is_header: bool| -> (Vec<std::sync::Arc<egui::Galley>>, f32) {
            let mut galleys = Vec::with_capacity(ncols);
            let mut max_h = 0.0f32;
            for (c, &col_w) in col_widths.iter().enumerate().take(ncols) {
                let txt = cells.get(c).map(String::as_str).unwrap_or("");
                let wrap_w = (col_w - CELL_PAD_X * 2.0).max(16.0);
                let galley = ui
                    .painter()
                    .layout_job(build_cell_layout_job(txt, t, is_header, wrap_w));
                max_h = max_h.max(galley.size().y);
                galleys.push(galley);
            }
            (galleys, (max_h + CELL_PAD_Y * 2.0).ceil())
        };

    let (header_galleys, header_h) = layout_row(&table.header, true);
    let mut body_rows = Vec::with_capacity(table.rows.len());
    let mut body_h = 0.0f32;
    for row in &table.rows {
        let (galleys, h) = layout_row(row, false);
        body_h += h;
        body_rows.push((galleys, h));
    }

    let total_h = header_h + body_h;
    let (table_rect, _) = ui.allocate_exact_size(vec2(total_w, total_h), Sense::hover());
    let painter = ui.painter();

    // Outer table fill
    painter.rect_filled(table_rect, CornerRadius::ZERO, t.bg);

    // Header row background & divider
    let header_fill = if t.dark {
        Color32::from_rgb(28, 31, 38)
    } else {
        t.sunken
    };
    let header_rect = Rect::from_min_size(table_rect.min, vec2(total_w, header_h));
    painter.rect_filled(header_rect, CornerRadius::ZERO, header_fill);

    // Paint header cells
    let mut x = table_rect.min.x;
    for (c, galley) in header_galleys.into_iter().enumerate() {
        let y = header_rect.min.y + (header_h - galley.size().y) * 0.5;
        painter.galley(Pos2::new(x + CELL_PAD_X, y), galley, t.ink_strong);
        x += col_widths[c];
    }
    painter.hline(
        table_rect.x_range(),
        header_rect.max.y,
        Stroke::new(1.0, t.line_strong),
    );

    // Paint body rows
    let mut y = header_rect.max.y;
    for (idx, (galleys, row_h)) in body_rows.into_iter().enumerate() {
        let row_rect = Rect::from_min_size(Pos2::new(table_rect.min.x, y), vec2(total_w, row_h));
        if idx % 2 == 1 {
            let stripe = if t.dark {
                Color32::from_rgb(20, 22, 26)
            } else {
                t.sunken
            };
            painter.rect_filled(row_rect, CornerRadius::ZERO, stripe);
        }

        let mut cx = table_rect.min.x;
        for (c, galley) in galleys.into_iter().enumerate() {
            let gy = row_rect.min.y + (row_h - galley.size().y) * 0.5;
            painter.galley(Pos2::new(cx + CELL_PAD_X, gy), galley, t.ink);
            cx += col_widths[c];
        }

        if idx + 1 < table.rows.len() {
            painter.hline(
                table_rect.x_range(),
                row_rect.max.y,
                Stroke::new(1.0, t.line),
            );
        }
        y += row_h;
    }

    // Paint vertical column dividers (top to bottom)
    let mut col_x = table_rect.min.x;
    for width in col_widths.iter().take(ncols.saturating_sub(1)) {
        col_x += *width;
        painter.vline(col_x, table_rect.y_range(), Stroke::new(1.0, t.line));
    }

    // Paint sharp outer border
    painter.rect_stroke(
        table_rect,
        CornerRadius::ZERO,
        Stroke::new(1.0, t.line_strong),
        StrokeKind::Inside,
    );
}

fn fit_column_widths(desired: &[f32], avail_w: f32) -> Vec<f32> {
    let sum: f32 = desired.iter().sum();
    if sum <= avail_w {
        return desired.to_vec();
    }

    let mut widths = desired.to_vec();
    let mut excess = sum - avail_w;

    // Shrink the widest columns iteratively above MIN_COL_W
    while excess > 0.5 {
        let max_w = widths.iter().copied().fold(0.0f32, f32::max);
        if max_w <= MIN_COL_W + 0.5 {
            break;
        }
        let next_w = widths
            .iter()
            .copied()
            .filter(|&w| w < max_w - 0.5)
            .fold(MIN_COL_W, f32::max);
        let widest_indices: Vec<usize> = widths
            .iter()
            .enumerate()
            .filter(|(_, w)| (**w - max_w).abs() < 0.5)
            .map(|(idx, _)| idx)
            .collect();
        let step_per_col = ((max_w - next_w).min(excess / widest_indices.len() as f32)).max(0.5);
        for idx in widest_indices {
            let take = (widths[idx] - MIN_COL_W).min(step_per_col);
            widths[idx] -= take;
            excess -= take;
        }
    }
    widths
}

/// Parses inline `` `code` `` and `**bold**` spans into a single wrapped `LayoutJob`.
fn build_cell_layout_job(raw: &str, t: &Tokens, is_header: bool, wrap_w: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_w;

    let base_fmt = TextFormat {
        font_id: FontId::proportional(12.0),
        color: if is_header { t.ink_strong } else { t.ink },
        ..Default::default()
    };
    let bold_fmt = TextFormat {
        font_id: FontId::proportional(12.0),
        color: t.ink_strong,
        ..Default::default()
    };
    let code_fmt = TextFormat {
        font_id: FontId::new(11.5, FontFamily::Monospace),
        color: t.accent_ink,
        background: if t.dark {
            Color32::from_rgb(24, 28, 36)
        } else {
            t.panel
        },
        ..Default::default()
    };

    let mut rest = raw;
    while !rest.is_empty() {
        let next_code = rest.find('`');
        let next_bold = rest.find("**");

        let pick = match (next_code, next_bold) {
            (Some(c), Some(b)) if c < b => Some((c, "`", &code_fmt)),
            (Some(c), None) => Some((c, "`", &code_fmt)),
            (_, Some(b)) => Some((b, "**", &bold_fmt)),
            (None, None) => None,
        };

        if let Some((pos, delim, fmt)) = pick {
            if pos > 0 {
                job.append(&rest[..pos], 0.0, base_fmt.clone());
            }
            let after = &rest[pos + delim.len()..];
            if let Some(end) = after.find(delim) {
                job.append(&after[..end], 0.0, fmt.clone());
                rest = &after[end + delim.len()..];
            } else {
                job.append(delim, 0.0, base_fmt.clone());
                rest = after;
            }
        } else {
            job.append(rest, 0.0, base_fmt.clone());
            break;
        }
    }

    job
}

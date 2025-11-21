use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine, Points as CanvasPoints};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::Panel;

use crate::ui::colors::current as theme_current;

/// Draw a chart of file sizes using Canvas.
pub fn draw_chart(f: &mut Frame, area: Rect, panel: &Panel) {
    // Collect numeric data from panel entries (skip synthetic rows).
    let mut points: Vec<(f64, f64)> = Vec::new();
    for (i, e) in panel.entries.iter().enumerate() {
        // Use index as X and size (or 0 for directories) as Y
        let y = if e.is_dir { 0.0 } else { e.size as f64 };
        points.push((i as f64, y));
    }

    let theme = theme_current();

    if points.is_empty() {
        let p = Paragraph::new("No numeric data for chart").block(
            Block::default()
                .borders(Borders::ALL)
                .title("Chart")
                .style(theme.preview_block_style),
        );
        f.render_widget(p, area);
        return;
    }

    // Determine Y bounds (min/max) with some padding so the chart looks nicer.
    let ys: Vec<f64> = points.iter().map(|(_, y)| *y).collect();
    let y_min = ys.iter().cloned().fold(f64::INFINITY, f64::min);
    let y_max = ys.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let y_pad = ((y_max - y_min) * 0.1).max(1.0);

    // Use a Canvas widget to render points and a connecting line.
    let x_min = 0.0f64;
    let x_max = (points.len().saturating_sub(1)) as f64;
    let y_min_plot = y_min - y_pad;
    let y_max_plot = y_max + y_pad;

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Sizes"))
        .x_bounds([x_min, x_max])
        .y_bounds([y_min_plot, y_max_plot]);

    f.render_widget(
        canvas.paint(|ctx| {
            // Draw points
            ctx.draw(&CanvasPoints {
                coords: &points,
                color: Color::Cyan,
            });
            // Draw connecting line segments between consecutive points
            for pair in points.windows(2) {
                if let [(x1, y1), (x2, y2)] = pair {
                    ctx.draw(&CanvasLine {
                        x1: *x1,
                        y1: *y1,
                        x2: *x2,
                        y2: *y2,
                        color: Color::Yellow,
                    });
                }
            }
        }),
        area,
    );
}

/// Draw a compact sparkline-like visualization of file sizes.
pub fn draw_sparkline(f: &mut Frame, area: Rect, panel: &Panel) {
    // Unicode sparkline blocks from low -> high
    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let theme = theme_current();

    // Collect numeric data from panel entries (skip synthetic rows).
    let values: Vec<f64> = panel
        .entries
        .iter()
        .map(|e| if e.is_dir { 0.0 } else { e.size as f64 })
        .collect();

    if values.is_empty() {
        let p = Paragraph::new("No data").block(
            Block::default()
                .borders(Borders::ALL)
                .title("Sparkline")
                .style(theme.preview_block_style),
        );
        f.render_widget(p, area);
        return;
    }

    // Determine drawing width (leave 2 chars for borders/padding)
    let width = area.width.saturating_sub(2) as usize;
    let width = std::cmp::max(1, width);

    // If we have more values than width, downsample by taking the max in each bucket
    let mut buckets: Vec<f64> = Vec::with_capacity(width);
    if values.len() <= width {
        buckets.extend(values.iter().cloned());
    } else {
        let chunk = (values.len() as f64) / (width as f64);
        for i in 0..width {
            let start = (i as f64 * chunk).floor() as usize;
            let end = ((i as f64 + 1.0) * chunk).ceil() as usize;
            let end = std::cmp::min(end, values.len());
            if start >= end {
                buckets.push(0.0);
            } else {
                let max = values[start..end]
                    .iter()
                    .cloned()
                    .fold(f64::NEG_INFINITY, f64::max);
                buckets.push(if max.is_finite() { max } else { 0.0 });
            }
        }
    }

    // Normalize buckets to 0..(BLOCKS.len()-1)
    let max_v = buckets.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let min_v = buckets.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_v = if max_v.is_finite() { max_v } else { 0.0 };
    let min_v = if min_v.is_finite() { min_v } else { 0.0 };
    let range = (max_v - min_v).max(1.0);

    let mut spark = String::with_capacity(buckets.len());
    for v in buckets.iter() {
        let norm = ((*v - min_v) / range).clamp(0.0, 1.0);
        let idx = (norm * ((BLOCKS.len() - 1) as f64)).round() as usize;
        spark.push(BLOCKS[idx]);
    }

    // Also show min/max numbers in a compact footer line
    let footer = format!("min:{:.0} max:{:.0}", min_v, max_v);

    // Build paragraph with the sparkline and footer stacked vertically
    let text = format!("{}\n{}", spark, footer);
    let p = Paragraph::new(text).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Sparkline")
            .style(theme.preview_block_style),
    );
    f.render_widget(p, area);
}

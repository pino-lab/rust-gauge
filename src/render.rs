use anyhow::{Context, Result};
use tray_icon::Icon;

use crate::{
    config::AppConfig,
    metrics::{MetricKind, MetricSample, MetricSnapshot, MetricStatus},
};

#[derive(Debug, Clone, Copy)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

pub fn tooltip_text(snapshot: &MetricSnapshot, config: &AppConfig) -> String {
    if snapshot.samples.is_empty() {
        return format!("{}\nNo metrics enabled", config.app_name);
    }

    let metrics = snapshot
        .samples
        .chunks(2)
        .map(|chunk| {
            chunk
                .iter()
                .map(format_sample)
                .collect::<Vec<_>>()
                .join(" | ")
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("{}\n{}", config.app_name, metrics)
}

pub fn render_icon(snapshot: &MetricSnapshot, config: &AppConfig) -> Result<Icon> {
    let size = config.icon.size.clamp(16, 64);
    let mut canvas = Canvas::new(size, size);

    let track_color =
        parse_hex_color(&config.icon.track_color).context("invalid icon.track_color")?;

    let visible_samples = snapshot
        .samples
        .iter()
        .filter(|sample| {
            matches!(
                sample.kind,
                MetricKind::Cpu | MetricKind::Memory | MetricKind::Gpu | MetricKind::Npu
            ) && sample.value_percent.is_some()
        })
        .collect::<Vec<_>>();

    if visible_samples.is_empty() {
        draw_empty_state(&mut canvas, track_color);
    } else {
        draw_metric_grid(&mut canvas, &visible_samples, track_color);
    }

    Icon::from_rgba(canvas.pixels, size, size).context("failed to create tray icon")
}

fn format_sample(sample: &MetricSample) -> String {
    let label = metric_label(sample.kind, sample.label);

    match (&sample.display_value, sample.value_percent, sample.status) {
        (Some(value), _, _) => format!("{} {}", label, value),
        (None, Some(value), _) => format!("{} {:.0}%", label, value.clamp(0.0, 100.0)),
        (None, None, MetricStatus::Unsupported) => format!("{} unsupported", label),
        (None, None, MetricStatus::Ok) => format!("{} unknown", label),
    }
}

fn metric_label(kind: MetricKind, fallback: &'static str) -> &'static str {
    let label = match kind {
        MetricKind::Cpu => "CPU",
        MetricKind::Memory => "MEM",
        MetricKind::Gpu => "GPU",
        MetricKind::Npu => "NPU",
        MetricKind::Network => "NET",
    };

    if label.is_empty() {
        fallback
    } else {
        label
    }
}

fn draw_empty_state(canvas: &mut Canvas, color: Rgba) {
    let size = canvas.width;
    let margin = size / 4;

    canvas.fill_rect(margin, margin, size - margin * 2, size / 8, color);
    canvas.fill_rect(margin, size / 2, size - margin * 2, size / 8, color);
    canvas.fill_rect(
        margin,
        size - margin - size / 8,
        size - margin * 2,
        size / 8,
        color,
    );
}

fn draw_metric_grid(canvas: &mut Canvas, samples: &[&MetricSample], track_color: Rgba) {
    let ordered_samples = [
        MetricKind::Cpu,
        MetricKind::Memory,
        MetricKind::Gpu,
        MetricKind::Npu,
    ]
    .into_iter()
    .filter_map(|kind| {
        samples
            .iter()
            .find(|sample| sample.kind == kind)
            .map(|sample| {
                (
                    kind,
                    sample.value_percent.unwrap_or_default().clamp(0.0, 100.0),
                )
            })
    })
    .collect::<Vec<_>>();

    draw_horizontal_metric_cells(canvas, &ordered_samples, track_color);
}

fn draw_horizontal_metric_cells(
    canvas: &mut Canvas,
    samples: &[(MetricKind, f32)],
    track_color: Rgba,
) {
    let size = canvas.width;
    let margin = (size / 10).max(2);
    let gap = 2;
    let count = samples.len() as u32;
    let available_width = size
        .saturating_sub(margin * 2)
        .saturating_sub(gap * count.saturating_sub(1));
    let cell_width = (available_width / count.max(1)).max(3);
    let cell_height = size.saturating_sub(margin * 2);

    for (index, (kind, value)) in samples.iter().enumerate() {
        let x = margin + index as u32 * (cell_width + gap);
        let y = margin;

        draw_metric_cell(
            canvas,
            Rect {
                x,
                y,
                width: cell_width,
                height: cell_height,
            },
            *value,
            metric_color(*kind),
            track_color,
        );
    }
}

fn draw_metric_cell(canvas: &mut Canvas, rect: Rect, value: f32, color: Rgba, track_color: Rgba) {
    let Rect {
        x,
        y,
        width,
        height,
    } = rect;

    canvas.fill_rect(x, y, width, height, with_alpha(track_color, 110));
    draw_cell_border(canvas, x, y, width, height, with_alpha(track_color, 220));

    let fill_height = ((height as f32 - 2.0) * (value / 100.0)).round() as u32;
    if fill_height > 0 {
        let fill_alpha = (145.0 + value * 1.05).round().clamp(145.0, 245.0) as u8;
        let fill_y = y + height.saturating_sub(1).saturating_sub(fill_height);
        canvas.fill_rect(
            x + 1,
            fill_y,
            width.saturating_sub(2),
            fill_height,
            with_alpha(color, fill_alpha),
        );
    }
}

fn draw_cell_border(canvas: &mut Canvas, x: u32, y: u32, width: u32, height: u32, color: Rgba) {
    canvas.fill_rect(x, y, width, 1, color);
    canvas.fill_rect(x, y + height.saturating_sub(1), width, 1, color);
    canvas.fill_rect(x, y, 1, height, color);
    canvas.fill_rect(x + width.saturating_sub(1), y, 1, height, color);
}

fn metric_color(kind: MetricKind) -> Rgba {
    match kind {
        MetricKind::Cpu => Rgba {
            r: 0,
            g: 210,
            b: 255,
            a: 255,
        },
        MetricKind::Memory => Rgba {
            r: 37,
            g: 99,
            b: 235,
            a: 255,
        },
        MetricKind::Gpu => Rgba {
            r: 211,
            g: 92,
            b: 255,
            a: 255,
        },
        MetricKind::Npu => Rgba {
            r: 182,
            g: 92,
            b: 255,
            a: 255,
        },
        MetricKind::Network => Rgba {
            r: 251,
            g: 191,
            b: 36,
            a: 255,
        },
    }
}

fn parse_hex_color(input: &str) -> Result<Rgba> {
    let value = input.trim().trim_start_matches('#');
    anyhow::ensure!(value.len() == 6, "expected #RRGGBB");

    let r = u8::from_str_radix(&value[0..2], 16)?;
    let g = u8::from_str_radix(&value[2..4], 16)?;
    let b = u8::from_str_radix(&value[4..6], 16)?;

    Ok(Rgba { r, g, b, a: 255 })
}

fn with_alpha(color: Rgba, a: u8) -> Rgba {
    Rgba { a, ..color }
}

struct Canvas {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; (width * height * 4) as usize],
        }
    }

    fn fill_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: Rgba) {
        let max_x = (x + width).min(self.width);
        let max_y = (y + height).min(self.height);

        for py in y..max_y {
            for px in x..max_x {
                let offset = ((py * self.width + px) * 4) as usize;
                self.pixels[offset] = color.r;
                self.pixels[offset + 1] = color.g;
                self.pixels[offset + 2] = color.b;
                self.pixels[offset + 3] = color.a;
            }
        }
    }
}

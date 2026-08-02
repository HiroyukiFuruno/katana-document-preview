use std::env;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{Context, Result, bail};
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::{RenderCache, RenderSettings, render};
use image::ImageFormat;
use office_oxide::Document;
use serde_json::json;

fn main() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let mode = args
        .next()
        .and_then(|value| value.into_string().ok())
        .context("mode must be `office` or `pdf`")?;
    let input = args.next().map(PathBuf::from).context("input path is required")?;
    let output = args
        .next()
        .map(PathBuf::from)
        .context("output directory is required")?;
    if args.next().is_some() {
        bail!("unexpected extra argument");
    }

    fs::create_dir_all(&output)
        .with_context(|| format!("failed to create {}", output.display()))?;

    match mode.as_str() {
        "office" => evaluate_office(&input, &output),
        "pdf" => evaluate_pdf(&input, &output),
        _ => bail!("unsupported mode: {mode}"),
    }
}

fn evaluate_office(input: &Path, output: &Path) -> Result<()> {
    let input_size = fs::metadata(input)
        .with_context(|| format!("failed to read metadata for {}", input.display()))?
        .len();

    let parse_started = Instant::now();
    let document =
        Document::open(input).with_context(|| format!("failed to open {}", input.display()))?;
    let parse_elapsed = parse_started.elapsed();

    let ir_started = Instant::now();
    let ir = document.to_ir();
    let ir_elapsed = ir_started.elapsed();

    let html_started = Instant::now();
    let html = ir.to_html();
    let html_elapsed = html_started.elapsed();
    let plain_text = ir.plain_text();
    let ir_json = serde_json::to_vec_pretty(&ir).context("failed to serialize document IR")?;

    fs::write(output.join("document.html"), &html).context("failed to write HTML output")?;
    fs::write(output.join("document.txt"), &plain_text)
        .context("failed to write text output")?;
    fs::write(output.join("document-ir.json"), &ir_json)
        .context("failed to write IR output")?;

    let metrics = json!({
        "engine": "office_oxide",
        "engine_version": "0.1.8",
        "format": format!("{:?}", document.format()).to_lowercase(),
        "input_bytes": input_size,
        "sections": ir.sections.len(),
        "elements": ir.sections.iter().map(|section| section.elements.len()).sum::<usize>(),
        "plain_text_bytes": plain_text.len(),
        "html_bytes": html.len(),
        "ir_json_bytes": ir_json.len(),
        "parse_microseconds": parse_elapsed.as_micros(),
        "ir_microseconds": ir_elapsed.as_micros(),
        "html_microseconds": html_elapsed.as_micros(),
        "total_microseconds": (parse_elapsed + ir_elapsed + html_elapsed).as_micros(),
    });
    write_metrics(output, metrics)
}

fn evaluate_pdf(input: &Path, output: &Path) -> Result<()> {
    let input_bytes =
        fs::read(input).with_context(|| format!("failed to read {}", input.display()))?;
    let input_size = input_bytes.len();

    let parse_started = Instant::now();
    let pdf = Pdf::new(input_bytes)
        .map_err(|error| anyhow::anyhow!("failed to parse PDF: {error:?}"))?;
    let parse_elapsed = parse_started.elapsed();

    let interpreter_settings = InterpreterSettings::default();
    let render_settings = RenderSettings::default();
    let cache = RenderCache::new();
    let render_started = Instant::now();
    let mut rendered_bytes = 0_usize;
    let mut page_render_microseconds = Vec::with_capacity(pdf.pages().len());
    for (index, page) in pdf.pages().iter().enumerate() {
        let page_started = Instant::now();
        let transparent_png = render(page, &cache, &interpreter_settings, &render_settings)
            .into_png()
            .context("failed to encode rendered page")?;
        let png = flatten_png_on_white(&transparent_png)?;
        page_render_microseconds.push(page_started.elapsed().as_micros());
        rendered_bytes += png.len();
        fs::write(output.join(format!("page-{index:04}.png")), png)
            .with_context(|| format!("failed to write rendered page {index}"))?;
    }
    let render_elapsed = render_started.elapsed();

    let metrics = json!({
        "engine": "hayro",
        "engine_version": "0.7.1",
        "format": "pdf",
        "input_bytes": input_size,
        "pages": pdf.pages().len(),
        "rendered_png_bytes": rendered_bytes,
        "parse_microseconds": parse_elapsed.as_micros(),
        "first_page_microseconds": page_render_microseconds.first(),
        "page_render_microseconds": page_render_microseconds,
        "render_microseconds": render_elapsed.as_micros(),
        "total_microseconds": (parse_elapsed + render_elapsed).as_micros(),
    });
    write_metrics(output, metrics)
}

fn flatten_png_on_white(png: &[u8]) -> Result<Vec<u8>> {
    let mut rgba = image::load_from_memory_with_format(png, ImageFormat::Png)
        .context("failed to decode rendered page")?
        .into_rgba8();
    for pixel in rgba.pixels_mut() {
        let alpha = u16::from(pixel[3]);
        for channel in &mut pixel.0[..3] {
            let value = u16::from(*channel);
            *channel = ((value * alpha + 255 * (255 - alpha) + 127) / 255) as u8;
        }
        pixel[3] = 255;
    }

    let mut output = Cursor::new(Vec::new());
    rgba.write_to(&mut output, ImageFormat::Png)
        .context("failed to encode flattened page")?;
    Ok(output.into_inner())
}

fn write_metrics(output: &Path, metrics: serde_json::Value) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(&metrics).context("failed to serialize metrics")?;
    fs::write(output.join("metrics.json"), bytes).context("failed to write metrics")
}

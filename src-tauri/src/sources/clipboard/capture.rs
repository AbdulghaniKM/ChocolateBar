use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use super::history::Content;
use crate::core::clipboard;
use crate::core::dib::{bgra_from_dib, fit_within, BgraImage};

const THUMBNAIL_SIZE: u32 = 240;
const MAX_TEXT_CHARS: usize = 100_000;

/// Reads whatever was just copied, preferring files, then text, then a bitmap, and skipping anything
/// the copying app marked as private.
pub fn read_current() -> Option<Content> {
    if clipboard::is_private() {
        return None;
    }
    if let Some(paths) = clipboard::read_files() {
        return Some(Content::Files(paths));
    }
    if let Some(text) = clipboard::read_text().filter(|text| !text.trim().is_empty()) {
        return Some(Content::Text(text.chars().take(MAX_TEXT_CHARS).collect()));
    }
    let dib = clipboard::read_image_bytes()?;
    let image = bgra_from_dib(&dib)?;
    let (width, height) = (image.width, image.height);
    let thumbnail = png_data_url(fit_within(image, THUMBNAIL_SIZE))?;
    Some(Content::Image { dib, thumbnail, width, height })
}

pub fn restore(content: &Content) -> bool {
    match content {
        Content::Text(text) => clipboard::write_text(text),
        Content::Image { dib, .. } => clipboard::write_image_dib(dib),
        Content::Files(paths) => clipboard::write_files(paths),
    }
}

fn png_data_url(image: BgraImage) -> Option<String> {
    let rgba: Vec<u8> = image.pixels.chunks_exact(4).flat_map(|pixel| [pixel[2], pixel[1], pixel[0], pixel[3]]).collect();
    let mut encoded = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut encoded, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&rgba).ok()?;
    }
    Some(format!("data:image/png;base64,{}", STANDARD.encode(encoded)))
}

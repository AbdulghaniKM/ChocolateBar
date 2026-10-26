use windows::Globalization::Language;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;

use crate::core::dib::{fit_within, BgraImage};

pub const AUTO_LANGUAGE: &str = "auto";

pub fn recognize_text(image: BgraImage, language_choice: &str) -> windows::core::Result<String> {
    let engine = engine_for(language_choice)?;
    let fitted = fit_within(image, OcrEngine::MaxImageDimension()?);
    let bitmap = bitmap_from(&fitted)?;
    let result = engine.RecognizeAsync(&bitmap)?.join()?;
    Ok(result.Text()?.to_string())
}

pub fn available_language_tags() -> Vec<String> {
    let Ok(languages) = OcrEngine::AvailableRecognizerLanguages() else {
        return Vec::new();
    };
    languages.into_iter().filter_map(|language| language.LanguageTag().ok()).map(|tag| tag.to_string()).collect()
}

fn engine_for(language_choice: &str) -> windows::core::Result<OcrEngine> {
    if language_choice != AUTO_LANGUAGE {
        if let Some(language) = installed_language_matching(language_choice) {
            return OcrEngine::TryCreateFromLanguage(&language);
        }
    }
    OcrEngine::TryCreateFromUserProfileLanguages()
}

fn installed_language_matching(language_choice: &str) -> Option<Language> {
    let wanted = language_choice.to_lowercase();
    let languages = OcrEngine::AvailableRecognizerLanguages().ok()?;
    languages.into_iter().find(|language| {
        language.LanguageTag().is_ok_and(|tag| tag.to_string().to_lowercase().starts_with(&wanted))
    })
}

fn bitmap_from(image: &BgraImage) -> windows::core::Result<SoftwareBitmap> {
    let writer = DataWriter::new()?;
    writer.WriteBytes(&image.pixels)?;
    let buffer = writer.DetachBuffer()?;
    SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, image.width as i32, image.height as i32)
}

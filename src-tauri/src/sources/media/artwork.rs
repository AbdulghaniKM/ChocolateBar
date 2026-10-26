use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use windows::Storage::Streams::{DataReader, IRandomAccessStreamReference};
use windows::Win32::Foundation::E_FAIL;
use windows_core::{Error, Result};

const MAX_THUMBNAIL_BYTES: u64 = 6 * 1024 * 1024;
const FALLBACK_MIME_TYPE: &str = "image/jpeg";

pub fn read_data_url(thumbnail: &IRandomAccessStreamReference) -> Option<String> {
    match load_data_url(thumbnail) {
        Ok(data_url) => Some(data_url),
        Err(error) => {
            log::debug!("media artwork unavailable: {error}");
            None
        }
    }
}

fn load_data_url(thumbnail: &IRandomAccessStreamReference) -> Result<String> {
    let stream = thumbnail.OpenReadAsync()?.join()?;
    let size = stream.Size()?;
    if size == 0 || size > MAX_THUMBNAIL_BYTES {
        return Err(Error::new(E_FAIL, "thumbnail size out of range"));
    }
    let reader = DataReader::CreateDataReader(&stream)?;
    reader.LoadAsync(size as u32)?.join()?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes)?;
    Ok(encode_data_url(&stream.ContentType()?.to_string(), &bytes))
}

fn encode_data_url(mime_type: &str, bytes: &[u8]) -> String {
    let mime_type = if mime_type.is_empty() { FALLBACK_MIME_TYPE } else { mime_type };
    format!("data:{mime_type};base64,{}", STANDARD.encode(bytes))
}

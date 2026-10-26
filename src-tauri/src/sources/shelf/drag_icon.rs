use std::fs;
use std::path::PathBuf;

const DRAG_ICON_BYTES: &[u8] = include_bytes!("../../../icons/32x32.png");
const DRAG_ICON_FILE_NAME: &str = "chocolatebar-drag-icon.png";

pub fn ensure_drag_icon() -> Result<PathBuf, String> {
    let path = std::env::temp_dir().join(DRAG_ICON_FILE_NAME);
    if !path.exists() {
        fs::write(&path, DRAG_ICON_BYTES).map_err(|error| format!("drag icon not written: {error}"))?;
    }
    Ok(path)
}

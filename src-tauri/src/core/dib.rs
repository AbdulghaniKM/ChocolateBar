const BI_RGB: u32 = 0;
const BI_BITFIELDS: u32 = 3;
const MIN_HEADER_SIZE: usize = 40;
const BITFIELD_MASKS_SIZE: usize = 12;
const BYTES_PER_PALETTE_ENTRY: usize = 4;
const OPAQUE: u8 = 255;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BgraImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

struct DibHeader {
    size: usize,
    width: usize,
    height: usize,
    top_down: bool,
    bits_per_pixel: usize,
    compression: u32,
    palette_entries: usize,
}

impl DibHeader {
    fn parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < MIN_HEADER_SIZE {
            return None;
        }
        let height = read_i32(bytes, 8)?;
        Some(Self {
            size: read_u32(bytes, 0)? as usize,
            width: usize::try_from(read_i32(bytes, 4)?).ok()?,
            height: height.unsigned_abs() as usize,
            top_down: height < 0,
            bits_per_pixel: usize::from(read_u16(bytes, 14)?),
            compression: read_u32(bytes, 16)?,
            palette_entries: read_u32(bytes, 32)? as usize,
        })
    }

    fn is_supported(&self) -> bool {
        let supported_compression = self.compression == BI_RGB || self.compression == BI_BITFIELDS;
        let supported_depth = self.bits_per_pixel == 24 || self.bits_per_pixel == 32;
        supported_compression && supported_depth && self.width > 0 && self.height > 0
    }

    fn pixel_data_offset(&self) -> usize {
        let masks = if self.compression == BI_BITFIELDS && self.size == MIN_HEADER_SIZE {
            BITFIELD_MASKS_SIZE
        } else {
            0
        };
        self.size + masks + self.palette_entries * BYTES_PER_PALETTE_ENTRY
    }

    fn row_stride(&self) -> usize {
        (self.width * self.bits_per_pixel).div_ceil(32) * 4
    }
}

pub fn bgra_from_dib(bytes: &[u8]) -> Option<BgraImage> {
    let header = DibHeader::parse(bytes).filter(DibHeader::is_supported)?;
    let pixel_data = bytes.get(header.pixel_data_offset()..)?;
    let stride = header.row_stride();
    if pixel_data.len() < stride * header.height {
        return None;
    }
    let mut pixels = Vec::with_capacity(header.width * header.height * 4);
    for row in 0..header.height {
        let source_row = if header.top_down { row } else { header.height - 1 - row };
        let row_bytes = &pixel_data[source_row * stride..][..stride];
        append_row_as_bgra(&mut pixels, row_bytes, header.width, header.bits_per_pixel / 8);
    }
    Some(BgraImage { width: header.width as u32, height: header.height as u32, pixels })
}

fn append_row_as_bgra(pixels: &mut Vec<u8>, row: &[u8], width: usize, bytes_per_pixel: usize) {
    for pixel in row.chunks_exact(bytes_per_pixel).take(width) {
        pixels.extend_from_slice(&[pixel[0], pixel[1], pixel[2], OPAQUE]);
    }
}

pub fn fit_within(image: BgraImage, max_dimension: u32) -> BgraImage {
    let longest_side = image.width.max(image.height);
    if longest_side <= max_dimension {
        return image;
    }
    let scale = f64::from(max_dimension) / f64::from(longest_side);
    let width = ((f64::from(image.width) * scale) as u32).max(1);
    let height = ((f64::from(image.height) * scale) as u32).max(1);
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for y in 0..height {
        let source_y = (u64::from(y) * u64::from(image.height) / u64::from(height)) as usize;
        for x in 0..width {
            let source_x = (u64::from(x) * u64::from(image.width) / u64::from(width)) as usize;
            let offset = (source_y * image.width as usize + source_x) * 4;
            pixels.extend_from_slice(&image.pixels[offset..offset + 4]);
        }
    }
    BgraImage { width, height, pixels }
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?))
}

fn read_i32(bytes: &[u8], offset: usize) -> Option<i32> {
    Some(i32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dib(width: i32, height: i32, bits: u16, compression: u32, pixel_data: &[u8]) -> Vec<u8> {
        let mut bytes = vec![0u8; MIN_HEADER_SIZE];
        bytes[0..4].copy_from_slice(&40u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&width.to_le_bytes());
        bytes[8..12].copy_from_slice(&height.to_le_bytes());
        bytes[12..14].copy_from_slice(&1u16.to_le_bytes());
        bytes[14..16].copy_from_slice(&bits.to_le_bytes());
        bytes[16..20].copy_from_slice(&compression.to_le_bytes());
        if compression == BI_BITFIELDS {
            bytes.extend_from_slice(&[0u8; BITFIELD_MASKS_SIZE]);
        }
        bytes.extend_from_slice(pixel_data);
        bytes
    }

    #[test]
    fn bottom_up_32_bit_rows_are_flipped_and_made_opaque() {
        let bottom_row = [1, 2, 3, 0, 4, 5, 6, 0];
        let top_row = [7, 8, 9, 0, 10, 11, 12, 0];
        let data = [bottom_row, top_row].concat();
        let image = bgra_from_dib(&dib(2, 2, 32, BI_BITFIELDS, &data)).unwrap();
        assert_eq!((image.width, image.height), (2, 2));
        assert_eq!(&image.pixels[..4], &[7, 8, 9, OPAQUE]);
        assert_eq!(&image.pixels[12..16], &[4, 5, 6, OPAQUE]);
    }

    #[test]
    fn top_down_24_bit_rows_respect_four_byte_padding() {
        let data = [1, 2, 3, 0, 4, 5, 6, 0];
        let image = bgra_from_dib(&dib(1, -2, 24, BI_RGB, &data)).unwrap();
        assert_eq!(image.pixels, vec![1, 2, 3, OPAQUE, 4, 5, 6, OPAQUE]);
    }

    #[test]
    fn truncated_pixel_data_is_rejected() {
        assert_eq!(bgra_from_dib(&dib(2, 2, 32, BI_RGB, &[0; 8])), None);
    }

    #[test]
    fn unsupported_depth_is_rejected() {
        assert_eq!(bgra_from_dib(&dib(1, 1, 8, BI_RGB, &[0; 4])), None);
    }

    #[test]
    fn small_images_are_left_alone() {
        let image = BgraImage { width: 2, height: 1, pixels: vec![0; 8] };
        assert_eq!(fit_within(image.clone(), 10), image);
    }

    #[test]
    fn large_images_are_scaled_to_the_limit() {
        let image = BgraImage { width: 400, height: 200, pixels: vec![9; 400 * 200 * 4] };
        let scaled = fit_within(image, 100);
        assert_eq!((scaled.width, scaled.height), (100, 50));
        assert_eq!(scaled.pixels.len(), 100 * 50 * 4);
    }
}

//! Color pictures: decoding PNG and JPEG, sampling, gray levels, PNG out.

use std::io::Cursor;

use crate::CameraError;

/// Largest picture side accepted, pixels.
pub const MAX_SIDE: u32 = 8192;

/// An 8-bit RGB picture, rows top to bottom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picture {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
}

impl Picture {
    /// A picture from packed RGB bytes.
    ///
    /// # Errors
    ///
    /// A size of zero, above [`MAX_SIDE`], or a buffer of the wrong length.
    pub fn new(width: u32, height: u32, rgb: Vec<u8>) -> Result<Self, CameraError> {
        if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
            return Err(CameraError::new(format!(
                "a picture must be 1–{MAX_SIDE} pixels on each side, not {width} × {height}"
            )));
        }
        if rgb.len() != width as usize * height as usize * 3 {
            return Err(CameraError::new("the picture data has the wrong length"));
        }
        Ok(Self { width, height, rgb })
    }

    /// A picture of one color.
    #[must_use]
    pub fn filled(width: u32, height: u32, color: [u8; 3]) -> Self {
        let n = width as usize * height as usize;
        Self {
            width,
            height,
            rgb: color.iter().copied().cycle().take(n * 3).collect(),
        }
    }

    /// Width, pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height, pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The packed RGB bytes.
    #[must_use]
    pub fn rgb(&self) -> &[u8] {
        &self.rgb
    }

    /// One pixel.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> [u8; 3] {
        let i = (y as usize * self.width as usize + x as usize) * 3;
        [self.rgb[i], self.rgb[i + 1], self.rgb[i + 2]]
    }

    /// Sets one pixel.
    pub fn set(&mut self, x: u32, y: u32, c: [u8; 3]) {
        let i = (y as usize * self.width as usize + x as usize) * 3;
        self.rgb[i..i + 3].copy_from_slice(&c);
    }

    /// Bilinear sample at pixel coordinates (pixel centers at `i + 0.5`);
    /// `None` outside the picture.
    #[must_use]
    pub fn sample(&self, u: f64, v: f64) -> Option<[u8; 3]> {
        let (x, y) = (u - 0.5, v - 0.5);
        let (w, h) = (f64::from(self.width), f64::from(self.height));
        if !(x >= -0.5 && y >= -0.5 && x <= w - 0.5 && y <= h - 0.5) {
            return None;
        }
        let x = x.clamp(0.0, w - 1.0);
        let y = y.clamp(0.0, h - 1.0);
        let (x0, y0) = (x.floor() as u32, y.floor() as u32);
        let (x1, y1) = ((x0 + 1).min(self.width - 1), (y0 + 1).min(self.height - 1));
        let (fx, fy) = (x - f64::from(x0), y - f64::from(y0));
        let mut out = [0u8; 3];
        let (a, b, c, d) = (
            self.get(x0, y0),
            self.get(x1, y0),
            self.get(x0, y1),
            self.get(x1, y1),
        );
        for k in 0..3 {
            let top = f64::from(a[k]) * (1.0 - fx) + f64::from(b[k]) * fx;
            let bottom = f64::from(c[k]) * (1.0 - fx) + f64::from(d[k]) * fx;
            out[k] = (top * (1.0 - fy) + bottom * fy).round().clamp(0.0, 255.0) as u8;
        }
        Some(out)
    }

    /// Gray levels (Rec. 601 luma), row by row.
    #[must_use]
    pub fn gray(&self) -> Vec<u8> {
        self.rgb
            .as_chunks::<3>()
            .0
            .iter()
            .map(|p| {
                ((299 * u32::from(p[0]) + 587 * u32::from(p[1]) + 114 * u32::from(p[2]) + 500)
                    / 1000) as u8
            })
            .collect()
    }

    /// The picture scaled to `width × height` (bilinear).
    #[must_use]
    pub fn resized(&self, width: u32, height: u32) -> Self {
        if (width, height) == (self.width, self.height) {
            return self.clone();
        }
        let sx = f64::from(self.width) / f64::from(width);
        let sy = f64::from(self.height) / f64::from(height);
        let mut out = Self::filled(width, height, [0, 0, 0]);
        for y in 0..height {
            for x in 0..width {
                let c = self
                    .sample((f64::from(x) + 0.5) * sx, (f64::from(y) + 0.5) * sy)
                    .unwrap_or([0, 0, 0]);
                out.set(x, y, c);
            }
        }
        out
    }

    /// Reads a PNG or JPEG.
    ///
    /// # Errors
    ///
    /// An unknown or broken file.
    pub fn decode(bytes: &[u8]) -> Result<Self, CameraError> {
        if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            decode_png(bytes)
        } else if bytes.starts_with(&[0xFF, 0xD8]) {
            decode_jpeg(bytes)
        } else {
            Err(CameraError::new("not a PNG or JPEG picture"))
        }
    }

    /// Reads a picture file.
    ///
    /// # Errors
    ///
    /// The file cannot be read or decoded.
    pub fn load(path: &std::path::Path) -> Result<Self, CameraError> {
        let bytes = std::fs::read(path)
            .map_err(|e| CameraError::new(format!("cannot read {}: {e}", path.display())))?;
        Self::decode(&bytes).map_err(|e| CameraError::new(format!("{}: {e}", path.display())))
    }

    /// An RGB PNG of the picture.
    #[must_use]
    pub fn encode_png(&self) -> Vec<u8> {
        encode_png(self.width, self.height, png::ColorType::Rgb, &self.rgb)
    }
}

/// A PNG of packed `color` pixels; the same pixels always give the same
/// bytes.
#[must_use]
pub fn encode_png(width: u32, height: u32, color: png::ColorType, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(color);
        encoder.set_depth(png::BitDepth::Eight);
        if let Ok(mut writer) = encoder.write_header() {
            let _ = writer.write_image_data(data);
        }
    }
    out
}

fn decode_png(bytes: &[u8]) -> Result<Picture, CameraError> {
    let fail = |e: png::DecodingError| CameraError::new(format!("cannot read the PNG: {e}"));
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(fail)?;
    let (w, h) = (reader.info().width, reader.info().height);
    if w > MAX_SIDE || h > MAX_SIDE {
        return Err(CameraError::new("the picture is too large"));
    }
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| CameraError::new("the picture is too large"))?;
    let mut buf = vec![0; size];
    let frame = reader.next_frame(&mut buf).map_err(fail)?;
    let channels = frame.color_type.samples();
    let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
    for row in buf.chunks(frame.line_size).take(h as usize) {
        for px in row.chunks(channels).take(w as usize) {
            match *px {
                [g] | [g, _] => rgb.extend([g, g, g]),
                [r, g, b] | [r, g, b, _] => rgb.extend([r, g, b]),
                _ => return Err(CameraError::new("unsupported PNG pixel layout")),
            }
        }
    }
    Picture::new(w, h, rgb)
}

fn decode_jpeg(bytes: &[u8]) -> Result<Picture, CameraError> {
    use zune_jpeg::zune_core::bytestream::ZCursor;
    use zune_jpeg::zune_core::colorspace::ColorSpace;
    use zune_jpeg::zune_core::options::DecoderOptions;

    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::RGB);
    let mut decoder = zune_jpeg::JpegDecoder::new_with_options(ZCursor::new(bytes), options);
    decoder
        .decode_headers()
        .map_err(|e| CameraError::new(format!("cannot read the JPEG: {e:?}")))?;
    let info = decoder
        .info()
        .ok_or_else(|| CameraError::new("cannot read the JPEG header"))?;
    let (w, h) = (u32::from(info.width), u32::from(info.height));
    if w > MAX_SIDE || h > MAX_SIDE {
        return Err(CameraError::new("the picture is too large"));
    }
    let rgb = decoder
        .decode()
        .map_err(|e| CameraError::new(format!("cannot read the JPEG: {e:?}")))?;
    Picture::new(w, h, rgb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_round_trips_and_sampling_blends() {
        let mut p = Picture::filled(4, 2, [10, 20, 30]);
        p.set(1, 0, [110, 120, 130]);
        let back = Picture::decode(&p.encode_png()).unwrap();
        assert_eq!(back, p);
        // Halfway between pixel 0 and pixel 1 centers.
        assert_eq!(p.sample(1.0, 0.5), Some([60, 70, 80]));
        assert_eq!(p.sample(-1.0, 0.5), None);
        assert_eq!(p.gray()[1], 118);
        assert_eq!(p.resized(2, 1).width(), 2);
        assert!(Picture::new(0, 1, vec![]).is_err());
        assert!(Picture::decode(b"GIF89a").is_err());
    }
}

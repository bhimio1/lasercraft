//! PNG and JPEG to grayscale, and grayscale to PNG (`docs/spec/images.md`).

use std::io::Cursor;

use crate::{Gray, RasterError, fail};

/// A decoded image and the resolution its file states.
#[derive(Debug, Clone, PartialEq)]
pub struct Decoded {
    /// The grayscale pixels.
    pub gray: Gray,
    /// Pixels per mm along x and y, when the file says.
    pub pixels_per_mm: Option<[f64; 2]>,
}

/// Luminance of an RGB pixel (ITU-R BT.601 weights), composited over white by
/// `alpha`.
fn luma(r: u8, g: u8, b: u8, alpha: u8) -> u8 {
    let y = 0.299 * f64::from(r) + 0.587 * f64::from(g) + 0.114 * f64::from(b);
    over_white(y, alpha)
}

fn over_white(level: f64, alpha: u8) -> u8 {
    let a = f64::from(alpha) / 255.0;
    // In 0..=255 by construction.
    (level * a + 255.0 * (1.0 - a)).round().clamp(0.0, 255.0) as u8
}

/// Decodes a PNG or JPEG (recognized by its signature).
///
/// # Errors
///
/// An unknown or broken file, or one too large.
pub fn decode(bytes: &[u8]) -> Result<Decoded, RasterError> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        decode_png(bytes)
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        decode_jpeg(bytes)
    } else {
        Err(fail("not a PNG or JPEG file"))
    }
}

fn decode_png(bytes: &[u8]) -> Result<Decoded, RasterError> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|e| fail(format!("cannot read the PNG: {e}")))?;
    let (w, h) = {
        let info = reader.info();
        (info.width, info.height)
    };
    // Check the size before allocating.
    crate::bitmap::check_size(w, h)?;
    let pixels_per_mm = reader.info().pixel_dims.and_then(|d| {
        (d.unit == png::Unit::Meter && d.xppu > 0 && d.yppu > 0)
            .then(|| [f64::from(d.xppu) / 1000.0, f64::from(d.yppu) / 1000.0])
    });
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| fail("the PNG is too large"))?;
    let mut buf = vec![0; size];
    let frame = reader
        .next_frame(&mut buf)
        .map_err(|e| fail(format!("cannot read the PNG: {e}")))?;
    let channels = frame.color_type.samples();
    let mut pixels = Vec::with_capacity(w as usize * h as usize);
    for row in buf.chunks(frame.line_size).take(h as usize) {
        for px in row.chunks(channels).take(w as usize) {
            pixels.push(match *px {
                [g] => g,
                [g, a] => over_white(f64::from(g), a),
                [r, g, b] => luma(r, g, b, 255),
                [r, g, b, a] => luma(r, g, b, a),
                _ => return Err(fail("unsupported PNG pixel layout")),
            });
        }
    }
    Ok(Decoded {
        gray: Gray::new(w, h, pixels)?,
        pixels_per_mm,
    })
}

/// The JFIF APP0 density, in pixels per mm (units 1: per inch, 2: per cm).
fn jfif_density(bytes: &[u8]) -> Option<[f64; 2]> {
    let mut i = 2;
    while i + 4 <= bytes.len() && bytes[i] == 0xFF {
        let marker = bytes[i + 1];
        let len = usize::from(u16::from_be_bytes([bytes[i + 2], bytes[i + 3]]));
        let body = bytes.get(i + 4..i + 2 + len)?;
        if marker == 0xE0 && body.starts_with(b"JFIF\0") && body.len() >= 12 {
            let units = body[7];
            let x = f64::from(u16::from_be_bytes([body[8], body[9]]));
            let y = f64::from(u16::from_be_bytes([body[10], body[11]]));
            let per_mm = match units {
                1 => 1.0 / 25.4,
                2 => 0.1,
                _ => return None,
            };
            return (x > 0.0 && y > 0.0).then_some([x * per_mm, y * per_mm]);
        }
        if marker == 0xDA {
            return None;
        }
        i += 2 + len;
    }
    None
}

fn decode_jpeg(bytes: &[u8]) -> Result<Decoded, RasterError> {
    use zune_jpeg::zune_core::bytestream::ZCursor;
    use zune_jpeg::zune_core::colorspace::ColorSpace;
    use zune_jpeg::zune_core::options::DecoderOptions;

    let options = DecoderOptions::default().jpeg_set_out_colorspace(ColorSpace::Luma);
    let mut decoder = zune_jpeg::JpegDecoder::new_with_options(ZCursor::new(bytes), options);
    decoder
        .decode_headers()
        .map_err(|e| fail(format!("cannot read the JPEG: {e:?}")))?;
    let info = decoder
        .info()
        .ok_or_else(|| fail("cannot read the JPEG header"))?;
    let (w, h) = (u32::from(info.width), u32::from(info.height));
    crate::bitmap::check_size(w, h)?;
    let pixels = decoder
        .decode()
        .map_err(|e| fail(format!("cannot read the JPEG: {e:?}")))?;
    Ok(Decoded {
        gray: Gray::new(w, h, pixels)?,
        pixels_per_mm: jfif_density(bytes),
    })
}

/// An 8-bit grayscale PNG of `gray`. The same bitmap always gives the same
/// bytes.
#[must_use]
pub fn encode_png(gray: &Gray) -> Vec<u8> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, gray.width(), gray.height());
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Balanced);
    // Writing to a Vec cannot fail, and the header matches the data.
    let written = encoder
        .write_header()
        .and_then(|mut w| w.write_image_data(gray.pixels()));
    debug_assert!(written.is_ok(), "{written:?}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png_of(color: png::ColorType, w: u32, h: u32, data: &[u8], dpm: Option<u32>) -> Vec<u8> {
        let mut out = Vec::new();
        let mut e = png::Encoder::new(&mut out, w, h);
        e.set_color(color);
        e.set_depth(png::BitDepth::Eight);
        e.set_pixel_dims(dpm.map(|d| png::PixelDimensions {
            xppu: d,
            yppu: d,
            unit: png::Unit::Meter,
        }));
        e.write_header().unwrap().write_image_data(data).unwrap();
        out
    }

    #[test]
    fn png_colors_become_luminance_over_white() {
        let rgba = png_of(
            png::ColorType::Rgba,
            3,
            1,
            &[255, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 128],
            Some(10_000),
        );
        let d = decode(&rgba).unwrap();
        assert_eq!(d.gray.pixels(), [76, 255, 127]);
        assert_eq!(d.pixels_per_mm, Some([10.0, 10.0]));
        let ga = png_of(png::ColorType::GrayscaleAlpha, 1, 1, &[0, 255], None);
        let d = decode(&ga).unwrap();
        assert_eq!((d.gray.pixels(), d.pixels_per_mm), (&[0u8][..], None));
    }

    #[test]
    fn grayscale_png_round_trips_identically() {
        let g = Gray::new(4, 3, (0..12).map(|v| v * 20).collect()).unwrap();
        let bytes = encode_png(&g);
        assert_eq!(bytes, encode_png(&g), "deterministic");
        assert_eq!(decode(&bytes).unwrap().gray, g);
    }

    #[test]
    fn jfif_density_is_read_from_app0() {
        let mut jfif = vec![0xFF, 0xD8, 0xFF, 0xE0, 0, 16];
        jfif.extend_from_slice(b"JFIF\0");
        jfif.extend_from_slice(&[1, 2, 1, 1, 44, 1, 44, 0, 0]);
        jfif.extend_from_slice(&[0xFF, 0xDA]);
        let [x, y] = jfif_density(&jfif).unwrap();
        assert!((x - 300.0 / 25.4).abs() < 1e-12 && (y - x).abs() < 1e-12);
        jfif[13] = 0;
        assert_eq!(jfif_density(&jfif), None, "aspect ratio only");
    }

    #[test]
    fn fixture_files_decode_with_their_resolution() {
        let jpg = decode(include_bytes!("../../../examples/fixtures/m7-tile.jpg")).unwrap();
        assert_eq!((jpg.gray.width(), jpg.gray.height()), (16, 8));
        assert!(
            jpg.gray.get(2, 4) < 20 && jpg.gray.get(13, 4) > 235,
            "dark left, white right"
        );
        let [x, _] = jpg.pixels_per_mm.unwrap();
        assert!((x - 300.0 / 25.4).abs() < 1e-9);
        let png = decode(include_bytes!("../../../examples/fixtures/m7-photo.png")).unwrap();
        assert_eq!((png.gray.width(), png.gray.height()), (120, 80));
        assert_eq!(png.pixels_per_mm, Some([10.0, 10.0]));
    }

    #[test]
    fn rejects_other_files() {
        assert!(decode(b"GIF89a").is_err());
        assert!(decode(b"\x89PNG\r\n\x1a\nbroken").is_err());
        assert!(decode(&[0xFF, 0xD8, 0xFF]).is_err());
    }
}

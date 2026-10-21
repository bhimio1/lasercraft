//! 8-bit grayscale bitmaps and the base64 text the project file stores them
//! in.

use crate::{RasterError, fail};

/// Most pixels in one image.
pub const MAX_PIXELS: u64 = 25_000_000;

/// Most pixels along one side.
pub const MAX_SIDE: u32 = 20_000;

/// Checks bitmap dimensions against the limits.
pub(crate) fn check_size(width: u32, height: u32) -> Result<(), RasterError> {
    if width == 0 || height == 0 {
        return Err(fail("an image needs at least one pixel"));
    }
    if width > MAX_SIDE || height > MAX_SIDE || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(fail(format!(
            "{width} × {height} pixels is too large (at most {MAX_SIDE} per side and {MAX_PIXELS} in all)"
        )));
    }
    Ok(())
}

/// An 8-bit grayscale bitmap, rows top to bottom; 0 is black, 255 white.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gray {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Gray {
    /// A bitmap from row-major pixels.
    ///
    /// # Errors
    ///
    /// Zero or oversized dimensions, or a pixel count that does not match.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, RasterError> {
        check_size(width, height)?;
        if pixels.len() as u64 != u64::from(width) * u64::from(height) {
            return Err(fail("pixel data does not match the size"));
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Row-major pixels.
    #[must_use]
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// The pixel at (`x`, `y`), which must be inside.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> u8 {
        self.pixels[y as usize * self.width as usize + x as usize]
    }

    /// Averages `k` × `k` blocks (partial blocks at the edges average what
    /// they have).
    #[must_use]
    pub fn shrink(&self, k: u32) -> Self {
        if k <= 1 {
            return self.clone();
        }
        let w = self.width.div_ceil(k);
        let h = self.height.div_ceil(k);
        let mut pixels = Vec::with_capacity(w as usize * h as usize);
        for by in 0..h {
            for bx in 0..w {
                let (mut sum, mut n) = (0u64, 0u64);
                for y in by * k..((by + 1) * k).min(self.height) {
                    for x in bx * k..((bx + 1) * k).min(self.width) {
                        sum += u64::from(self.get(x, y));
                        n += 1;
                    }
                }
                // n ≥ 1, and the average of bytes fits in a byte.
                pixels.push(((sum + n / 2) / n) as u8);
            }
        }
        Self {
            width: w,
            height: h,
            pixels,
        }
    }
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Standard base64 with padding (RFC 4648).
#[must_use]
pub fn base64_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            chunk.get(1).copied().unwrap_or(0),
            chunk.get(2).copied().unwrap_or(0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[(n >> (18 - 6 * i) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// Decodes standard base64; whitespace is ignored.
///
/// # Errors
///
/// A character outside the alphabet or a bad length.
pub fn base64_decode(text: &str) -> Result<Vec<u8>, RasterError> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut acc = 0u32;
    let mut bits = 0;
    let mut padding = 0;
    let mut count = 0;
    for c in text.bytes().filter(|c| !c.is_ascii_whitespace()) {
        count += 1;
        if c == b'=' {
            padding += 1;
            continue;
        }
        if padding > 0 {
            return Err(fail("base64 data continues after padding"));
        }
        let v = ALPHABET
            .iter()
            .position(|&a| a == c)
            .ok_or_else(|| fail(format!("`{}` is not base64", char::from(c))))?;
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    if count % 4 != 0 || padding > 2 {
        return Err(fail("base64 data has a bad length"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_round_trips_and_matches_rfc_4648() {
        for (plain, coded) in [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
        ] {
            assert_eq!(base64_encode(plain.as_bytes()), coded);
            assert_eq!(base64_decode(coded).unwrap(), plain.as_bytes());
        }
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(base64_decode(&base64_encode(&all)).unwrap(), all);
        assert!(base64_decode("Zm9v!").is_err());
        assert!(base64_decode("Zm9").is_err());
        assert!(base64_decode("Zg==Zg==").is_err());
        assert_eq!(base64_decode("Zm9v\nYmFy").unwrap(), b"foobar");
    }

    #[test]
    fn bitmaps_check_their_size_and_shrink_by_averaging() {
        assert!(Gray::new(0, 1, vec![]).is_err());
        assert!(Gray::new(2, 2, vec![0; 3]).is_err());
        assert!(Gray::new(MAX_SIDE + 1, 1, vec![0; MAX_SIDE as usize + 1]).is_err());
        let g = Gray::new(3, 2, vec![0, 100, 200, 50, 150, 250]).unwrap();
        assert_eq!(g.get(2, 1), 250);
        let s = g.shrink(2);
        assert_eq!((s.width(), s.height()), (2, 1));
        assert_eq!(s.pixels(), [75, 225]);
        assert_eq!(g.shrink(1), g);
    }
}

//! RGBA color type with alpha-blending helpers.

use std::fmt;

/// A 16-bit-per-channel RGBA color (matching `taffy`/`crossterm` precision needs).
///
/// Stored as `[r, g, b, a]` with each channel in `0..=255`. The alpha channel
/// uses straight (non-premultiplied) alpha; compositing helpers handle the
/// conversion internally.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red channel, `0..=255`.
    pub r: u8,
    /// Green channel, `0..=255`.
    pub g: u8,
    /// Blue channel, `0..=255`.
    pub b: u8,
    /// Alpha channel, `0..=255` (0 = fully transparent, 255 = opaque).
    pub a: u8,
}

impl Color {
    /// Fully opaque black.
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    /// Fully opaque white.
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    /// Fully opaque red.
    pub const RED: Self = Self::rgb(220, 60, 80);
    /// Fully opaque green.
    pub const GREEN: Self = Self::rgb(80, 200, 120);
    /// Fully opaque blue.
    pub const BLUE: Self = Self::rgb(80, 140, 240);
    /// Fully opaque cyan.
    pub const CYAN: Self = Self::rgb(80, 220, 220);
    /// Fully opaque yellow.
    pub const YELLOW: Self = Self::rgb(220, 200, 80);
    /// Fully opaque magenta.
    pub const MAGENTA: Self = Self::rgb(220, 100, 220);
    /// Fully transparent (used for "no fill").
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);

    /// Construct an opaque RGB color.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Construct an RGBA color.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// A 256-color palette index, encoded as an opaque RGB triple.
    ///
    /// This is a convenience for the common 16-color terminal palette; for
    /// true 256-color or truecolor output the backend is responsible for
    /// picking the closest representable color.
    pub const fn palette256(index: u8) -> Self {
        // 6x6x6 color cube starting at index 16.
        if index >= 16 && index < 232 {
            let v = index - 16;
            let r = (v / 36) % 6;
            let g = (v / 6) % 6;
            let b = v % 6;
            Self::rgb(
                55 * r + (if r > 0 { 40 } else { 0 }),
                55 * g + (if g > 0 { 40 } else { 0 }),
                55 * b + (if b > 0 { 40 } else { 0 }),
            )
        } else if index >= 232 {
            let gray = 8 + (index - 232) * 10;
            Self::rgb(gray, gray, gray)
        } else {
            // Standard 16-color palette approximation.
            match index {
                0 => Self::rgb(0, 0, 0),
                1 => Self::rgb(180, 30, 30),
                2 => Self::rgb(40, 160, 60),
                3 => Self::rgb(200, 170, 40),
                4 => Self::rgb(40, 90, 200),
                5 => Self::rgb(170, 50, 170),
                6 => Self::rgb(40, 170, 170),
                7 => Self::rgb(200, 200, 200),
                8 => Self::rgb(80, 80, 80),
                9 => Self::rgb(220, 60, 80),
                10 => Self::rgb(80, 200, 120),
                11 => Self::rgb(220, 200, 80),
                12 => Self::rgb(80, 140, 240),
                13 => Self::rgb(220, 100, 220),
                14 => Self::rgb(80, 220, 220),
                15 => Self::rgb(255, 255, 255),
                _ => Self::BLACK,
            }
        }
    }

    /// Alpha-blend `self` over `dst` using the "over" (Porter-Duff) operator.
    #[must_use]
    pub fn over(self, dst: Self) -> Self {
        if self.a == 0 {
            return dst;
        }
        if self.a == 255 {
            return self;
        }
        let sa = f32::from(self.a) / 255.0;
        let da = f32::from(dst.a) / 255.0;
        let out_a = sa + da * (1.0 - sa);
        if out_a == 0.0 {
            return Self::TRANSPARENT;
        }
        let blend = |s: u8, d: u8| -> u8 {
            let s = f32::from(s);
            let d = f32::from(d);
            let v = (s * sa + d * da * (1.0 - sa)) / out_a;
            v.round().clamp(0.0, 255.0) as u8
        };
        Self::rgba(
            blend(self.r, dst.r),
            blend(self.g, dst.g),
            blend(self.b, dst.b),
            (out_a * 255.0).round() as u8,
        )
    }

    /// Linear interpolation between two colors at `t` in `0.0..=1.0`.
    #[must_use]
    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let l = |a: u8, b: u8| -> u8 {
            let v = f32::from(a) + (f32::from(b) - f32::from(a)) * t;
            v.round().clamp(0.0, 255.0) as u8
        };
        Self::rgba(
            l(self.r, other.r),
            l(self.g, other.g),
            l(self.b, other.b),
            l(self.a, other.a),
        )
    }
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "Color(#{:02X}{:02X}{:02X})", self.r, self.g, self.b)
        } else {
            write!(
                f,
                "Color(#{:02X}{:02X}{:02X}{:02X})",
                self.r, self.g, self.b, self.a
            )
        }
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_over_anything_is_self() {
        let a = Color::rgb(10, 20, 30);
        let b = Color::rgb(200, 100, 50);
        assert_eq!(a.over(b), a);
    }

    #[test]
    fn transparent_over_anything_is_dst() {
        let a = Color::TRANSPARENT;
        let b = Color::rgb(200, 100, 50);
        assert_eq!(a.over(b), b);
    }

    #[test]
    fn lerp_endpoints() {
        let a = Color::rgb(0, 0, 0);
        let b = Color::rgb(255, 255, 255);
        assert_eq!(Color::lerp(a, b, 0.0), a);
        assert_eq!(Color::lerp(a, b, 1.0), b);
    }
}

use std::{borrow::Cow, num::ParseIntError, str::FromStr};

use crate::style::{TryFromDescriptors, special::SpecialError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8);

impl TryFromDescriptors for Color {
    fn try_from_descriptors(descriptors: Vec<Cow<'_, str>>) -> Result<Self, SpecialError> {
        let mut iter = descriptors.into_iter();
        let color_string = iter
            .next()
            .ok_or(SpecialError::MissingArgument("color_hex"))?;

        Ok(Self::from_str(&color_string)?)
    }
}

impl Color {
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let h = h.fract();
        let i = (h * 6.0).floor() as i32;
        let f = h * 6.0 - i as f32;

        let p = v * (1.0 - s);
        let q = v * (1.0 - f * s);
        let t = v * (1.0 - (1.0 - f) * s);

        let (r, g, b) = match i % 6 {
            0 => (v, t, p),
            1 => (q, v, p),
            2 => (p, v, t),
            3 => (p, q, v),
            4 => (t, p, v),
            _ => (v, p, q),
        };

        Color(
            (r * 255.0).round() as u8,
            (g * 255.0).round() as u8,
            (b * 255.0).round() as u8,
        )
    }

    pub fn lerp(self, target: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);

        let r = (self.0 as f32 + (target.0 as f32 - self.0 as f32) * t).round() as u8;
        let g = (self.1 as f32 + (target.1 as f32 - self.1 as f32) * t).round() as u8;
        let b = (self.2 as f32 + (target.2 as f32 - self.2 as f32) * t).round() as u8;

        Color(r, g, b)
    }

    pub fn gradient(start: Self, end: Self, steps: usize) -> Vec<Self> {
        if steps <= 1 {
            return vec![start];
        }

        (0..steps)
            .map(|i| {
                let t = i as f32 / (steps - 1) as f32;
                start.lerp(end, t)
            })
            .collect()
    }

    pub fn gradient_to(self, target: Self, steps: usize) -> Vec<Self> {
        if steps == 0 {
            return vec![];
        }
        if steps == 1 {
            return vec![self];
        }

        (0..steps)
            .map(|i| {
                let t = i as f32 / (steps - 1) as f32;

                self.lerp(target, t)
            })
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum ParseColorError {
    #[error("Invalid hex string length")]
    InvalidLength,

    InvalidHex(ParseIntError),
}

impl FromStr for Color {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.strip_prefix('#').unwrap_or(s);

        if hex.len() != 6 {
            return Err(ParseColorError::InvalidLength);
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(ParseColorError::InvalidHex)?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(ParseColorError::InvalidHex)?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(ParseColorError::InvalidHex)?;

        Ok(Color(r, g, b))
    }
}

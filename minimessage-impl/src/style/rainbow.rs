use std::{borrow::Cow, str::FromStr};

use crate::style::{Color, SpecialError, TryFromDescriptors};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum RainbowError {
    #[error("Expected an integer, found {_0} (`!` is stripped)")]
    ExpectedInt(String),

    #[error("Empty expressions are not allowed")]
    EmptyString,

    #[error("Integer was overflown")]
    IntegerOverflow,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Rainbow {
    pub inverted: bool,
    pub offset: usize,
}

impl Rainbow {
    pub fn text(self, text: &str) -> Vec<(Color, char)> {
        let char_count = text.chars().count();
        if char_count == 0 {
            return Vec::new();
        }

        text.chars()
            .enumerate()
            .map(|(i, ch)| {
                let mut t = if char_count > 1 {
                    i as f32 / (char_count - 1) as f32
                } else {
                    0.0
                };

                if self.inverted {
                    t = 1.0 - t;
                }

                let hue = (t + self.offset as f32) * 0.75;

                (Color::from_hsv(hue, 0.75, 1.0), ch)
            })
            .collect()
    }
}

impl FromStr for Rainbow {
    type Err = RainbowError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(RainbowError::EmptyString);
        }

        let mut chars = s.chars().peekable();
        let mut inverted = false;
        let mut offset = 0usize;

        if chars.peek().is_some_and(|c| *c == '!') {
            chars.next();
            inverted = true;
        }

        for c in chars {
            if !c.is_ascii_digit() {
                return Err(RainbowError::ExpectedInt(
                    s.strip_prefix('!').unwrap_or(s).to_owned(),
                ));
            }

            offset = offset
                .checked_mul(10)
                .and_then(|v| v.checked_add(c as usize - '0' as usize))
                .ok_or(RainbowError::IntegerOverflow)?;
        }

        Ok(Self { inverted, offset })
    }
}

impl TryFromDescriptors for Rainbow {
    fn try_from_descriptors(args: Vec<Cow<'_, str>>) -> Result<Self, SpecialError> {
        match args.as_slice() {
            [] => Ok(Rainbow::default()),
            [rainbow_fmt] => Ok(Rainbow::from_str(rainbow_fmt)?),
            args => Err(SpecialError::TooManyArguments(
                args.iter().map(ToString::to_string).collect(),
            )),
        }
    }
}

use std::{borrow::Cow, num::ParseIntError, str::FromStr};

use strum::{EnumDiscriminants, EnumString};

use crate::style::{
    TryFromDescriptors,
    click_event::ClickEvent,
    color::{Color, ParseColorError},
    hover_event::HoverEvent,
    rainbow::{Rainbow, RainbowError},
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum SpecialError {
    #[error("missing the special's event type")]
    MissingSpecialType,

    #[error("unknown special `{_0}`")]
    UnknownTag(String),

    NotFound(#[from] strum::ParseError),
    ParseInt(#[from] ParseIntError),
    ParseColor(#[from] ParseColorError),

    Snbt(#[from] fastsnbt::error::Error),

    #[error("Modifier {_0} is not valid")]
    InvalidModifier(String),

    #[error("The argument {_0} is missing")]
    MissingArgument(&'static str),

    #[error("The arguments are invalid: {_0:?}")]
    TooManyArguments(Vec<String>),

    #[error("An invalid event was reached. Should be unreachable.")]
    EmptyEventReached,

    RainbowError(#[from] RainbowError),
}

#[derive(Clone, Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumString))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum Special {
    Click(ClickEvent),
    Hover(HoverEvent),
    Color(Color),
    Rainbow(Rainbow),
}

impl Special {
    pub fn from_descriptor(
        tag: &str,
        descriptors: Vec<Cow<'_, str>>,
    ) -> Result<Self, SpecialError> {
        match SpecialDiscriminants::from_str(tag)? {
            SpecialDiscriminants::Click => Ok(Special::Click(ClickEvent::try_from_descriptors(
                descriptors,
            )?)),
            SpecialDiscriminants::Hover => Ok(Special::Hover(HoverEvent::try_from_descriptors(
                descriptors,
            )?)),
            SpecialDiscriminants::Color => {
                Ok(Special::Color(Color::try_from_descriptors(descriptors)?))
            },
            SpecialDiscriminants::Rainbow => Ok(Special::Rainbow(Rainbow::try_from_descriptors(
                descriptors,
            )?)),
        }
    }
}

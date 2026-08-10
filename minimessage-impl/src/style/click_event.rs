use std::{borrow::Cow, str::FromStr};

use strum::{EnumDiscriminants, EnumString};

use crate::style::{SpecialError, TryFromDescriptors};

#[derive(Default, Clone, Debug, EnumDiscriminants, EnumString)]
#[strum_discriminants(derive(EnumString))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum ClickEvent {
    OpenUrl(String),
    RunCommand(String),
    SuggestCommand(String),
    CopyToClipboard(String),
    #[default]
    __Empty,
}

impl TryFromDescriptors for ClickEvent {
    fn try_from_descriptors(args: Vec<Cow<'_, str>>) -> Result<Self, SpecialError> {
        let mut iter = args.into_iter();
        let event = iter.next().ok_or(SpecialError::MissingSpecialType)?;
        let mut args = iter.collect::<Vec<_>>();

        match ClickEventDiscriminants::from_str(&event)? {
            ClickEventDiscriminants::OpenUrl => Ok(Self::OpenUrl(
                args.pop()
                    .ok_or(SpecialError::MissingArgument("url"))?
                    .into(),
            )),
            ClickEventDiscriminants::RunCommand => Ok(Self::RunCommand(
                args.pop()
                    .ok_or(SpecialError::MissingArgument("command"))?
                    .into(),
            )),
            ClickEventDiscriminants::SuggestCommand => Ok(Self::SuggestCommand(
                args.pop()
                    .ok_or(SpecialError::MissingArgument("command"))?
                    .into(),
            )),
            ClickEventDiscriminants::CopyToClipboard => Ok(Self::CopyToClipboard(
                args.pop()
                    .ok_or(SpecialError::MissingArgument("contents"))?
                    .into(),
            )),
            ClickEventDiscriminants::__Empty => Err(SpecialError::EmptyEventReached),
        }
    }
}

use std::{borrow::Cow, collections::HashMap, str::FromStr};

use strum::{EnumDiscriminants, EnumString};

use crate::style::{Components, Enchantments, Item, TryFromDescriptors, special::SpecialError};

#[derive(Default, Clone, Debug, EnumDiscriminants, EnumString)]
#[strum_discriminants(derive(EnumString))]
#[strum_discriminants(strum(serialize_all = "snake_case"))]
pub enum HoverEvent {
    ShowEntity {
        entity_type: String,
        id: String,
        name: Option<String>,
    },
    ShowItem(String),
    ShowText(String),
    #[default]
    __Empty,
}

impl TryFromDescriptors for HoverEvent {
    fn try_from_descriptors(descriptors: Vec<Cow<'_, str>>) -> Result<Self, SpecialError> {
        let mut iter = descriptors.into_iter();
        let event = iter.next().ok_or(SpecialError::MissingSpecialType)?;
        let args = iter.collect::<Vec<_>>();

        match HoverEventDiscriminants::from_str(&event)? {
            HoverEventDiscriminants::ShowText => Ok(Self::ShowText(
                args.into_iter()
                    .next()
                    .ok_or(SpecialError::MissingArgument("text"))?
                    .into(),
            )),
            HoverEventDiscriminants::ShowItem => match args.as_slice() {
                [] => Err(SpecialError::MissingArgument("item")),
                [id] => Ok(Self::ShowItem(id.to_string())),
                [id, count] => {
                    let item = Item::new(format!("minecraft:{id}"), count.parse()?);
                    Ok(Self::ShowItem(fastsnbt::to_string(&item)?))
                },
                [id, count, modifier_tag, modifier_value] => match &**modifier_tag {
                    "enchantments" => {
                        let levels: HashMap<String, i32> = fastsnbt::from_str(modifier_value)?;
                        let mut enchantments = Enchantments::default();
                        for (key, value) in levels {
                            enchantments
                                .levels
                                .insert(format!("minecraft:{key}"), value);
                        }
                        let item = Item::new_with_components(
                            format!("minecraft:{id}"),
                            count.parse()?,
                            Components { enchantments },
                        );
                        Ok(Self::ShowItem(fastsnbt::to_string(&item)?))
                    },
                    modifier => Err(SpecialError::InvalidModifier(modifier.to_owned())),
                },
                args => Err(SpecialError::TooManyArguments(
                    args.iter().map(|cow| cow.to_string()).collect(),
                )),
            },
            HoverEventDiscriminants::ShowEntity => {
                let mut args = args.into_iter();
                Ok(Self::ShowEntity {
                    entity_type: args
                        .next()
                        .ok_or(SpecialError::MissingArgument("entity_type"))?
                        .into(),
                    id: args
                        .next()
                        .ok_or(SpecialError::MissingArgument("id"))?
                        .into(),
                    name: args.next().map(String::from),
                })
            },
            HoverEventDiscriminants::__Empty => Err(SpecialError::EmptyEventReached),
        }
    }
}

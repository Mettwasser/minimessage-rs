pub mod click_event;
pub mod color;
pub mod hover_event;
pub mod rainbow;
pub mod special;
pub mod gradient_impl;

use std::{borrow::Cow, collections::HashMap};

use serde::Serialize;
use strum::EnumString;

pub use crate::style::{
    click_event::{ClickEvent, ClickEventDiscriminants},
    color::{Color, ParseColorError},
    hover_event::{HoverEvent, HoverEventDiscriminants},
    special::{Special, SpecialDiscriminants, SpecialError},
};

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum NamedColor {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
}

#[derive(Debug, Clone, Copy, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Decoration {
    Bold,
    Italic,
    Underlined,
    Strikethrough,
    Obfuscated,
}

pub trait TryFromDescriptors: Sized {
    fn try_from_descriptors(args: Vec<Cow<'_, str>>) -> Result<Self, SpecialError>;
}

#[derive(Debug, PartialEq, Serialize)]
struct Item {
    id: String,
    count: i32,
    components: Components,
}

impl Item {
    pub fn new(id: String, count: i32) -> Self {
        Item {
            id,
            count,
            components: Components::default(),
        }
    }

    pub fn new_with_components(id: String, count: i32, components: Components) -> Self {
        Item {
            id,
            count,
            components,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Default)]
struct Components {
    #[serde(rename = "minecraft:enchantments")]
    enchantments: Enchantments,
}

#[derive(Debug, PartialEq, Serialize, Default)]
struct Enchantments {
    levels: HashMap<String, i32>,
}

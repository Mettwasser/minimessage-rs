use minimessage_impl::style::{ClickEvent, HoverEvent, rainbow::Rainbow};
use pumpkin_plugin_api::{
    common::{NamedColor as PumpkinNamedColor, RgbColor},
    text::TextComponent,
};

use crate::{Component, ComponentColor, NamedColor};

fn map_named_color(color: NamedColor) -> PumpkinNamedColor {
    match color {
        NamedColor::Black => PumpkinNamedColor::Black,
        NamedColor::DarkBlue => PumpkinNamedColor::DarkBlue,
        NamedColor::DarkGreen => PumpkinNamedColor::DarkGreen,
        NamedColor::DarkAqua => PumpkinNamedColor::DarkAqua,
        NamedColor::DarkRed => PumpkinNamedColor::DarkRed,
        NamedColor::DarkPurple => PumpkinNamedColor::DarkPurple,
        NamedColor::Gold => PumpkinNamedColor::Gold,
        NamedColor::Gray => PumpkinNamedColor::Gray,
        NamedColor::DarkGray => PumpkinNamedColor::DarkGray,
        NamedColor::Blue => PumpkinNamedColor::Blue,
        NamedColor::Green => PumpkinNamedColor::Green,
        NamedColor::Aqua => PumpkinNamedColor::Aqua,
        NamedColor::Red => PumpkinNamedColor::Red,
        NamedColor::LightPurple => PumpkinNamedColor::LightPurple,
        NamedColor::Yellow => PumpkinNamedColor::Yellow,
        NamedColor::White => PumpkinNamedColor::White,
    }
}

fn apply_click(comp: TextComponent, event: ClickEvent) -> TextComponent {
    match event {
        ClickEvent::OpenUrl(url) => comp.click_open_url(&url),
        ClickEvent::RunCommand(cmd) => comp.click_run_command(&cmd),
        ClickEvent::SuggestCommand(cmd) => comp.click_suggest_command(&cmd),
        ClickEvent::CopyToClipboard(text) => comp.click_copy_to_clipboard(&text),

        ClickEvent::__Empty => unreachable!(),
    }
}

fn apply_hover(comp: TextComponent, hover: HoverEvent) -> TextComponent {
    match hover {
        HoverEvent::ShowText(text) => {
            let text_comp = to_pumpkin_component(&text);
            comp.hover_show_text(text_comp)
        },
        HoverEvent::ShowItem(item) => comp.hover_show_item(&item),
        HoverEvent::ShowEntity {
            entity_type,
            id,
            name,
        } => {
            let name_comp = name.map(|n| to_pumpkin_component(&n));
            comp.hover_show_entity(&entity_type, &id, name_comp)
        },

        HoverEvent::__Empty => unreachable!(),
    }
}

struct RainbowState {
    rainbow: Rainbow,
    char_count: usize,
    index: usize,
}

fn apply_rainbow(mut comp: TextComponent, state: &mut RainbowState, text: &str) -> TextComponent {
    for char in text.chars() {
        let color = state.rainbow.color_at(state.index, state.char_count);
        state.index += 1;
        let text_component = TextComponent::text(&char.to_string());

        comp = comp.add_child(text_component.color_rgb(RgbColor {
            r: color.0,
            g: color.1,
            b: color.2,
        }));
    }

    comp
}

fn to_pumpkin_component(input: &str) -> TextComponent {
    match crate::deserialize_into_component(input) {
        Ok(comp) => convert(&comp),
        Err(_) => TextComponent::text(input),
    }
}

fn rainbow_char_count(comp: &Component, include_root: bool) -> usize {
    if !include_root && (comp.color.is_some() || comp.rainbow.is_some()) {
        return 0;
    }

    comp.text.chars().count()
        + comp
            .children
            .iter()
            .map(|child| rainbow_char_count(child, false))
            .sum::<usize>()
}

fn convert_component(
    comp: &Component,
    inherited_rainbow: Option<&mut RainbowState>,
) -> TextComponent {
    let mut result = apply_style(TextComponent::text(""), comp);
    let mut own_rainbow = comp.rainbow.map(|rainbow| RainbowState {
        rainbow,
        char_count: rainbow_char_count(comp, true),
        index: 0,
    });
    let mut rainbow = if own_rainbow.is_some() {
        own_rainbow.as_mut()
    } else if comp.color.is_some() {
        None
    } else {
        inherited_rainbow
    };

    if let Some(state) = rainbow.as_deref_mut() {
        result = apply_rainbow(result, state, &comp.text);
    } else if !comp.text.is_empty() {
        result = result.add_text(&comp.text);
    }

    for child in &comp.children {
        result = result.add_child(convert_component(child, rainbow.as_deref_mut()));
    }

    result
}

fn apply_style(mut comp: TextComponent, from: &Component) -> TextComponent {
    if let Some(ref color) = from.color {
        comp = match color {
            ComponentColor::Named(named) => comp.color_named(map_named_color(*named)),
            ComponentColor::Rgb(r, g, b) => comp.color_rgb(RgbColor {
                r: *r,
                g: *g,
                b: *b,
            }),
        };
    }

    if from.bold {
        comp = comp.bold(true);
    }
    if from.italic {
        comp = comp.italic(true);
    }
    if from.underlined {
        comp = comp.underlined(true);
    }
    if from.strikethrough {
        comp = comp.strikethrough(true);
    }
    if from.obfuscated {
        comp = comp.obfuscated(true);
    }

    if let Some(event) = from.click_event.clone() {
        comp = apply_click(comp, event);
    }

    if let Some(hover) = from.hover_event.clone() {
        comp = apply_hover(comp, hover);
    }

    comp
}

pub fn convert(comp: &Component) -> TextComponent {
    convert_component(comp, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rainbow_count_crosses_decorations_and_excludes_colors() {
        let component = crate::deserialize_into_component(
            "<rainbow>a<bold>b</bold><blue>excluded</blue>c</rainbow>",
        )
        .unwrap();
        let rainbow = &component.children[0];

        assert_eq!(rainbow_char_count(rainbow, true), 3);
    }
}

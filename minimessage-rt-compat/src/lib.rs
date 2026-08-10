use minimessage_impl::style::{ClickEvent, HoverEvent, rainbow::Rainbow};
use minimessage_runtime::{Component, ComponentColor, NamedColor};
use pumpkin_plugin_api::{
    common::{NamedColor as PumpkinNamedColor, RgbColor},
    text::TextComponent,
};

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

fn apply_click(comp: &TextComponent, event: ClickEvent) {
    match event {
        ClickEvent::OpenUrl(url) => {
            comp.click_open_url(&url);
        },
        ClickEvent::RunCommand(cmd) => {
            comp.click_run_command(&cmd);
        },
        ClickEvent::SuggestCommand(cmd) => {
            comp.click_suggest_command(&cmd);
        },
        ClickEvent::CopyToClipboard(text) => {
            comp.click_copy_to_clipboard(&text);
        },

        ClickEvent::__Empty => unreachable!(),
    }
}

fn apply_hover(comp: &TextComponent, hover: HoverEvent) {
    match hover {
        HoverEvent::ShowText(text) => {
            let text_comp = to_pumpkin_component(&text);
            comp.hover_show_text(text_comp);
        },
        HoverEvent::ShowItem(item) => {
            comp.hover_show_item(&item);
        },
        HoverEvent::ShowEntity {
            entity_type,
            id,
            name,
        } => {
            let name_comp = name.map(|n| to_pumpkin_component(&n));
            comp.hover_show_entity(&entity_type, &id, name_comp);
        },

        HoverEvent::__Empty => unreachable!(),
    }
}

fn apply_rainbow(comp: &TextComponent, rainbow: Rainbow, text: &str) {
    for (color, char) in rainbow.text(text) {
        let text_component = TextComponent::text(&char.to_string());
        text_component.color_rgb(RgbColor {
            r: color.0,
            g: color.1,
            b: color.2,
        });
        comp.add_child(text_component);
    }
}

fn to_pumpkin_component(input: &str) -> TextComponent {
    match minimessage_runtime::deserialize(input) {
        Ok(comp) => convert(&comp),
        Err(_) => TextComponent::text(input),
    }
}

fn convert_child(comp: &Component, result: &TextComponent) {
    if let Some(rainbow) = comp.rainbow {
        let text = flatten_text(comp);
        apply_rainbow(result, rainbow, &text);
        return;
    }

    if !comp.text.is_empty() {
        result.add_text(&comp.text);
    }

    apply_style(result, comp);

    for child in &comp.children {
        convert_child(child, result);
    }
}

fn flatten_text(comp: &Component) -> String {
    let mut text = comp.text.clone();
    for child in &comp.children {
        text.push_str(&flatten_text(child));
    }
    text
}

fn apply_style(comp: &TextComponent, from: &Component) {
    if let Some(ref color) = from.color {
        match color {
            ComponentColor::Named(named) => {
                comp.color_named(map_named_color(*named));
            },
            ComponentColor::Rgb(r, g, b) => {
                comp.color_rgb(RgbColor {
                    r: *r,
                    g: *g,
                    b: *b,
                });
            },
        }
    }

    if from.bold {
        comp.bold(true);
    }
    if from.italic {
        comp.italic(true);
    }
    if from.underlined {
        comp.underlined(true);
    }
    if from.strikethrough {
        comp.strikethrough(true);
    }
    if from.obfuscated {
        comp.obfuscated(true);
    }

    if let Some(event) = from.click_event.clone() {
        apply_click(comp, event);
    }

    if let Some(hover) = from.hover_event.clone() {
        apply_hover(comp, hover);
    }
}

pub fn convert(comp: &Component) -> TextComponent {
    let result = TextComponent::text("");

    apply_style(&result, comp);

    if !comp.text.is_empty() {
        if let Some(rainbow) = comp.rainbow {
            apply_rainbow(&result, rainbow, &comp.text);
        } else {
            result.add_text(&comp.text);
        }
    }

    for child in &comp.children {
        convert_child(child, &result);
    }

    result
}

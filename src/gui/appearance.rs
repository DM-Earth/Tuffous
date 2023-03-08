use iced::{
    alignment,
    widget::{container, text, Text},
    Font, Theme,
};

const ICONS: Font = Font::External {
    name: "Nerd Icons",
    bytes: include_bytes!("../../fonts/nerd_font.ttf"),
};

pub fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
        .size(20)
}

pub struct TagStyle {}

impl container::StyleSheet for TagStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(style.palette().text),
            background: None,
            border_radius: 100.0,
            border_width: 1.0,
            border_color: style.palette().text,
        }
    }
}

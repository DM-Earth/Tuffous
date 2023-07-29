use iced::{
    alignment,
    widget::{container, text, Text},
    Font, Theme,
};
use once_cell::sync::Lazy;

const ICONS: Font = Font::with_name("Symbols Nerd Font");

pub static FONT: Lazy<Option<Font>> = Lazy::new(|| {
    let config = super::config::ConfigInstance::get();

    if let Some(font) = config.fonts.get(0) {
        Some(Font::with_name(Box::leak(Box::new(font.to_string()))))
    } else {
        None
    }
});

pub fn icon(unicode: char) -> Text<'static> {
    text(unicode.to_string())
        .font(ICONS)
        .width(20)
        .horizontal_alignment(alignment::Horizontal::Center)
        .size(20)
}

pub struct TagStyle;

impl container::StyleSheet for TagStyle {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            text_color: Some(StyleSheet::from_theme(style).gray),
            background: None,
            border_radius: 100.0.into(),
            border_width: 1.0,
            border_color: StyleSheet::from_theme(style).gray,
        }
    }
}

pub struct StyleSheet {
    pub flag: iced::Color,
    pub star: iced::Color,
    pub checkbox: iced::Color,
    pub gray: iced::Color,
    pub green: iced::Color,
    pub blue_green: iced::Color,
}

impl StyleSheet {
    pub fn from_theme(theme: &iced::Theme) -> StyleSheet {
        match theme {
            Theme::Dark => StyleSheet {
                flag: iced::Color::from_rgb(0.843, 0.251, 0.267),
                star: iced::Color::from_rgb(1.0, 0.843, 0.0),
                checkbox: theme.palette().primary,
                gray: iced::Color::from_rgb(0.5, 0.5, 0.5),
                green: iced::Color::from_rgb(0.196, 0.8039, 0.196),
                blue_green: iced::Color::from_rgb(0.0, 0.545, 0.545),
            },
            _ => StyleSheet {
                flag: iced::Color::from_rgb(0.86, 0.078, 0.235),
                star: iced::Color::from_rgb(1.0, 0.843, 0.0),
                checkbox: iced::Color::from_rgb(0.07, 0.23, 0.591),
                gray: iced::Color::from_rgb(0.5, 0.5, 0.5),
                green: iced::Color::from_rgb(0.086, 0.596, 0.1686),
                blue_green: iced::Color::from_rgb(0.0, 0.502, 0.502),
            },
        }
    }
}

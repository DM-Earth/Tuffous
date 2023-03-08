use iced::{
    alignment,
    widget::{text, Text},
    Font,
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

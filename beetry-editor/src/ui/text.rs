pub fn text_width_from(text: &str, font_size: u8) -> f64 {
    const AVG_CHAR_WIDTH_RATIO: f64 = 0.6;
    f64::from(font_size) * text.chars().count() as f64 * AVG_CHAR_WIDTH_RATIO
}

pub const fn font_family() -> &'static str {
    "system-ui, -apple-system, sans-serif"
}

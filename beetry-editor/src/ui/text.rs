pub(crate) fn text_width_from(text: &str, font_size: u8) -> f64 {
    const AVG_CHAR_WIDTH_RATIO: f64 = 0.6;
    font_size as f64 * text.chars().count() as f64 * AVG_CHAR_WIDTH_RATIO
}

pub(crate) fn font_family() -> &'static str {
    "system-ui, -apple-system, sans-serif"
}

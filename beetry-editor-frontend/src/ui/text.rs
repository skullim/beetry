pub const FONT_SIZE_SMALL: u8 = 10;
pub const FONT_SIZE_NORMAL: u8 = 12;

#[expect(
    clippy::cast_precision_loss,
    reason = "number of text characters is reasonably small"
)]
pub fn text_width_from(text: &str, font_size: u8) -> f64 {
    const AVG_CHAR_WIDTH_RATIO: f64 = 0.6;
    f64::from(font_size) * text.chars().count() as f64 * AVG_CHAR_WIDTH_RATIO
}

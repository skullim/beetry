pub const FONT_SIZE_SMALL: u8 = 10;
pub const FONT_SIZE_NORMAL: u8 = 12;

const AVG_CHAR_WIDTH_RATIO: f64 = 0.55;

#[expect(
    clippy::cast_precision_loss,
    reason = "number of text characters is reasonably small"
)]
pub fn text_width_from(text: &str, font_size: u8) -> f64 {
    char_pixel_width(font_size) * text.chars().count() as f64
}

pub fn char_pixel_width(font_size: u8) -> f64 {
    f64::from(font_size) * AVG_CHAR_WIDTH_RATIO
}

pub fn truncate_label(label: &str, font_size: u8, width: f64) -> String {
    let max_count = max_chars_count_per_width(font_size, width);
    if max_count > label.len() {
        label.to_string()
    } else {
        const ELLIPSIS: &str = "..";
        label
            .chars()
            .take(max_count.saturating_sub(ELLIPSIS.len()))
            .chain(ELLIPSIS.chars())
            .collect()
    }
}

#[expect(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    reason = "floored non-negative pixel width converted to character count"
)]
fn max_chars_count_per_width(font_size: u8, width: f64) -> usize {
    (width / char_pixel_width(font_size)) as usize
}

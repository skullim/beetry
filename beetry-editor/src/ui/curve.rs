use crate::definitions::Point;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Curve;

impl Curve {
    pub fn calculate_vertical(start: &Point, end: &Point) -> String {
        let control_offset = ((end.y - start.y).abs() * 0.5).max(50.0);

        format!(
            "M{},{} C{},{} {},{} {},{}",
            start.x,
            start.y,
            start.x,
            start.y + control_offset,
            end.x,
            end.y - control_offset,
            end.x,
            end.y
        )
    }

    pub fn calculate_horizontal(start: &Point, end: &Point) -> String {
        let control_offset = ((end.x - start.x).abs() * 0.3).max(80.0);

        format!(
            "M{},{} C{},{} {},{} {},{}",
            start.x,
            start.y,
            start.x + control_offset.copysign(end.x - start.x),
            start.y,
            end.x - control_offset.copysign(end.x - start.x),
            end.y,
            end.x,
            end.y
        )
    }
}

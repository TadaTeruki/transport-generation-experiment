use crate::Site2D;

pub fn get_cross(
    line_a_start: Site2D,
    line_a_end: Site2D,
    line_b_start: Site2D,
    line_b_end: Site2D,
) -> Option<(Site2D, bool)> {
    let a1 = line_a_end.y - line_a_start.y;
    let b1 = line_a_start.x - line_a_end.x;
    let c1 = a1 * line_a_start.x + b1 * line_a_start.y;

    let a2 = line_b_end.y - line_b_start.y;
    let b2 = line_b_start.x - line_b_end.x;
    let c2 = a2 * line_b_start.x + b2 * line_b_start.y;

    let determinant = a1 * b2 - a2 * b1;

    if determinant == 0.0 {
        return None;
    }

    let x = (b2 * c1 - b1 * c2) / determinant;
    let y = (a1 * c2 - a2 * c1) / determinant;

    let passing = (x - line_a_start.x) * (x - line_a_end.x) <= 0.0
        && (y - line_a_start.y) * (y - line_a_end.y) <= 0.0
        && (x - line_b_start.x) * (x - line_b_end.x) <= 0.0
        && (y - line_b_start.y) * (y - line_b_end.y) <= 0.0;

    Some((Site2D { x, y }, passing))
}

use skrifa::outline::{pen::ControlBoundsPen, OutlinePen};

use crate::{GlyphOutline, QualitySettings};

type Point = (f32, f32);

pub(crate) struct GlyphOutlineBuilder {
    bounds: ControlBoundsPen,
    contours: Vec<Vec<u32>>,
    current_point: (f32, f32),
    font_height: f32,
    index: u32,
    points: Vec<Point>,
    quality: QualitySettings,
    start_index: u32,
}

impl GlyphOutlineBuilder {
    pub(crate) fn new(font_height: f32, quality: QualitySettings) -> Self {
        Self {
            bounds: ControlBoundsPen::default(),
            contours: Vec::new(),
            current_point: (0f32, 0f32),
            font_height,
            index: 0,
            points: Vec::new(),
            quality,
            start_index: 0,
        }
    }

    pub(crate) fn get_glyph_outline(&mut self) -> GlyphOutline {
        GlyphOutline {
            contours: self.contours.clone(),
            points: self.points.clone(),
        }
    }

    pub(crate) fn bounds(&self) -> Option<skrifa::metrics::BoundingBox> {
        self.bounds.bounding_box()
    }

    fn add_point(&mut self, point: (f32, f32)) {
        self.current_point = point;

        // Normalize the coordinates of each glyph into the range `0..=1`.
        self.points
            .push((point.0 / self.font_height, point.1 / self.font_height));
        self.index += 1;
    }
}

impl OutlinePen for GlyphOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.bounds.move_to(x, y);
        self.start_index = self.index;
        self.contours.push(vec![self.start_index]);
        self.add_point((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.bounds.line_to(x, y);
        self.contours.last_mut().unwrap().push(self.index);
        self.add_point((x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.bounds.quad_to(x1, y1, x, y);
        let p1 = (x1, y1);
        let p2 = (x, y);

        // We deliberately omit the first interpolation segment, because the
        // start point of the curve is already in the list.
        for step in 1..=self.quality.quad_interpolation_steps {
            let t = step as f32 / self.quality.quad_interpolation_steps as f32;
            let p = point_on_quad(&self.current_point, &p1, &p2, t);
            self.line_to(p.0, p.1);
        }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.bounds.curve_to(x1, y1, x2, y2, x, y);
        let p1 = (x1, y1);
        let p2 = (x2, y2);
        let p3 = (x, y);

        // We deliberately omit the first interpolation segment, because the
        // start point of the curve is already in the list.
        for step in 1..=self.quality.cubic_interpolation_steps {
            let t = step as f32 / self.quality.cubic_interpolation_steps as f32;
            let p = point_on_cubic(&self.current_point, &p1, &p2, &p3, t);
            self.line_to(p.0, p.1);
        }
    }

    fn close(&mut self) {
        // Pens may close a contour either explicitly or implicitly. Only remove
        // the final vertex when it actually duplicates the starting vertex.
        let current_contour = self
            .contours
            .last_mut()
            .expect("Contour has no start point.");

        if current_contour.len() > 1
            && self.points.last() == self.points.get(self.start_index as usize)
        {
            current_contour.pop();
            self.points.pop();
            self.index -= 1;
        }
        current_contour.push(self.start_index);
    }
}

/// Returns the 2D coordinates of a point on a straight line
/// with the normalized distance `t` from the start.
///
/// Arguments:
///
/// * `a`: The start point of the line.
/// * `b`: The end point of the line.
/// * `t`: The normalized distance from the start in the range `0..=1`.
///
/// Returns:
///
/// The coordinates of the given point.
fn point_on_line(a: &Point, b: &Point, t: f32) -> Point {
    (a.0 - (a.0 - b.0) * t, a.1 - (a.1 - b.1) * t)
}

/// Returns the 2D coordinates of a point on a quadratic spline
/// with the control parameter `t`.
///
/// Arguments:
///
/// * `p0`: The start point of the curve.
/// * `p1`: The first control point.
/// * `p2`: The end point of the curve.
/// * `t`: The control parameter in the range `0..=1`.
///
/// Returns:
///
/// The coordinates of the given point.
fn point_on_quad(p0: &Point, p1: &Point, p2: &Point, t: f32) -> Point {
    let a = point_on_line(p0, p1, t);
    let b = point_on_line(p1, p2, t);
    point_on_line(&a, &b, t)
}

/// Returns the 2D coordinates of a point on a cubic spline
/// with the control parameter `t`.
///
/// Arguments:
///
/// * `p0`: The start point of the curve.
/// * `p1`: The first control point.
/// * `p2`: The end point of the curve.
/// * `t`: The control parameter in the range `0..=1`.
///
/// Returns:
///
/// The coordinates of the given point.
fn point_on_cubic(p0: &Point, p1: &Point, p2: &Point, p3: &Point, t: f32) -> Point {
    let a = point_on_quad(p0, p1, p2, t);
    let b = point_on_quad(p1, p2, p3, t);
    point_on_line(&a, &b, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_and_implicit_closure_preserve_all_triangle_vertices() {
        for explicit in [false, true] {
            let mut builder = GlyphOutlineBuilder::new(1.0, QualitySettings::default());
            for offset in [0.0, 2.0] {
                builder.move_to(offset, 0.0);
                builder.line_to(offset + 1.0, 0.0);
                builder.line_to(offset, 1.0);
                if explicit {
                    builder.line_to(offset, 0.0);
                }
                builder.close();
            }
            let outline = builder.get_glyph_outline();
            assert_eq!(
                outline.points,
                vec![
                    (0.0, 0.0),
                    (1.0, 0.0),
                    (0.0, 1.0),
                    (2.0, 0.0),
                    (3.0, 0.0),
                    (2.0, 1.0),
                ]
            );
            assert_eq!(outline.contours, vec![vec![0, 1, 2, 0], vec![3, 4, 5, 3]]);
        }
    }

    #[test]
    fn cubic_bounds_include_control_points_and_closure_keeps_endpoint() {
        let mut builder = GlyphOutlineBuilder::new(1.0, QualitySettings::default());
        builder.move_to(0.0, 0.0);
        builder.curve_to(0.0, 2.0, 2.0, 2.0, 2.0, 0.0);
        builder.close();
        let outline = builder.get_glyph_outline();
        assert_eq!(outline.points.last(), Some(&(2.0, 0.0)));
        assert_eq!(outline.contours[0], vec![0, 1, 2, 3, 0]);
        let bounds = builder.bounds().unwrap();
        assert_eq!(
            (bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max),
            (0.0, 0.0, 2.0, 2.0)
        );
    }
}

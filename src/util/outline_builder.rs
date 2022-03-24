use ttf_parser::OutlineBuilder;

use crate::{GlyphOutline, QualitySettings};

type Point = (f32, f32);

pub(crate) struct GlyphOutlineBuilder {
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

    fn add_point(&mut self, point: (f32, f32)) {
        self.current_point = point;

        // Normalize the coordinates of each glyph into the range `0..=1`.
        self.points
            .push((point.0 / self.font_height, point.1 / self.font_height));
        self.index += 1;
    }
}

impl OutlineBuilder for GlyphOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start_index = self.index;
        self.contours.push(vec![self.start_index]);
        self.add_point((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.contours.last_mut().unwrap().push(self.index);
        self.add_point((x, y));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
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
        // The last point is a duplicate so we remove it.
        // At this point there should be at least two points in the contour.
        let current_contour = self
            .contours
            .last_mut()
            .expect("Contour has no start point.");

        current_contour.pop();
        current_contour.push(self.start_index);
        self.points.pop();
        self.index -= 1;
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
    let a = point_on_line(&p0, &p1, t);
    let b = point_on_line(&p1, &p2, t);
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

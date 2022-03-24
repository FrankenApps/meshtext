type Point = (f32, f32);

/// The internal representation of a rasterized glyph outline.
pub(crate) struct GlyphOutline {
    /// The indices that form closed contours of points.
    pub contours: Vec<Vec<u32>>,

    /// A point cloud that contains one or more contours.
    pub points: Vec<Point>,
}

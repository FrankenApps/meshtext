use ttf_parser::{GlyphId, OutlineBuilder, Rect};

/// Common methods of a [ttf_parser::Face] or `OwnedFace` that
/// are used within the crate.
pub trait FontFace {
    /// Computes glyph's horizontal advance.
    ///
    /// This method is affected by variation axes.
    ///
    /// Returns:
    ///
    /// The horizontal advance of the glyph.
    fn glyph_hor_advance(&self, glyph_id: GlyphId) -> Option<u16>;

    /// Resolves a Glyph ID for a code point.
    ///
    /// All subtable formats except Mixed Coverage (8) are supported.
    ///
    /// If you need a more low-level control, prefer `Face::tables().cmap`.
    ///
    /// Returns:
    ///
    /// The [GlyphId] or `None` when the glyph is not found.
    fn glyph_index(&self, code_point: char) -> Option<GlyphId>;

    /// Computes the face's height.
    ///
    /// This method is affected by variation axes.
    ///
    /// Returns:
    ///
    /// The line height.
    fn height(&self) -> i16;

    /// Outlines a glyph and returns its tight bounding box.
    ///
    /// **Warning**: since `ttf-parser` is a pull parser,
    /// `OutlineBuilder` will emit segments even when outline is partially malformed.
    /// You must check `outline_glyph()` result before using
    /// `OutlineBuilder`'s output.
    ///
    /// `gvar`, `glyf`, `CFF` and `CFF2` tables are supported.
    /// And they will be accesses in this specific order.
    ///
    /// This method is affected by variation axes.
    ///
    /// Returns `None` when glyph has no outline or on error.
    ///
    /// # Example
    ///
    /// ```
    /// use std::fmt::Write;
    /// use ttf_parser;
    ///
    /// struct Builder(String);
    ///
    /// impl ttf_parser::OutlineBuilder for Builder {
    ///     fn move_to(&mut self, x: f32, y: f32) {
    ///         write!(&mut self.0, "M {} {} ", x, y).unwrap();
    ///     }
    ///
    ///     fn line_to(&mut self, x: f32, y: f32) {
    ///         write!(&mut self.0, "L {} {} ", x, y).unwrap();
    ///     }
    ///
    ///     fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
    ///         write!(&mut self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap();
    ///     }
    ///
    ///     fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
    ///         write!(&mut self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap();
    ///     }
    ///
    ///     fn close(&mut self) {
    ///         write!(&mut self.0, "Z ").unwrap();
    ///     }
    /// }
    ///
    /// let data = std::fs::read("assets/font/FiraMono-Regular.ttf").unwrap();
    /// let face = ttf_parser::Face::parse(&data, 0).unwrap();
    /// let mut builder = Builder(String::new());
    /// let bbox = face.outline_glyph(ttf_parser::GlyphId(36), &mut builder).unwrap();
    /// assert_eq!(builder.0, "M 161 176 L 106 0 L 20 0 L 245 689 L 355 689 L 579 0 L 489 0 \
    ///                        L 434 176 L 161 176 Z M 411 248 L 298 615 L 184 248 L 411 248 Z ");
    /// assert_eq!(bbox, ttf_parser::Rect { x_min: 20, y_min: 0, x_max: 579, y_max: 689 });
    /// ```
    fn outline_glyph(&self, glyph_id: GlyphId, builder: &mut dyn OutlineBuilder) -> Option<Rect>;
}

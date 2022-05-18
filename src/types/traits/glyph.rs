use crate::error::MeshTextError;

/// A single character from a font.
pub trait Glyph<T> {
    /// Generates a mesh for a single character.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that will be transformed into a mesh.
    /// * `flat`: Wether the mesh is flat or has an extent of `1` unit in the z-axis.
    /// * `transform`: The optional homogenous 4x4 transformation matrix that will be
    /// applied to each vertex of the mesh.
    ///
    /// Returns:
    ///
    /// The mesh for the given glyph.
    ///
    /// # Example
    ///
    /// ```rust
    /// use meshtext::{MeshGenerator, MeshText, Glyph};
    ///
    /// let font_data = include_bytes!("../../../assets/font/FiraMono-Regular.ttf");
    /// let mut generator = MeshGenerator::new(font_data);
    /// let result: MeshText = generator
    ///     .generate_glyph('A', true, None)
    ///     .expect("Failed to generate mesh.");
    /// ```
    fn generate_glyph(
        &mut self,
        glyph: char,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<T, Box<dyn MeshTextError>>;

    /// Generates a two-dimensional mesh for a single character.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that will be transformed into a mesh.
    /// * `transform`: The optional homogenous 3x3 transformation matrix that will be
    /// applied to each vertex of the mesh.
    ///
    /// Returns:
    ///
    /// The mesh for the given glyph.
    ///
    /// # Example
    ///
    /// ```rust
    /// use meshtext::{MeshGenerator, MeshText, Glyph};
    ///
    /// let font_data = include_bytes!("../../../assets/font/FiraMono-Regular.ttf");
    /// let mut generator = MeshGenerator::new(font_data);
    /// let result: MeshText = generator
    ///     .generate_glyph_2d('A', None)
    ///     .expect("Failed to generate mesh.");
    /// ```
    fn generate_glyph_2d(
        &mut self,
        glyph: char,
        transform: Option<&[f32; 9]>,
    ) -> Result<T, Box<dyn MeshTextError>>;
}

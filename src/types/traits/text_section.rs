use crate::error::MeshTextError;

/// A section of text.
pub trait TextSection<T> {
    /// Generates a mesh for a section of text.
    ///
    /// Arguments:
    ///
    /// * `text`: The text section that will be transformed into a mesh.
    /// * `flat`: Wether the mesh is flat or has an extent of `1` unit in the z-axis
    /// * `transform`: The optional homogenous 4x4 transformation matrix that will be
    /// applied to each vertex of the mesh.
    ///
    /// Returns:
    ///
    /// The mesh for the given text section.
    ///
    /// # Example
    ///
    /// ```rust
    /// use meshtext::{MeshGenerator, IndexedMeshText, TextSection};
    ///
    /// let font_data = include_bytes!("../../../assets/font/FiraMono-Regular.ttf");
    /// let mut generator = MeshGenerator::new(font_data);
    ///
    /// // Scale text section to 10% on z-axis.
    /// let transform = [1.0, 0.0, 0.0, 0.0,
    ///                  0.0, 1.0, 0.0, 0.0,
    ///                  0.0, 0.0, 0.1, 0.0,
    ///                  1.0, 0.0, 0.0, 1.0];
    ///
    /// let result: IndexedMeshText = generator
    ///     .generate_section(
    ///         "Hello World!",
    ///         false,
    ///         Some(&transform)
    ///     )
    ///     .expect("Failed to generate mesh.");
    /// ```
    fn generate_section(
        &mut self,
        text: &str,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<T, Box<dyn MeshTextError>>;

    /// Generates a mesh for a section of text.
    ///
    /// Arguments:
    ///
    /// * `text`: The text section that will be transformed into a mesh.
    /// * `flat`: Wether the mesh is flat or has an extent of `1` unit in the z-axis
    /// * `transform`: The optional homogenous 4x4 transformation matrix that will be
    /// applied to each vertex of the mesh.
    ///
    /// Returns:
    ///
    /// The mesh for the given text section.
    ///
    /// # Example
    ///
    /// ```rust
    /// use meshtext::{MeshGenerator, IndexedMeshText, TextSection};
    ///
    /// let font_data = include_bytes!("../../../assets/font/FiraMono-Regular.ttf");
    /// let mut generator = MeshGenerator::new(font_data);
    ///
    /// // Scale text section to 10%.
    /// let transform = [0.1, 0.0, 0.0,
    ///                  0.0, 0.1, 0.0,
    ///                  0.0, 0.0, 1.0];
    ///
    /// let result: IndexedMeshText = generator
    ///     .generate_section_2d(
    ///         "Hello World!",
    ///         Some(&transform)
    ///     )
    ///     .expect("Failed to generate mesh.");
    /// ```
    fn generate_section_2d(
        &mut self,
        text: &str,
        transform: Option<&[f32; 9]>,
    ) -> Result<T, Box<dyn MeshTextError>>;
}

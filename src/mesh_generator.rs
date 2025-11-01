use std::collections::HashMap;

use glam::{Mat3, Mat4, Vec2, Vec3, Vec3A};
use ttf_parser::GlyphId;

use crate::{
    error::MeshTextError,
    util::{
        glam_3d_vecs_from_raw_2d, glam_vecs_from_raw, mesh_to_flat_2d, mesh_to_indexed_flat_2d,
        raster_to_mesh, raster_to_mesh_indexed, text_mesh_from_data, text_mesh_from_data_2d,
        text_mesh_from_data_indexed, text_mesh_from_data_indexed_2d, GlyphOutlineBuilder,
    },
    BoundingBox, CacheType, FontFace, Glyph, IndexedMeshText, MeshText, QualitySettings,
    TextSection, TriangleMesh,
};

type Mesh = (Vec<Vec3A>, BoundingBox);
type Mesh2D = (Vec<Vec2>, BoundingBox);

type IndexedMesh = (Vec<u32>, Vec<Vec3A>, BoundingBox);
type IndexedMesh2D = (Vec<u32>, Vec<Vec2>, BoundingBox);

/// A [MeshGenerator] handles rasterizing individual glyphs.
///
/// Each [MeshGenerator] will handle exactly one font. This means
/// if you need support for multiple fonts, you will need to create
/// multiple instances (one per font) of this generator.
pub struct MeshGenerator<T>
where
    T: FontFace,
{
    /// Cached non-indexed glyphs are stored in this [HashMap].
    ///
    /// The key is the character itself, however because each
    /// character can have a 2D and a 3D variant, in the 3D
    /// variant each character is prefixed with an `_`.
    #[allow(unused)]
    pub(super) cache: HashMap<String, Mesh>,

    /// The current [FontFace].
    pub(super) font: T,

    /// Cached indexed glyphs are stored in this [HashMap].
    ///
    /// The key is the character itself, however because each
    /// character can have a 2D and a 3D variant, in the 3D
    /// variant each character is prefixed with an `_`.
    #[allow(unused)]
    pub(super) indexed_cache: HashMap<String, IndexedMesh>,

    /// Quality settings for generating the text meshes.
    pub(super) quality: QualitySettings,

    /// Controls wether the generator will automatically
    /// cache glyphs.
    #[allow(unused)]
    pub(super) use_cache: bool,
}

#[cfg(not(feature = "owned"))]
mod borrowed_mesh_generator {
    use std::collections::HashMap;

    use ttf_parser::GlyphId;

    use crate::{FontFace, MeshGenerator, QualitySettings};

    impl FontFace for ttf_parser::Face<'_> {
        /// Computes glyph's horizontal advance.
        ///
        /// This method is affected by variation axes.
        ///
        /// Returns:
        ///
        /// The horizontal advance of the glyph.
        fn glyph_hor_advance(&self, glyph_id: GlyphId) -> Option<u16> {
            ttf_parser::Face::glyph_hor_advance(self, glyph_id)
        }

        /// Resolves a Glyph ID for a code point.
        ///
        /// All sub-table formats except Mixed Coverage (8) are supported.
        ///
        /// If you need a more low-level control, prefer `Face::tables().cmap`.
        ///
        /// Returns:
        ///
        /// The [GlyphId] or `None` when the glyph is not found.
        fn glyph_index(&self, code_point: char) -> Option<GlyphId> {
            ttf_parser::Face::glyph_index(self, code_point)
        }

        /// Computes the face's height.
        ///
        /// This method is affected by variation axes.
        ///
        /// Returns:
        ///
        /// The line height.
        fn height(&self) -> i16 {
            ttf_parser::Face::height(self)
        }

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
        fn outline_glyph(
            &self,
            glyph_id: GlyphId,
            builder: &mut dyn ttf_parser::OutlineBuilder,
        ) -> Option<ttf_parser::Rect> {
            ttf_parser::Face::outline_glyph(self, glyph_id, builder)
        }
    }

    impl MeshGenerator<ttf_parser::Face<'_>> {
        /// Creates a new [MeshGenerator].
        ///
        /// Arguments:
        ///
        /// * `font`: The font that will be used for rasterizing.
        pub fn new(font: &'static [u8]) -> Self {
            let face =
                ttf_parser::Face::parse(font, 0).expect("Failed to generate font from data.");

            Self {
                cache: HashMap::new(),
                font: face,
                indexed_cache: HashMap::new(),
                quality: QualitySettings::default(),
                use_cache: true,
            }
        }

        /// Creates a new [MeshGenerator] with custom quality settings.
        ///
        /// Arguments:
        ///
        /// * `font`: The font that will be used for rasterizing.
        /// * `quality`: The [QualitySettings] that should be used.
        pub fn new_with_quality(font: &'static [u8], quality: QualitySettings) -> Self {
            let face =
                ttf_parser::Face::parse(font, 0).expect("Failed to generate font from data.");

            Self {
                cache: HashMap::new(),
                font: face,
                indexed_cache: HashMap::new(),
                quality,
                use_cache: true,
            }
        }

        /// Creates a new [MeshGenerator] with custom quality settings and no caching.
        ///
        /// Arguments:
        ///
        /// * `font`: The font that will be used for rasterizing.
        /// * `quality`: The [QualitySettings] that should be used.
        pub fn new_without_cache(font: &'static [u8], quality: QualitySettings) -> Self {
            let face =
                ttf_parser::Face::parse(font, 0).expect("Failed to generate font from data.");

            Self {
                cache: HashMap::new(),
                font: face,
                indexed_cache: HashMap::new(),
                quality,
                use_cache: false,
            }
        }
    }
}

#[cfg(feature = "owned")]
mod owned_mesh_generator {
    use crate::{FontFace, MeshGenerator, QualitySettings};
    use std::collections::HashMap;

    use owned_ttf_parser::{AsFaceRef, OwnedFace};

    impl FontFace for OwnedFace {
        /// Computes glyph's horizontal advance.
        ///
        /// This method is affected by variation axes.
        ///
        /// Returns:
        ///
        /// The horizontal advance of the glyph.
        fn glyph_hor_advance(&self, glyph_id: owned_ttf_parser::GlyphId) -> Option<u16> {
            self.as_face_ref().glyph_hor_advance(glyph_id)
        }

        /// Resolves a Glyph ID for a code point.
        ///
        /// All sub-table formats except Mixed Coverage (8) are supported.
        ///
        /// If you need a more low-level control, prefer `Face::tables().cmap`.
        ///
        /// Returns:
        ///
        /// The [GlyphId] or `None` when the glyph is not found.
        fn glyph_index(&self, code_point: char) -> Option<owned_ttf_parser::GlyphId> {
            self.as_face_ref().glyph_index(code_point)
        }

        /// Computes the face's height.
        ///
        /// This method is affected by variation axes.
        ///
        /// Returns:
        ///
        /// The line height.
        fn height(&self) -> i16 {
            self.as_face_ref().height()
        }

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
        fn outline_glyph(
            &self,
            glyph_id: owned_ttf_parser::GlyphId,
            builder: &mut dyn owned_ttf_parser::OutlineBuilder,
        ) -> Option<owned_ttf_parser::Rect> {
            self.as_face_ref().outline_glyph(glyph_id, builder)
        }
    }

    impl MeshGenerator<OwnedFace> {
        /// Creates a new [MeshGenerator].
        ///
        /// Arguments:
        ///
        /// * `font`: The font that will be used for rasterizing.
        pub fn new(font: Vec<u8>) -> Self {
            let face = OwnedFace::from_vec(font, 0).expect("Failed to generate font from data.");

            Self {
                cache: HashMap::new(),
                font: face,
                indexed_cache: HashMap::new(),
                quality: QualitySettings::default(),
                use_cache: true,
            }
        }

        /// Creates a new [MeshGenerator] with custom quality settings.
        ///
        /// Arguments:
        ///
        /// * `font`: The font that will be used for rasterizing.
        /// * `quality`: The [QualitySettings] that should be used.
        pub fn new_with_quality(font: Vec<u8>, quality: QualitySettings) -> Self {
            let face = OwnedFace::from_vec(font, 0).expect("Failed to generate font from data.");

            Self {
                cache: HashMap::new(),
                font: face,
                indexed_cache: HashMap::new(),
                quality,
                use_cache: true,
            }
        }

        /// Creates a new [MeshGenerator] with custom quality settings and no caching.
        ///
        /// Arguments:
        ///
        /// * `font`: The font that will be used for rasterizing.
        /// * `quality`: The [QualitySettings] that should be used.
        pub fn new_without_cache(font: Vec<u8>, quality: QualitySettings) -> Self {
            let face = OwnedFace::from_vec(font, 0).expect("Failed to generate font from data.");

            Self {
                cache: HashMap::new(),
                font: face,
                indexed_cache: HashMap::new(),
                quality,
                use_cache: false,
            }
        }
    }
}

impl<T> MeshGenerator<T>
where
    T: FontFace,
{
    /// Removes all stored glyphs from the internal cache.
    ///
    /// Normally it should not be necessary to do this manually unless your program
    /// cached so many glyphs, that memory consumption becomes an issue.
    ///
    /// This function does nothing if the current [MeshGenerator] does not have a cache.
    ///
    /// # Example
    ///
    /// ```rust
    /// use meshtext::MeshGenerator;
    ///
    /// let font_data = include_bytes!("../assets/font/FiraMono-Regular.ttf");
    /// let mut generator = MeshGenerator::new(font_data);
    ///
    /// generator.clear_cache();
    /// ```
    pub fn clear_cache(&mut self) {
        if self.use_cache {
            self.cache.clear();
        }
    }

    /// Gets a reference to the currently loaded [FontFace].
    /// 
    /// For example this allows reading out the [FontFace::height],
    /// or retrieving the [FontFace::glyph_index] of a certain [char].
    pub fn font(&self) -> &T {
        &self.font
    }

    /// Allows inserting a custom mesh into the internal cache that will be used for rendering
    /// the given `glyph`.
    ///
    /// Please note that this will not work if [MeshGenerator::new_without_cache] was used to
    /// construct this [MeshGenerator].
    ///
    /// Arguments:
    ///
    /// * `glyph`: The glyph that will be pre-cached. If the given glyph is already present in the
    ///   cache, it will be overwritten.
    /// * `flat`: Wether the flat or three-dimensional variant of the characters should be preloaded.
    ///   When set to `true` two coordinates per vertex must be specified in the `mesh`, otherwise three.
    /// * `mesh`: The mesh that should be used for rendering the given `glyph`.
    ///
    /// Note: For optimal results, make sure that all vertices of the `mesh` have coordinates in the range `0..1`.
    /// This ensures that the font size will be consistent with that of the generated glyphs.
    ///
    /// Returns:
    ///
    /// A [Result] indicating if the operation was successful.
    ///
    /// # Example
    ///
    /// ```rust
    /// use meshtext::{MeshGenerator, MeshText};
    ///
    /// let font_data = include_bytes!("../assets/font/FiraMono-Regular.ttf");
    /// let mut generator = MeshGenerator::new(font_data);
    ///
    /// let triangle: Vec<f32> = vec![
    ///     0.50, 0f32,
    ///     0.25, 0.57,
    ///     0f32, 0f32];
    /// let triangle_mesh = MeshText::new(triangle).unwrap();
    ///
    /// // Substitute the uppercase letter 'A' with a triangle for non-indexed flat meshes.
    /// generator.precache_custom_glyph('A', true, &triangle_mesh).unwrap();
    /// ```
    pub fn precache_custom_glyph<M>(
        &mut self,
        glyph: char,
        flat: bool,
        mesh: &M,
    ) -> Result<(), Box<dyn MeshTextError>>
    where
        M: TriangleMesh,
    {
        if let Some(indices) = mesh.indices() {
            if flat {
                self.indexed_cache.insert(
                    glyph.to_string(),
                    (
                        indices,
                        glam_3d_vecs_from_raw_2d(mesh.vertices()),
                        mesh.bbox(),
                    ),
                );
            } else {
                self.indexed_cache.insert(
                    format!("_{}", glyph),
                    (indices, glam_vecs_from_raw(mesh.vertices()), mesh.bbox()),
                );
            }
        } else if flat {
            self.cache.insert(
                glyph.to_string(),
                (glam_3d_vecs_from_raw_2d(mesh.vertices()), mesh.bbox()),
            );
        } else {
            self.cache.insert(
                format!("_{}", glyph),
                (glam_vecs_from_raw(mesh.vertices()), mesh.bbox()),
            );
        }

        Ok(())
    }

    /// Fills the internal cache of a [MeshGenerator] with the given characters.
    ///
    /// Arguments:
    ///
    /// * `glyphs`: The glyphs that will be pre-cached. Each character should appear exactly once.
    /// * `flat`: Wether the flat or three-dimensional variant of the characters should be preloaded.
    ///   If both variants should be pre-cached this function must be called twice with this parameter set
    ///   to `true` and `false`.
    /// * `cache`: An optional value that controls which cache will be filled. [None] means both caches will be filled.
    ///
    /// Returns:
    ///
    /// A [Result] indicating if the operation was successful.
    ///
    /// # Example
    ///
    /// ```rust
    /// use meshtext::MeshGenerator;
    ///
    /// let font_data = include_bytes!("../assets/font/FiraMono-Regular.ttf");
    /// let mut generator = MeshGenerator::new(font_data);
    ///
    /// let common = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789".to_string();
    ///
    /// // Pre-cache both flat and three-dimensional glyphs both for indexed and non-indexed meshes.
    /// generator.precache_glyphs(&common, false, None);
    /// generator.precache_glyphs(&common, true, None);
    /// ```
    pub fn precache_glyphs(
        &mut self,
        glyphs: &str,
        flat: bool,
        cache: Option<CacheType>,
    ) -> Result<(), Box<dyn MeshTextError>> {
        if let Some(cache_type) = cache {
            match cache_type {
                CacheType::Normal => {
                    for c in glyphs.chars() {
                        self.generate_glyph(c, flat, None)?;
                    }
                }
                CacheType::Indexed => {
                    for c in glyphs.chars() {
                        self.generate_glyph_indexed(c, flat, None)?;
                    }
                }
            }
        } else {
            // If no type is set explicitly, both variants will be pre-cached.
            for c in glyphs.chars() {
                self.generate_glyph(c, flat, None)?;
            }
            for c in glyphs.chars() {
                self.generate_glyph_indexed(c, flat, None)?;
            }
        }

        Ok(())
    }

    /// Generates the [MeshText] of a single character with a custom transformation.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be converted to a mesh.
    /// * `flat`: Set this to `true` for 2D meshes, or to `false` in order
    ///   to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The 4x4 homogenous transformation matrix in column
    ///   major order that will be applied to this text.
    ///
    /// Returns:
    ///
    /// The desired [MeshText] or an [MeshTextError] if anything went wrong in the
    /// process.
    fn generate_glyph(
        &mut self,
        glyph: char,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<MeshText, Box<dyn MeshTextError>> {
        let mut mesh = self.load_from_cache(glyph, flat)?;

        if let Some(value) = transform {
            let transform = Mat4::from_cols_array(value);

            for v in mesh.0.iter_mut() {
                *v = transform.transform_point3a(*v);
            }
            mesh.1.transform(&transform);
        }

        Ok(text_mesh_from_data(mesh))
    }

    /// Generates the two-dimensional [MeshText] of a single character with a custom transformation.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be converted to a mesh.
    /// * `transform`: The 3x3 homogenous transformation matrix in column
    ///   major order that will be applied to this text.
    ///
    /// Returns:
    ///
    /// The desired two-dimensional [MeshText] or an [MeshTextError] if anything went wrong in the
    /// process.
    fn generate_glyph_2d(
        &mut self,
        glyph: char,
        transform: Option<&[f32; 9]>,
    ) -> Result<MeshText, Box<dyn MeshTextError>> {
        let mesh = self.load_from_cache(glyph, true)?;
        let mut mesh = mesh_to_flat_2d(mesh);

        if let Some(value) = transform {
            let transform = Mat3::from_cols_array(value);

            for v in mesh.0.iter_mut() {
                *v = transform.transform_point2(*v);
            }
            mesh.1.transform_2d(&transform);
        }

        Ok(text_mesh_from_data_2d(mesh))
    }

    /// Generates the [IndexedMeshText] of a single character with a custom transformation.
    ///
    /// This function generates a mesh with indices and vertices.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be converted to a mesh.
    /// * `flat`: Set this to `true` for 2D meshes, or to `false` in order
    ///   to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The 4x4 homogenous transformation matrix in column
    ///   major order that will be applied to this text.
    ///
    /// Returns:
    ///
    /// The desired [IndexedMeshText] or an [MeshTextError] if anything went wrong in the
    /// process.
    fn generate_glyph_indexed(
        &mut self,
        glyph: char,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<IndexedMeshText, Box<dyn MeshTextError>> {
        let mut mesh = self.load_from_cache_indexed(glyph, flat)?;

        if let Some(value) = transform {
            let transform = Mat4::from_cols_array(value);

            for v in mesh.1.iter_mut() {
                *v = transform.transform_point3a(*v);
            }
            mesh.2.transform(&transform);
        }

        Ok(text_mesh_from_data_indexed(mesh))
    }

    /// Generates the two-dimensional [IndexedMeshText] of a single character
    /// with a custom transformation.
    ///
    /// This function generates a mesh with indices and vertices.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be converted to a mesh.
    /// * `transform`: The 3x3 homogenous transformation matrix in column
    ///   major order that will be applied to this text.
    ///
    /// Returns:
    ///
    /// The desired [IndexedMeshText] or an [MeshTextError] if anything went wrong in the
    /// process.
    fn generate_glyph_indexed_2d(
        &mut self,
        glyph: char,
        transform: Option<&[f32; 9]>,
    ) -> Result<IndexedMeshText, Box<dyn MeshTextError>> {
        let mesh = self.load_from_cache_indexed(glyph, true)?;
        let mut mesh = mesh_to_indexed_flat_2d(mesh);

        if let Some(value) = transform {
            let transform = Mat3::from_cols_array(value);

            for v in mesh.1.iter_mut() {
                *v = transform.transform_point2(*v);
            }
            mesh.2.transform_2d(&transform);
        }

        Ok(text_mesh_from_data_indexed_2d(mesh))
    }

    /// Generates the [Mesh] of a single character with a custom transformation given
    /// as a [Mat4].
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be converted to a mesh.
    /// * `flat`: Set this to `true` for 2D meshes, or to `false` in order
    ///   to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The 4x4 homogenous transformation matrix.
    ///
    /// Returns:
    ///
    /// The desired [Mesh] or an [MeshTextError] if anything went wrong in the
    /// process.
    pub(crate) fn generate_glyph_with_glam_transform(
        &mut self,
        glyph: char,
        flat: bool,
        transform: &Mat4,
    ) -> Result<Mesh, Box<dyn MeshTextError>> {
        let mut mesh = self.load_from_cache(glyph, flat)?;

        for v in mesh.0.iter_mut() {
            *v = transform.transform_point3a(*v);
        }
        mesh.1.transform(transform);

        Ok(mesh)
    }

    /// Generates the [Mesh2D] of a single character with a custom transformation given
    /// as a [Mat3].
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be converted to a mesh.
    /// * `transform`: The 3x3 homogenous transformation matrix.
    ///
    /// Returns:
    ///
    /// The desired [Mesh2D] or an [MeshTextError] if anything went wrong in the
    /// process.
    pub(crate) fn generate_glyph_with_glam_transform_2d(
        &mut self,
        glyph: char,
        transform: &Mat3,
    ) -> Result<Mesh2D, Box<dyn MeshTextError>> {
        let mesh = self.load_from_cache(glyph, true)?;
        let mut mesh = mesh_to_flat_2d(mesh);

        for v in mesh.0.iter_mut() {
            *v = transform.transform_point2(*v);
        }
        mesh.1.transform_2d(transform);

        Ok(mesh)
    }

    /// Generates the [IndexedMesh] of a single character with a custom transformation given
    /// as a [Mat4].
    ///
    /// This function handles indexed meshes.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be converted to a mesh.
    /// * `flat`: Set this to `true` for 2D meshes, or to `false` in order
    ///   to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The 4x4 homogenous transformation matrix.
    ///
    /// Returns:
    ///
    /// The desired [IndexedMesh] or an [MeshTextError] if anything went wrong in the
    /// process.
    pub(crate) fn generate_glyph_with_glam_transform_indexed(
        &mut self,
        glyph: char,
        flat: bool,
        transform: &Mat4,
    ) -> Result<IndexedMesh, Box<dyn MeshTextError>> {
        let mut mesh = self.load_from_cache_indexed(glyph, flat)?;

        for v in mesh.1.iter_mut() {
            *v = transform.transform_point3a(*v);
        }
        mesh.2.transform(transform);

        Ok(mesh)
    }

    /// Generates the [IndexedMesh2D] of a single character with a custom transformation given
    /// as a [Mat3].
    ///
    /// This function handles indexed meshes.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be converted to a mesh.
    /// * `transform`: The 3x3 homogenous transformation matrix.
    ///
    /// Returns:
    ///
    /// The desired [IndexedMesh2D] or an [MeshTextError] if anything went wrong in the
    /// process.
    pub(crate) fn generate_glyph_with_glam_transform_indexed_2d(
        &mut self,
        glyph: char,
        transform: &Mat3,
    ) -> Result<IndexedMesh2D, Box<dyn MeshTextError>> {
        let mesh = self.load_from_cache_indexed(glyph, true)?;
        let mut mesh = mesh_to_indexed_flat_2d(mesh);

        for v in mesh.1.iter_mut() {
            *v = transform.transform_point2(*v);
        }
        mesh.2.transform_2d(transform);

        Ok(mesh)
    }

    /// Generates the [MeshText] of a given text section.
    ///
    /// Arguments:
    ///
    /// * `text`: The text that should be converted to a mesh.
    /// * `flat`: Set this to `true` for 2D meshes, or to `false` in order
    ///   to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The optional 4x4 homogenous transformation matrix in column
    ///   major order that will be applied to this text.
    ///
    /// Returns:
    ///
    /// The desired [MeshText] or an [MeshTextError] if anything went wrong in the
    /// process.
    fn generate_text_section(
        &mut self,
        text: &str,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<MeshText, Box<dyn MeshTextError>> {
        let base_transform = match transform {
            Some(value) => Mat4::from_cols_array(value),
            None => Mat4::IDENTITY,
        };

        let mut mesh = (Vec::new(), BoundingBox::empty());
        let mut overall_advance = 0f32;

        let mut chars_iter = text.chars();

        // The first char will be handled differently if present.
        if let Some(first_glyph) = chars_iter.next() {
            let x_advance = self
                .font
                .glyph_hor_advance(self.glyph_id_of_char(first_glyph))
                .unwrap_or(0) as f32
                / self.font.height() as f32;

            let transform =
                base_transform * Mat4::from_translation(Vec3::new(overall_advance, 0f32, 0f32));
            let mut glyph_mesh =
                self.generate_glyph_with_glam_transform(first_glyph, flat, &transform)?;

            // Add vertices and replace bbox.
            mesh.0.append(&mut glyph_mesh.0);
            mesh = (mesh.0, glyph_mesh.1);

            overall_advance += x_advance;
        }

        for glyph in chars_iter {
            let x_advance = self
                .font
                .glyph_hor_advance(self.glyph_id_of_char(glyph))
                .unwrap_or(0) as f32
                / self.font.height() as f32;

            let transform =
                base_transform * Mat4::from_translation(Vec3::new(overall_advance, 0f32, 0f32));
            let mut glyph_mesh =
                self.generate_glyph_with_glam_transform(glyph, flat, &transform)?;

            // Add vertices and adjust bbox.
            mesh.0.append(&mut glyph_mesh.0);
            mesh = (mesh.0, mesh.1.combine(&glyph_mesh.1));

            overall_advance += x_advance;
        }

        Ok(text_mesh_from_data(mesh))
    }

    /// Generates two-dimensional [MeshText] for a given text section.
    ///
    /// Arguments:
    ///
    /// * `text`: The text that should be converted to a mesh.
    /// * `transform`: The optional 3x3 homogenous transformation matrix in column
    ///   major order that will be applied to this text.
    ///
    /// Returns:
    ///
    /// The desired [MeshText] or an [MeshTextError] if anything went wrong in the
    /// process.
    fn generate_text_section_2d(
        &mut self,
        text: &str,
        transform: Option<&[f32; 9]>,
    ) -> Result<MeshText, Box<dyn MeshTextError>> {
        let base_transform = match transform {
            Some(value) => Mat3::from_cols_array(value),
            None => Mat3::IDENTITY,
        };

        let mut mesh = (Vec::new(), BoundingBox::empty());
        let mut overall_advance = 0f32;

        let mut chars_iter = text.chars();

        // The first char will be handled differently if present.
        if let Some(first_glyph) = chars_iter.next() {
            let x_advance = self
                .font
                .glyph_hor_advance(self.glyph_id_of_char(first_glyph))
                .unwrap_or(0) as f32
                / self.font.height() as f32;

            let transform =
                base_transform * Mat3::from_translation(Vec2::new(overall_advance, 0f32));
            let mut glyph_mesh =
                self.generate_glyph_with_glam_transform_2d(first_glyph, &transform)?;

            // Add vertices and replace bbox.
            mesh.0.append(&mut glyph_mesh.0);
            mesh = (mesh.0, glyph_mesh.1);

            overall_advance += x_advance;
        }

        for glyph in chars_iter {
            let x_advance = self
                .font
                .glyph_hor_advance(self.glyph_id_of_char(glyph))
                .unwrap_or(0) as f32
                / self.font.height() as f32;

            let transform =
                base_transform * Mat3::from_translation(Vec2::new(overall_advance, 0f32));
            let mut glyph_mesh = self.generate_glyph_with_glam_transform_2d(glyph, &transform)?;

            // Add vertices and adjust bbox.
            mesh.0.append(&mut glyph_mesh.0);
            mesh = (mesh.0, mesh.1.combine(&glyph_mesh.1));

            overall_advance += x_advance;
        }

        Ok(text_mesh_from_data_2d(mesh))
    }

    /// Generates the [MeshText] of a given text section.
    ///
    /// This function handles indexed meshes.
    ///
    /// Arguments:
    ///
    /// * `text`: The text that should be converted to a mesh.
    /// * `flat`: Set this to `true` for 2D meshes, or to `false` in order
    ///   to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The optional 4x4 homogenous transformation matrix in column
    ///   major order that will be applied to this text.
    ///
    /// Returns:
    ///
    /// The desired [MeshText] or an [MeshTextError] if anything went wrong in the
    /// process.
    fn generate_text_section_indexed(
        &mut self,
        text: &str,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<IndexedMeshText, Box<dyn MeshTextError>> {
        let base_transform = match transform {
            Some(value) => Mat4::from_cols_array(value),
            None => Mat4::IDENTITY,
        };

        let mut mesh = (Vec::new(), Vec::new(), BoundingBox::empty());
        let mut overall_advance = 0f32;
        let mut index_offset = 0;

        let mut chars_iter = text.chars();

        // The first char will be handled differently if present.
        if let Some(first_glyph) = chars_iter.next() {
            let x_advance = self
                .font
                .glyph_hor_advance(self.glyph_id_of_char(first_glyph))
                .unwrap_or(0) as f32
                / self.font.height() as f32;

            let transform =
                base_transform * Mat4::from_translation(Vec3::new(overall_advance, 0f32, 0f32));
            let mut glyph_mesh =
                self.generate_glyph_with_glam_transform_indexed(first_glyph, flat, &transform)?;

            // Update index offset (note that glyph meshes can be empty).
            if let Some(max) = glyph_mesh.0.iter().max() {
                index_offset = *max + 1;
            }

            // Add vertices and replace bbox.
            mesh.0.append(&mut glyph_mesh.0);
            mesh.1.append(&mut glyph_mesh.1);
            mesh = (mesh.0, mesh.1, glyph_mesh.2);

            overall_advance += x_advance;
        }

        for glyph in chars_iter {
            let x_advance = self
                .font
                .glyph_hor_advance(self.glyph_id_of_char(glyph))
                .unwrap_or(0) as f32
                / self.font.height() as f32;

            let transform =
                base_transform * Mat4::from_translation(Vec3::new(overall_advance, 0f32, 0f32));
            let mut glyph_mesh =
                self.generate_glyph_with_glam_transform_indexed(glyph, flat, &transform)?;

            // Offset indices.
            for i in glyph_mesh.0.iter_mut() {
                *i += index_offset;
            }

            // Update index offset (note that glyph meshes can be empty).
            if let Some(max) = glyph_mesh.0.iter().max() {
                index_offset = *max + 1;
            }

            // Add vertices and indices and adjust bbox.
            mesh.0.append(&mut glyph_mesh.0);
            mesh.1.append(&mut glyph_mesh.1);
            mesh = (mesh.0, mesh.1, mesh.2.combine(&glyph_mesh.2));

            overall_advance += x_advance;
        }

        Ok(text_mesh_from_data_indexed(mesh))
    }

    /// Generates two-dimensional [MeshText] for a given text section.
    ///
    /// This function handles indexed meshes.
    ///
    /// Arguments:
    ///
    /// * `text`: The text that should be converted to a mesh.
    /// * `transform`: The optional 3x3 homogenous transformation matrix in column
    ///   major order that will be applied to this text.
    ///
    /// Returns:
    ///
    /// The desired [MeshText] or an [MeshTextError] if anything went wrong in the
    /// process.
    fn generate_text_section_indexed_2d(
        &mut self,
        text: &str,
        transform: Option<&[f32; 9]>,
    ) -> Result<IndexedMeshText, Box<dyn MeshTextError>> {
        let base_transform = match transform {
            Some(value) => Mat3::from_cols_array(value),
            None => Mat3::IDENTITY,
        };

        let mut mesh = (Vec::new(), Vec::new(), BoundingBox::empty());
        let mut overall_advance = 0f32;
        let mut index_offset = 0;

        let mut chars_iter = text.chars();

        // The first char will be handled differently if present.
        if let Some(first_glyph) = chars_iter.next() {
            let x_advance = self
                .font
                .glyph_hor_advance(self.glyph_id_of_char(first_glyph))
                .unwrap_or(0) as f32
                / self.font.height() as f32;

            let transform =
                base_transform * Mat3::from_translation(Vec2::new(overall_advance, 0f32));
            let mut glyph_mesh =
                self.generate_glyph_with_glam_transform_indexed_2d(first_glyph, &transform)?;

            // Update index offset (note that glyph meshes can be empty).
            if let Some(max) = glyph_mesh.0.iter().max() {
                index_offset = *max + 1;
            }

            // Add vertices and replace bbox.
            mesh.0.append(&mut glyph_mesh.0);
            mesh.1.append(&mut glyph_mesh.1);
            mesh = (mesh.0, mesh.1, glyph_mesh.2);

            overall_advance += x_advance;
        }

        for glyph in chars_iter {
            let x_advance = self
                .font
                .glyph_hor_advance(self.glyph_id_of_char(glyph))
                .unwrap_or(0) as f32
                / self.font.height() as f32;

            let transform =
                base_transform * Mat3::from_translation(Vec2::new(overall_advance, 0f32));
            let mut glyph_mesh =
                self.generate_glyph_with_glam_transform_indexed_2d(glyph, &transform)?;

            // Offset indices.
            for i in glyph_mesh.0.iter_mut() {
                *i += index_offset;
            }

            // Update index offset (note that glyph meshes can be empty).
            if let Some(max) = glyph_mesh.0.iter().max() {
                index_offset = *max + 1;
            }

            // Add vertices and indices and adjust bbox.
            mesh.0.append(&mut glyph_mesh.0);
            mesh.1.append(&mut glyph_mesh.1);
            mesh = (mesh.0, mesh.1, mesh.2.combine(&glyph_mesh.2));

            overall_advance += x_advance;
        }

        Ok(text_mesh_from_data_indexed_2d(mesh))
    }

    /// Loads the given glyph from the cache or adds it.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be loaded.
    /// * `flat`: Wether the character should be laid out in a 2D mesh.
    ///
    /// Returns:
    ///
    /// A [Result] containing the [Mesh] if successful, otherwise an [MeshTextError].
    fn load_from_cache(&mut self, glyph: char, flat: bool) -> Result<Mesh, Box<dyn MeshTextError>> {
        if flat {
            match self.cache.get(&glyph.to_string()) {
                Some(glyph_mesh) => Ok(glyph_mesh.to_owned()),
                None => self.insert_into_cache(glyph, flat),
            }
        } else {
            match self.cache.get(&format!("_{}", glyph)) {
                Some(glyph_mesh) => Ok(glyph_mesh.to_owned()),
                None => self.insert_into_cache(glyph, flat),
            }
        }
    }

    /// Loads the given glyph from the cache or adds it.
    ///
    /// This function deals with indexed meshes.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be loaded.
    /// * `flat`: Wether the character should be laid out in a 2D mesh.
    ///
    /// Returns:
    ///
    /// A [Result] containing the [IndexedMesh] if successful, otherwise an [MeshTextError].
    fn load_from_cache_indexed(
        &mut self,
        glyph: char,
        flat: bool,
    ) -> Result<IndexedMesh, Box<dyn MeshTextError>> {
        if flat {
            match self.indexed_cache.get(&glyph.to_string()) {
                Some(glyph_mesh) => Ok(glyph_mesh.to_owned()),
                None => self.insert_into_cache_indexed(glyph, flat),
            }
        } else {
            match self.indexed_cache.get(&format!("_{}", glyph)) {
                Some(glyph_mesh) => Ok(glyph_mesh.to_owned()),
                None => self.insert_into_cache_indexed(glyph, flat),
            }
        }
    }

    /// Generates a new [Mesh] from the loaded font and the given `glyph`
    /// and inserts it into the internal `cache`.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be loaded.
    /// * `flat`: Wether the character should be laid out in a 2D mesh.
    ///
    /// Returns:
    ///
    /// A [Result] containing the [Mesh] if successful, otherwise an [MeshTextError].
    fn insert_into_cache(
        &mut self,
        glyph: char,
        flat: bool,
    ) -> Result<Mesh, Box<dyn MeshTextError>> {
        let font_height = self.font.height() as f32;
        let mut builder = GlyphOutlineBuilder::new(font_height, self.quality);

        let glyph_index = self.glyph_id_of_char(glyph);

        let mut depth = (0.5f32, -0.5f32);
        let (rect, mesh) = match self.font.outline_glyph(glyph_index, &mut builder) {
            Some(bbox) => {
                let mesh = raster_to_mesh(&builder.get_glyph_outline(), flat)?;
                (bbox, mesh)
            }
            None => {
                // The glyph has no outline so it is most likely a space or any other
                // character that can not be displayed.
                // An empty mesh is cached for simplicity nevertheless.
                depth = (0f32, 0f32);
                (
                    ttf_parser::Rect {
                        x_min: 0,
                        y_min: 0,
                        x_max: 0,
                        y_max: 0,
                    },
                    Vec::new(),
                )
            }
        };

        // Add mesh to cache.
        let bbox;
        if flat {
            bbox = BoundingBox {
                max: Vec3A::new(
                    rect.x_max as f32 / font_height,
                    rect.y_max as f32 / font_height,
                    0f32,
                ),
                min: Vec3A::new(
                    rect.x_min as f32 / font_height,
                    rect.y_min as f32 / font_height,
                    0f32,
                ),
            };
            self.cache.insert(glyph.to_string(), (mesh.clone(), bbox));
        } else {
            bbox = BoundingBox {
                max: Vec3A::new(
                    rect.x_max as f32 / font_height,
                    rect.y_max as f32 / font_height,
                    depth.0,
                ),
                min: Vec3A::new(
                    rect.x_min as f32 / font_height,
                    rect.y_min as f32 / font_height,
                    depth.1,
                ),
            };
            self.cache
                .insert(format!("_{}", glyph), (mesh.clone(), bbox));
        }

        Ok((mesh, bbox))
    }

    /// Generates a new [IndexedMesh] from the loaded font and the given `glyph`
    /// and inserts it into the internal `cache`.
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character that should be loaded.
    /// * `flat`: Wether the character should be laid out in a 2D mesh.
    ///
    /// Returns:
    ///
    /// A [Result] containing the [IndexedMesh] if successful, otherwise an [MeshTextError].
    fn insert_into_cache_indexed(
        &mut self,
        glyph: char,
        flat: bool,
    ) -> Result<IndexedMesh, Box<dyn MeshTextError>> {
        let font_height = self.font.height() as f32;
        let mut builder = GlyphOutlineBuilder::new(font_height, self.quality);

        let glyph_index = self.glyph_id_of_char(glyph);

        let mut depth = (0.5f32, -0.5f32);
        let (rect, vertices, indices) = match self.font.outline_glyph(glyph_index, &mut builder) {
            Some(bbox) => {
                let mesh = raster_to_mesh_indexed(&builder.get_glyph_outline(), flat)?;
                (bbox, mesh.0, mesh.1)
            }
            None => {
                // The glyph has no outline so it is most likely a space or any other
                // character that can not be displayed.
                // An empty mesh is cached for simplicity nevertheless.
                depth = (0f32, 0f32);
                (
                    ttf_parser::Rect {
                        x_min: 0,
                        y_min: 0,
                        x_max: 0,
                        y_max: 0,
                    },
                    Vec::new(),
                    Vec::new(),
                )
            }
        };

        // Add mesh to cache.
        let bbox;
        if flat {
            bbox = BoundingBox {
                max: Vec3A::new(
                    rect.x_max as f32 / font_height,
                    rect.y_max as f32 / font_height,
                    0f32,
                ),
                min: Vec3A::new(
                    rect.x_min as f32 / font_height,
                    rect.y_min as f32 / font_height,
                    0f32,
                ),
            };
            self.indexed_cache
                .insert(glyph.to_string(), (indices.clone(), vertices.clone(), bbox));
        } else {
            bbox = BoundingBox {
                max: Vec3A::new(
                    rect.x_max as f32 / font_height,
                    rect.y_max as f32 / font_height,
                    depth.0,
                ),
                min: Vec3A::new(
                    rect.x_min as f32 / font_height,
                    rect.y_min as f32 / font_height,
                    depth.1,
                ),
            };
            self.indexed_cache.insert(
                format!("_{}", glyph),
                (indices.clone(), vertices.clone(), bbox),
            );
        }

        Ok((indices, vertices, bbox))
    }

    /// Finds the [GlyphId] of a certain [char].
    ///
    /// Arguments:
    ///
    /// * `glyph`: The character of which the id is determined.
    ///
    /// Returns:
    ///
    /// The corresponding [GlyphId].
    fn glyph_id_of_char(&self, glyph: char) -> GlyphId {
        self.font
            .glyph_index(glyph)
            .unwrap_or(ttf_parser::GlyphId(0))
    }
}

impl<T> TextSection<MeshText> for MeshGenerator<T>
where
    T: FontFace,
{
    fn generate_section(
        &mut self,
        text: &str,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<MeshText, Box<dyn MeshTextError>> {
        self.generate_text_section(text, flat, transform)
    }

    fn generate_section_2d(
        &mut self,
        text: &str,
        transform: Option<&[f32; 9]>,
    ) -> Result<MeshText, Box<dyn MeshTextError>> {
        self.generate_text_section_2d(text, transform)
    }
}

impl<T> TextSection<IndexedMeshText> for MeshGenerator<T>
where
    T: FontFace,
{
    fn generate_section(
        &mut self,
        text: &str,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<IndexedMeshText, Box<dyn MeshTextError>> {
        self.generate_text_section_indexed(text, flat, transform)
    }

    fn generate_section_2d(
        &mut self,
        text: &str,
        transform: Option<&[f32; 9]>,
    ) -> Result<IndexedMeshText, Box<dyn MeshTextError>> {
        self.generate_text_section_indexed_2d(text, transform)
    }
}

impl<T> Glyph<MeshText> for MeshGenerator<T>
where
    T: FontFace,
{
    fn generate_glyph(
        &mut self,
        glyph: char,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<MeshText, Box<dyn MeshTextError>> {
        self.generate_glyph(glyph, flat, transform)
    }

    fn generate_glyph_2d(
        &mut self,
        glyph: char,
        transform: Option<&[f32; 9]>,
    ) -> Result<MeshText, Box<dyn MeshTextError>> {
        self.generate_glyph_2d(glyph, transform)
    }
}

impl<T> Glyph<IndexedMeshText> for MeshGenerator<T>
where
    T: FontFace,
{
    fn generate_glyph(
        &mut self,
        glyph: char,
        flat: bool,
        transform: Option<&[f32; 16]>,
    ) -> Result<IndexedMeshText, Box<dyn MeshTextError>> {
        self.generate_glyph_indexed(glyph, flat, transform)
    }

    fn generate_glyph_2d(
        &mut self,
        glyph: char,
        transform: Option<&[f32; 9]>,
    ) -> Result<IndexedMeshText, Box<dyn MeshTextError>> {
        self.generate_glyph_indexed_2d(glyph, transform)
    }
}

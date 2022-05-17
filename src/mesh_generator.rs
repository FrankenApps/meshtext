use std::collections::HashMap;

use glam::{Mat3, Mat4, Vec2, Vec3, Vec3A};
use ttf_parser::{Face, GlyphId};

use crate::{
    error::MeshTextError,
    util::{
        mesh_to_flat_2d, mesh_to_indexed_flat_2d, raster_to_mesh, raster_to_mesh_indexed,
        text_mesh_from_data, text_mesh_from_data_2d, text_mesh_from_data_indexed,
        text_mesh_from_data_indexed_2d, GlyphOutlineBuilder,
    },
    BoundingBox, CacheType, Glyph, IndexedMeshText, MeshText, QualitySettings, TextSection,
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
pub struct MeshGenerator {
    /// Cached non-indexed glyphs are stored in this [HashMap].
    ///
    /// The key is the character itself, however because each
    /// character can have a 2D and a 3D variant, in the 3D
    /// variant each character is prefixed with an `_`.
    #[allow(unused)]
    cache: HashMap<String, Mesh>,

    /// The current [Face].
    font: Face<'static>,

    /// Cached indexed glyphs are stored in this [HashMap].
    ///
    /// The key is the character itself, however because each
    /// character can have a 2D and a 3D variant, in the 3D
    /// variant each character is prefixed with an `_`.
    #[allow(unused)]
    indexed_cache: HashMap<String, IndexedMesh>,

    /// Quality settings for generating the text meshes.
    quality: QualitySettings,

    /// Controls wether the generator will automatically
    /// cache glyphs.
    #[allow(unused)]
    use_cache: bool,
}

impl MeshGenerator {
    /// Creates a new [MeshGenerator].
    ///
    /// Arguments:
    ///
    /// * `font`: The font that will be used for rasterizing.
    pub fn new(font: &'static [u8]) -> Self {
        let face = Face::from_slice(font, 0).expect("Failed to generate font from data.");

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
        let face = Face::from_slice(font, 0).expect("Failed to generate font from data.");

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
        let face = Face::from_slice(font, 0).expect("Failed to generate font from data.");

        Self {
            cache: HashMap::new(),
            font: face,
            indexed_cache: HashMap::new(),
            quality,
            use_cache: false,
        }
    }

    /// Fills the internal cache of a [MeshGenerator] with the given characters.
    ///
    /// Arguments:
    ///
    /// * `glyphs`: The glyphs that will be precached. Each character should appear exactly once.
    /// * `flat`: Wether the flat or three-dimensional variant of the characters should be preloaded.
    /// If both variants should be precached this function must be called twice with this parameter set
    /// to `true` and `false`.
    /// * `cache`: An optional value that controls which cache will be filled. `None` means both caches will be filled.
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
    /// // Precache both flat and three-dimensional glyphs both for indexed and non-indexed meshes.
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
            // If no type is set explicitely, both variants will be precached.
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
    /// to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The 4x4 homogenous transformation matrix in column
    /// major order that will be applied to this text.
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
    /// major order that will be applied to this text.
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
    /// to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The 4x4 homogenous transformation matrix in column
    /// major order that will be applied to this text.
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
    /// major order that will be applied to this text.
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
    /// to generate a mesh with a depth of `1.0` units.
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
    /// to generate a mesh with a depth of `1.0` units.
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
    /// to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The optional 4x4 homogenous transformation matrix in column
    /// major order that will be applied to this text.
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
    /// major order that will be applied to this text.
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
    /// to generate a mesh with a depth of `1.0` units.
    /// * `transform`: The optional 4x4 homogenous transformation matrix in column
    /// major order that will be applied to this text.
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
    /// major order that will be applied to this text.
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
                // charcter that can not be displayed.
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
                // charcter that can not be displayed.
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

impl TextSection<MeshText> for MeshGenerator {
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

impl TextSection<IndexedMeshText> for MeshGenerator {
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

impl Glyph<MeshText> for MeshGenerator {
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

impl Glyph<IndexedMeshText> for MeshGenerator {
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

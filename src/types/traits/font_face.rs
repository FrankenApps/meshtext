use skrifa::{
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
    FontRef, GlyphId, MetadataProvider,
};

use crate::error::{GlyphOutlineError, MeshTextError};

/// Shared access to the borrowed and owned font backends.
pub trait FontFace {
    fn as_font_ref(&self) -> &FontRef<'_>;

    fn glyph_hor_advance(&self, glyph_id: GlyphId) -> Option<f32> {
        self.as_font_ref()
            .glyph_metrics(Size::unscaled(), LocationRef::default())
            .advance_width(glyph_id)
    }

    fn glyph_index(&self, code_point: char) -> Option<GlyphId> {
        self.as_font_ref().charmap().map(code_point)
    }

    fn height(&self) -> f32 {
        let metrics = self
            .as_font_ref()
            .metrics(Size::unscaled(), LocationRef::default());
        // Preserve normalization by ascent minus descent, excluding the line gap.
        metrics.ascent - metrics.descent
    }

    fn outline_glyph(
        &self,
        glyph_id: GlyphId,
        builder: &mut impl OutlinePen,
    ) -> Result<(), Box<dyn MeshTextError>> {
        let outlines = self.as_font_ref().outline_glyphs();
        let Some(glyph) = outlines.get(glyph_id) else {
            return Ok(());
        };
        glyph
            .draw(
                DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                builder,
            )
            .map_err(|_| Box::new(GlyphOutlineError) as Box<dyn MeshTextError>)?;
        Ok(())
    }
}

impl FontFace for FontRef<'_> {
    fn as_font_ref(&self) -> &FontRef<'_> {
        self
    }
}

#[cfg(feature = "owned")]
impl FontFace for crate::OwnedFace {
    fn as_font_ref(&self) -> &FontRef<'_> {
        self.as_face_ref()
    }
}

use skrifa::{raw::ReadError, FontRef};

self_cell::self_cell! {
    struct FontCell {
        owner: Vec<u8>,

        #[covariant]
        dependent: FontRef,
    }
}

/// A parsed font that owns its backing bytes.
///
/// The font is parsed once and can safely move with its data. References returned
/// by [Self::as_face_ref] are tied to this owner's lifetime.
pub struct OwnedFace {
    inner: FontCell,
}

impl OwnedFace {
    /// Parses a font, or a face at `index` in a font collection, taking ownership
    /// of `data`. A standalone font requires index zero.
    pub fn from_vec(data: Vec<u8>, index: u32) -> Result<Self, ReadError> {
        Ok(Self {
            inner: FontCell::try_new(data, |data| FontRef::from_index(data, index))?,
        })
    }

    /// Borrows the parsed font for use with [skrifa::MetadataProvider].
    pub fn as_face_ref(&self) -> &FontRef<'_> {
        self.inner.borrow_dependent()
    }

    /// Returns the original font bytes.
    pub fn as_slice(&self) -> &[u8] {
        self.inner.borrow_owner()
    }

    /// Consumes the font and returns its backing buffer.
    pub fn into_vec(self) -> Vec<u8> {
        self.inner.into_owner()
    }
}

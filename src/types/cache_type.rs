/// Allows referencing one of the internal caches.
#[derive(Debug, Clone, Copy)]
pub enum CacheType {
    /// The caache that handles non-indexed meshes.
    Normal,

    /// The cache that handles indexed meshes.
    Indexed,
}

impl Default for CacheType {
    fn default() -> Self {
        CacheType::Normal
    }
}

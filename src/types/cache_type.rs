/// Allows referencing one of the internal caches.
#[derive(Debug, Clone, Copy, Default)]
pub enum CacheType {
    /// The cache that handles non-indexed meshes.
    #[default]
    Normal,

    /// The cache that handles indexed meshes.
    Indexed,
}

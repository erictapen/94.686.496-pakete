/// Dumb name for a trait, that is implemented by any piece of image data, that was generated from
/// the raw_data.
use std::path::PathBuf;

pub trait CachableData {
    /// Return the data, but first if is already in the cache.
    fn data_cached(&self) -> Vec<u64>;
    /// Return the data, but force recomputation.
    fn data_uncached(&self) -> Vec<u64>;
}

pub trait CachablePNG {
    /// Return the image, but first if is already in the cache.
    fn png_cached(&self) -> Result<PathBuf, &str>;
    /// Return the image, but force recomputation.
    fn png_uncached(&self, path: PathBuf) -> Result<PathBuf, &str>;
}

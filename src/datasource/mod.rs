pub mod blob_fetcher;

#[derive(Debug, Clone)]
pub enum CompressionType {
    None,
    Zlib,
}

use alloy::transports::TransportError;
use thiserror::Error;

/// Configuration for the live event indexer.
#[derive(Clone, Debug)]
pub struct EventIndexerConfig {
    pub batch_size: u64,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub max_block_range: u64,
}

/// Default configuration values for the live event indexer.
impl Default for EventIndexerConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            max_retries: 3,
            retry_delay_ms: 1000,
            max_block_range: 10000,
        }
    }
}

#[derive(Debug, Error)]
pub enum EventIndexerError {
    #[error("Provider error: {0}")]
    ProviderError(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl From<TransportError> for EventIndexerError {
    fn from(err: TransportError) -> Self {
        EventIndexerError::ProviderError(err.to_string())
    }
}

use crate::traits::{DataAvailabilityWatcher, DataSourceFetcher};
use async_trait::async_trait;
use std::fmt::{Display, Formatter};
use tokio::sync::mpsc::{self, Receiver};
use tracing::{debug, info};

pub struct DAWatcher {
    poll_interval: std::time::Duration,
    buffer_size: usize,
}

impl DAWatcher {
    pub fn new(poll_interval: std::time::Duration, buffer_size: usize) -> Self {
        Self {
            poll_interval,
            buffer_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProposalManifest {
    pub block_number: u64,
    pub timestamp: u64,
    pub data_hash: Vec<u8>,
}

#[derive(Debug)]
pub enum WatcherError {
    ChannelError(String),
    FetchError(String),
}

impl Display for WatcherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WatcherError::ChannelError(msg) => write!(f, "Channel error: {}", msg),
            WatcherError::FetchError(msg) => write!(f, "Fetch error: {}", msg),
        }
    }
}

pub struct DefaultDataSourceFetcher;

#[derive(Debug)]
pub enum FetcherError {
    NetworkError(String),
    DecodeError(String),
    DecompressionError(String),
}

impl Display for FetcherError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FetcherError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            FetcherError::DecodeError(msg) => write!(f, "Decode error: {}", msg),
            FetcherError::DecompressionError(msg) => write!(f, "Decompression error: {}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataQuery {
    pub block_range: (u64, u64),
}

#[derive(Debug, Clone)]
pub enum CompressionType {
    None,
    Gzip,
    Zstd,
}

#[async_trait]
impl DataSourceFetcher for DefaultDataSourceFetcher {
    type Query = DataQuery;
    type Compression = CompressionType;
    type RawDataType = Vec<u8>;
    type DecodedType = Vec<u8>;
    type DecompressedType = Vec<u8>;
    type Error = FetcherError;

    async fn fetch(&self, query: &Self::Query) -> Result<Self::RawDataType, Self::Error> {
        debug!("Fetching data for block range: {:?}", query.block_range);
        Ok(vec![0u8; 32])
    }

    async fn decode(&self, raw: Self::RawDataType) -> Result<Self::DecodedType, Self::Error> {
        Ok(raw)
    }

    async fn decompress(
        &self,
        data: Self::DecodedType,
    ) -> Result<Self::DecompressedType, Self::Error> {
        Ok(data)
    }

    fn compression_type(&self) -> Self::Compression {
        CompressionType::None
    }
}

#[async_trait]
impl DataAvailabilityWatcher for DAWatcher {
    type ProposalManifest = ProposalManifest;
    type DataSourceFetcher = DefaultDataSourceFetcher;
    type Error = WatcherError;

    async fn watch(&self) -> Result<Receiver<Self::ProposalManifest>, Self::Error> {
        let (tx, rx) = mpsc::channel(self.buffer_size);
        let poll_interval = self.poll_interval;

        tokio::spawn(async move {
            let mut block_number = 0u64;
            loop {
                info!("Watching for new proposals at block {}", block_number);

                let proposal = ProposalManifest {
                    block_number,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    data_hash: vec![0u8; 32],
                };

                if tx.send(proposal).await.is_err() {
                    debug!("Receiver dropped, stopping watcher");
                    break;
                }

                block_number += 1;
                tokio::time::sleep(poll_interval).await;
            }
        });

        Ok(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_watcher_creation() {
        let watcher = DAWatcher::new(std::time::Duration::from_secs(1), 100);

        let mut receiver = watcher.watch().await.unwrap();
        let proposal = receiver.recv().await.unwrap();

        assert_eq!(proposal.block_number, 0);
        assert_eq!(proposal.data_hash.len(), 32);
    }
}

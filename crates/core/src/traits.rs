use std::fmt::Display;

use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

#[async_trait]
pub trait Driver {
    /// The Data Availability Watcher type for subscribing to on-chain events.
    type DataAvailabilityWatcher: DataAvailabilityWatcher;
    type DerivationPipeline: DerivationPipeline;
    type EngineExecutor: EngineExecutor;
    type Error: Display;

    async fn run(&self) -> Result<(), Self::Error>;
}

#[async_trait]
pub trait DataAvailabilityWatcher {
    type ProposalManifest;
    type DataSourceFetcher: DataSourceFetcher;
    type Error: Display;

    async fn watch(&self) -> Result<Receiver<Self::ProposalManifest>, Self::Error>;
}

#[async_trait]
pub trait DerivationPipeline {
    type ProposalManifest;
    type BlockPayloadAttributes;
    type Error: Display;

    async fn derive(
        &self,
        proposal: Self::ProposalManifest,
    ) -> Result<Self::BlockPayloadAttributes, Self::Error>;
}

#[async_trait]
pub trait EngineExecutor {
    type BlockPayloadAttributes;
    type ExecutionResult;
    type Error: Display;

    async fn execute(
        &self,
        payload: Self::BlockPayloadAttributes,
    ) -> Result<Self::ExecutionResult, Self::Error>;
}

#[async_trait]
pub trait DataSourceFetcher {
    type Query;
    type Compression;
    type RawDataType;
    type DecodedType;
    type DecompressedType;
    type Error: Display;

    async fn fetch(&self, query: &Self::Query) -> Result<Self::RawDataType, Self::Error>;

    async fn decode(&self, raw: Self::RawDataType) -> Result<Self::DecodedType, Self::Error>;

    async fn decompress(
        &self,
        data: Self::DecodedType,
    ) -> Result<Self::DecompressedType, Self::Error>;

    fn compression_type(&self) -> Self::Compression;
}

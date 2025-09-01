use std::time::Duration;

use alloy::{
    eips::BlockNumberOrTag,
    primitives::{Address, B256},
    providers::Provider,
    rpc::types::{Filter, Log},
};
use futures::StreamExt;
use tokio::time::sleep;
use tracing::info;

use crate::event_indexer::common::{EventIndexerConfig, EventIndexerError};

#[derive(Clone, Debug)]
pub struct EventIndexer<P> {
    provider: P,
    config: EventIndexerConfig,
    contract_address: Address,
    topic: B256,
    last_indexed_block: u64,
    is_indexing: bool,
}

impl<P: Provider + Clone + Send + Sync + 'static> EventIndexer<P> {
    pub fn new(provider: P, config: EventIndexerConfig) -> Self {
        Self {
            provider,
            config,
            contract_address: Address::ZERO,
            topic: B256::ZERO,
            last_indexed_block: 0,
            is_indexing: false,
        }
    }

    pub async fn run(&mut self, start_block: Option<u64>) -> Result<(), EventIndexerError> {
        // 1. Fetch the latest block number from the provider.
        let latest_block = self.provider.get_block_number().await?;
        info!("Latest block number: {}", latest_block);

        let start_block = start_block.unwrap_or(self.last_indexed_block);
        self.last_indexed_block = start_block;

        // 2. Index historical events from start_block to latest_block.
        if start_block < latest_block {
            info!(
                "Starting indexing historical blocks: {} to {}",
                start_block, latest_block
            );
            self.index_events(start_block, latest_block).await?;
        } else {
            info!(
                "No historical blocks to index. Current block: {}",
                start_block
            );
        }

        // 3. Subscribe to new blocks and index events in real-time.
        self.subscribe_and_index().await?;

        Ok(())
    }

    pub async fn index_events(
        &mut self,
        from_block: u64,
        to_block: u64,
    ) -> Result<(), EventIndexerError> {
        self.is_indexing = true;
        let _total_blocks = to_block - from_block + 1;

        let mut current = from_block;
        let mut all_logs = Vec::new();

        while current <= to_block {
            let end = (current + self.config.batch_size - 1).min(to_block);

            let logs = self.fetch_logs_range(current, end).await?;

            info!("Fetched {} logs for blocks {}-{}", logs.len(), current, end);
            all_logs.extend(logs);

            self.last_indexed_block = end;
            current = end + 1;
        }

        info!("Indexing complete: {} total logs", all_logs.len());

        for log in all_logs {
            self.process_log(&log).await?;
        }

        self.is_indexing = false;
        Ok(())
    }

    async fn subscribe_and_index(&mut self) -> Result<(), EventIndexerError> {
        let subscription = self.provider.subscribe_blocks().await?;
        let mut block_stream = subscription.into_stream();

        info!("Subscribed to new blocks via WebSocket/IPC");

        while let Some(block) = block_stream.next().await {
            let block_number = block.number;

            let from_block = self.last_indexed_block + 1;
            let logs = self.fetch_logs_range(from_block, block_number).await?;

            if !logs.is_empty() {
                info!(
                    "Blocks {}-{}: processing {} events",
                    from_block,
                    block_number,
                    logs.len()
                );
                for log in &logs {
                    self.process_log(log).await?;
                }
            }

            self.last_indexed_block = block_number;
        }

        info!("Block subscription ended");
        Ok(())
    }

    async fn fetch_logs_range(&self, from: u64, to: u64) -> Result<Vec<Log>, EventIndexerError> {
        let filter = Filter::new()
            .from_block(BlockNumberOrTag::Number(from))
            .to_block(BlockNumberOrTag::Number(to))
            .address(self.contract_address)
            .event_signature(self.topic);

        let mut retries = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            match self.provider.get_logs(&filter).await {
                Ok(logs) => return Ok(logs),
                Err(e) if retries < MAX_RETRIES => {
                    retries += 1;
                    info!("Retry {}/{}: {}", retries, MAX_RETRIES, e);
                    sleep(Duration::from_millis(1000 * retries as u64)).await;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    async fn process_log(&self, log: &Log) -> Result<(), EventIndexerError> {
        info!(
            "Event: block={}, tx={:?}",
            log.block_number.unwrap_or_default(),
            log.transaction_hash,
        );
        Ok(())
    }
}

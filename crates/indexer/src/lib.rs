use std::{cmp::min, time::Duration};

use alloy::{
    primitives::Address,
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{Filter, Log},
    transports::http::{Client, Http},
};
use anyhow::Result;
use keystore_bindings::KeyStore::{BatchProved, ForcedTxRegistered, VkRegistered};
use tokio::{sync::mpsc::Sender, time::sleep};

mod keystore_bindings;

/// State updates identified by monitoring the L1 KeyStore contract events.
#[derive(Debug)]
pub enum StateUpdate {
    /// A new verifying key has been registered.
    VerifyingKey(VkRegistered),
    /// A batch of record updates has been proved.
    Records(BatchProved),
}

/// A forced transaction that has been submitted directly to the L1 KeyStore contract.
#[derive(Debug, Default)]
pub struct ForcedTx(pub ForcedTxRegistered);

/// The indexer is monitoring the L1 KeyStore contract and forwards the information to
/// the state manager or transaction pool.
#[derive(Debug)]
pub struct Indexer {
    provider: RootProvider<Http<Client>>,

    start_block: u64,
    blocks_batch_size: u64,
    keystore_address: Address,

    state_manager_sink: Sender<StateUpdate>,
    tx_pool_sink: Sender<ForcedTx>,
}

impl Indexer {
    /// Creates a new [Indexer].
    pub fn new(
        rpc_url: &str,
        start_block: u64,
        blocks_batch_size: u64,
        keystore_address: Address,
        state_manager_sink: Sender<StateUpdate>,
        tx_pool_sink: Sender<ForcedTx>,
    ) -> Result<Self> {
        let rpc_url = rpc_url.parse()?;
        let provider = ProviderBuilder::new().on_http(rpc_url);

        Ok(Self {
            provider,
            start_block,
            blocks_batch_size,
            keystore_address,
            state_manager_sink,
            tx_pool_sink,
        })
    }

    /// Runs the [Indexer] to monitor the L1 KeyStore contract.
    pub async fn run(&self) -> Result<()> {
        let mut from_block = self.start_block;
        loop {
            let latest_block = self.provider.get_block_number().await?;
            let to_block = min(from_block + self.blocks_batch_size - 1, latest_block);

            // If we've caught up, wait for a few seconds.
            if from_block >= to_block {
                sleep(Duration::from_secs(15)).await;
                continue;
            }

            // Fetch and process events.
            self.fetch_and_process_events(from_block, to_block).await?;

            // Move to the next batch
            from_block = to_block + 1;
        }
    }

    /// Fetches all events emitted by the L1 KeyStore contract in the range (`from_block`..=`to_block`)
    /// and handles them appropriately.
    async fn fetch_and_process_events(&self, from_block: u64, to_block: u64) -> Result<()> {
        // Create a filter for logs emitted by the Keystore contract in the block range.
        let filter = Filter::new()
            .address(self.keystore_address)
            .from_block(from_block)
            .to_block(to_block);

        // Fetch all logs in the block range for the contract
        let logs = self.provider.get_logs(&filter).await?;

        // Process each log.
        for log in logs {
            if let Ok(vk_registered) = log.log_decode::<VkRegistered>() {
                self.handle_vk_registered(vk_registered).await?;
            } else if let Ok(forced_tx_registered) = log.log_decode::<ForcedTxRegistered>() {
                self.handle_forced_tx_registered(forced_tx_registered)
                    .await?;
            } else if let Ok(batch_proved) = log.log_decode::<BatchProved>() {
                self.handle_batch_proved(batch_proved).await?;
            } else {
                // Unknown event.
                println!("Unknown event detected: {:?}", log);
            }
        }

        Ok(())
    }

    /// Wraps the [VkRegistered] event in a [StateUpdate::VerifyingKey] and forwards it to the state manager.
    async fn handle_vk_registered(&self, event: Log<VkRegistered>) -> Result<()> {
        let msg = StateUpdate::VerifyingKey(event.inner.data);
        self.state_manager_sink.send(msg).await?;
        Ok(())
    }

    /// Wraps the [ForcedTxRegistered] event in a [ForcedTx] and forwards it to the transaction pool.
    async fn handle_forced_tx_registered(&self, event: Log<ForcedTxRegistered>) -> Result<()> {
        let msg = ForcedTx(event.inner.data);
        self.tx_pool_sink.send(msg).await?;
        Ok(())
    }

    /// Wraps the [BatchProved] event in a [StateUpdate::Records] and forwards it to the state manager.
    async fn handle_batch_proved(&self, event: Log<BatchProved>) -> Result<()> {
        let msg = StateUpdate::Records(event.inner.data);
        self.state_manager_sink.send(msg).await?;
        Ok(())
    }
}

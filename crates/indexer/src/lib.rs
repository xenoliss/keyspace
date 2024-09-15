use std::{cmp::min, time::Duration};

use alloy::{
    primitives::{Address, Bytes, U256},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{Filter, Log},
    transports::http::{Client, Http},
};
use anyhow::Result;
use tokio::{sync::mpsc::Sender, time::sleep};

mod keystore_bindings;
use keystore_bindings::KeyStore::{
    proved as ProvedEvent, transaction as TransactionEvent, vkSubmitted as VkSubmittedEvent,
    OffchainTransaction,
};

pub enum StateUpdate {
    VerifyingKey {
        hash: U256,
        vk: Bytes,
    },

    RecordUpdate {
        tx_hash: U256,
        root: U256,
        onchain_tx_count: U256,
        offchain_txs: Vec<OffchainTransaction>,
    },
}

pub struct Transaction {
    pub original_key: U256,
    pub new_key: U256,
    pub current_vk_hash: U256,
    pub current_data: Bytes,
    pub proof: Bytes,
    pub pending_tx_hash: U256,
}

pub struct Indexer {
    provider: RootProvider<Http<Client>>,

    start_block: u64,
    blocks_batch_size: u64,
    keystore_address: Address,

    state_manager_sink: Sender<StateUpdate>,
    tx_pool_sink: Sender<Transaction>,
}

impl Indexer {
    pub fn new(
        start_block: u64,
        blocks_batch_size: u64,
        keystore_address: Address,
        state_manager_sink: Sender<StateUpdate>,
        tx_pool_sink: Sender<Transaction>,
    ) -> Result<Self> {
        let rpc_url = "https://eth.llamarpc.com".parse()?;
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

    pub async fn start(&self) -> Result<()> {
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
            // Try to decode each event type.
            if let Ok(vk_submitted_event) = log.log_decode::<VkSubmittedEvent>() {
                self.handle_vk_submitted_event(vk_submitted_event).await?;
            } else if let Ok(transaction_event) = log.log_decode::<TransactionEvent>() {
                self.handle_transaction_event(transaction_event).await?;
            } else if let Ok(proved_event) = log.log_decode::<ProvedEvent>() {
                self.handle_proved_event(proved_event).await?;
            } else {
                // Unknown event.
                println!("Unknown event detected: {:?}", log);
            }
        }

        Ok(())
    }

    async fn handle_vk_submitted_event(&self, event: Log<VkSubmittedEvent>) -> Result<()> {
        let VkSubmittedEvent { vkHash, vk } = event.inner.data;

        let msg = StateUpdate::VerifyingKey { hash: vkHash, vk };

        self.state_manager_sink.send(msg).await?;
        Ok(())
    }

    async fn handle_transaction_event(&self, event: Log<TransactionEvent>) -> Result<()> {
        let TransactionEvent {
            originalKey,
            newKey,
            currentVkHash,
            currentData,
            proof,
            pendingTxHash,
        } = event.inner.data;

        let msg = Transaction {
            original_key: originalKey,
            new_key: newKey,
            current_vk_hash: currentVkHash,
            current_data: currentData,
            proof,
            pending_tx_hash: pendingTxHash,
        };

        self.tx_pool_sink.send(msg).await?;
        Ok(())
    }

    async fn handle_proved_event(&self, event: Log<ProvedEvent>) -> Result<()> {
        let ProvedEvent {
            txHash,
            root,
            onchainTxCount,
            offchainTxs,
        } = event.inner.data;

        let msg = StateUpdate::RecordUpdate {
            tx_hash: txHash,
            root,
            onchain_tx_count: onchainTxCount,
            offchain_txs: offchainTxs,
        };

        self.state_manager_sink.send(msg).await?;
        Ok(())
    }
}

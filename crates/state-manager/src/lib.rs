use tokio::sync::mpsc::Receiver;

use keyspace_indexer::StateUpdate;

pub struct StateManager {
    indexer_stream: Receiver<StateUpdate>,
}

impl StateManager {
    pub fn new(indexer_stream: Receiver<StateUpdate>) -> Self {
        Self { indexer_stream }
    }

    pub async fn run(&mut self) {
        while let Some(state_update) = self.indexer_stream.recv().await {
            match state_update {
                StateUpdate::VerifyingKey { .. } => {
                    todo!("Register the VK in the DB")
                }
                StateUpdate::RecordUpdate { .. } => {
                    todo!("Apply the update the IMT")
                }
            }
        }
    }
}

use anyhow::Result;
use sp1_sdk::ProverClient;

use crate::transaction::PendingTransaction;

/// The [TransactionVerifier] is responsible for performing straightforward checks as a first
/// layer of filter to remove invalid transactions.
pub struct TransactionVerifier {
    client: ProverClient,
}

impl TransactionVerifier {
    /// Creates a new [TransactionVerifier].
    pub fn new() -> Self {
        Self {
            client: ProverClient::new(),
        }
    }

    /// Verifies the provided [PendingTransaction] by verifiying its attached Record proof.
    pub fn verify_tx(&self, tx: &PendingTransaction) -> Result<()> {
        Ok(self.client.verify(&tx.proof, &tx.vk)?)
    }
}

use keyspace_keystore_bindings::bindings::KeyStore::{BatchProved, ForcedTransactionSubmitted};

/// This enum defines the different messages that the [crate::manager::StateManager] listen for.
pub enum StateManagerMessage {
    /// Wrapper around the [BatchProved] emitted by the L1 KeyStore contract.
    BatchProved(BatchProved),
    /// Wrapper around the [ForcedTransactionSubmitted] emitted by the L1 KeyStore contract.
    ForcedTransactionSubmitted(ForcedTransactionSubmitted),
}

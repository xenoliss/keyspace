use alloy::sol;

sol! {
    #[derive(Debug, Default)]
    /// @notice A KeySpace transaction that updates a specific record.
    struct Transaction {
        /// @dev The KeySpace id.
        bytes32 keySpaceId;
        /// @dev The current KeySpace record value stored.
        bytes32 currentValue;
        /// @dev The new KeySpace record value to store.
        bytes32 newValue;
        /// @dev The zkVM record program verifier key hash.
        bytes32 zkVmVkHash;
    }

    #[sol(rpc)]
    contract KeyStore {
        #[derive(Debug, Default)]
        /// @notice Emitted when a forced transaction is submitted to the contract.
        /// @param keySpaceId The KeySpace id.
        /// @param currentValue The KeySpace record current value.
        /// @param newValue The KeySpace record new value to set.
        /// @param zkVmVkHash The zkVM record program verifier key hash.
        /// @param proof The record program proof wrapped in a PLONK BN254.
        event ForcedTxSubmitted(
            bytes32 indexed keySpaceId,
            bytes32 indexed currentValue,
            bytes32 indexed newValue,
            bytes32 zkVmVkHash,
            bytes proof
        );

        #[derive(Debug, Default)]
        /// @notice Emitted when a batch fo transactions has been proved, advancing the KeySpace root.
        /// @param newRoot The new KeySpace root.
        /// @param forcedTxCommitmentProved The commitment of the latest forced transaction proved in this batch.
        /// @param forcedTxCount The number of forced transactions proved in this batch.
        /// @param txs The normal transactions proved in this batch.
        event BatchProved(
            bytes32 indexed newRoot, bytes32 indexed forcedTxCommitmentProved, uint256 forcedTxCount, Transaction[] txs
        );
    }
}

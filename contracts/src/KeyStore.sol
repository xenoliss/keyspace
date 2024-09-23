// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

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

contract KeyStore {
    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //                                             STATE                                              //
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    /// @notice The SP1 PLONK verifier address.
    address public immutable verifier;

    /// @notice The current state root of KeySpace.
    bytes32 public root;

    /// @notice The number of forced transactions waiting to be proved.
    uint256 public forcedTxPendingCount;

    /// @notice The latest forced transaction commitment.
    bytes32 public latestForcedTxCommitment;

    /// @notice The forced transaction commitments stored as a linked list.
    mapping(bytes32 txCommitment => bytes32 nextTxCommitment) public forcedTxCommitments;

    /// @notice The latest forced transaction commitment that has been proved.
    bytes32 public latestForcedTxCommitmentProved;

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //                                              EVENTS                                            //
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    /// @notice Emitted when a forced transaction is submitted to the contract.
    /// @param keySpaceId The KeySpace id.
    /// @param currentValue The KeySpace record current value.
    /// @param newValue The KeySpace record new value to set.
    /// @param zkVmVkHash The zkVM record program verifier key hash.
    /// @param proof The record program proof wrapped in a PLONK BN254.
    event ForcedTransactionSubmitted(
        bytes32 indexed keySpaceId,
        bytes32 indexed currentValue,
        bytes32 indexed newValue,
        bytes32 zkVmVkHash,
        bytes proof
    );

    /// @notice Emitted when a batch fo transactions has been proved, advancing the KeySpace root.
    /// @param newRoot The new KeySpace root.
    /// @param forcedTxCommitmentProved The commitment of the latest forced transaction proved in this batch.
    /// @param forcedTxCount The number of forced transactions proved in this batch.
    /// @param sequencedTxs The sequenced transactions proved in this batch.
    event BatchProved(
        bytes32 indexed newRoot,
        bytes32 indexed forcedTxCommitmentProved,
        uint256 forcedTxCount,
        Transaction[] sequencedTxs
    );

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //                                           CONSTRUCTOR                                          //
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    constructor(bytes32 root_, address verifier_) {
        root = root_;
        verifier = verifier_;
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //                                          VIEW FUNCTIONS                                        //
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    function validateProof(bytes calldata proof) public pure {}

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //                                        PUBLIC FUNCTIONS                                        //
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    /// @notice Submits a forced transaction.
    /// @param forcedTx The forced transaction to submit.
    /// @param proof The record program proof wrapped in a PLONK BN254.
    function submitForcedTransaction(Transaction calldata forcedTx, bytes calldata proof) external {
        validateProof(proof);

        bytes32 newForcedTxsCommitment = keccak256(
            abi.encodePacked(
                latestForcedTxCommitment,
                forcedTx.keySpaceId,
                forcedTx.currentValue,
                forcedTx.newValue,
                forcedTx.zkVmVkHash,
                proof
            )
        ) >> 8;

        forcedTxCommitments[latestForcedTxCommitment] = newForcedTxsCommitment;
        latestForcedTxCommitment = newForcedTxsCommitment;
        forcedTxPendingCount++;

        emit ForcedTransactionSubmitted({
            keySpaceId: forcedTx.keySpaceId,
            currentValue: forcedTx.currentValue,
            newValue: forcedTx.newValue,
            zkVmVkHash: forcedTx.zkVmVkHash,
            proof: proof
        });
    }

    /// @notice Proves a transactions batch (and forced tansactions when relevant), advancing the KeySpace state root.
    /// @param newRoot The new expected KeySpace root.
    /// @param forcedTxCount The number of forced transaction included in the `proof`.
    /// @param sequencedTxs The sequenced transactions included in the `proof`.
    /// @param proof The Batcher proof, wrapped in a PLONK BN254.
    function prove(bytes32 newRoot, uint256 forcedTxCount, Transaction[] calldata sequencedTxs, bytes calldata proof)
        external
    {
        // Ensure the forced transaction count is not invalid.
        require(forcedTxCount <= forcedTxPendingCount);

        // Ensure the prover has no choice but to also prove the forced transactions.
        require(forcedTxCount >= sequencedTxs.length || forcedTxCount == forcedTxPendingCount);

        // Follow the forced transaction commitments linked list until we reach the relevant one.
        bytes32 forcedTxCommitmentProved = latestForcedTxCommitmentProved;
        for (uint256 i; i < forcedTxCount; i++) {
            bytes32 forcedTxCommitment = forcedTxCommitments[forcedTxCommitmentProved];
            delete forcedTxCommitments[forcedTxCommitmentProved];

            forcedTxCommitmentProved = forcedTxCommitment;
        }

        // Compute the commitments to the provided transactions and merge them in a single commitment
        // that commits to both the forced and nirmal transactions.
        bytes32 allTxsCommitment = forcedTxCommitmentProved;
        for (uint256 i; i < sequencedTxs.length; i++) {
            allTxsCommitment = keccak256(
                abi.encodePacked(
                    allTxsCommitment,
                    sequencedTxs[i].keySpaceId,
                    sequencedTxs[i].currentValue,
                    sequencedTxs[i].newValue,
                    sequencedTxs[i].zkVmVkHash
                )
            ) >> 8;
        }

        // TODO: Plug the actual proof verification.
        // uint256[] memory publicInputs = new uint256[](3);
        // publicInputs[0] = root;
        // publicInputs[1] = newRoot;
        // publicInputs[2] = allTxsHash;
        // require(blockVerifier.Verify(proof, publicInputs), "proof is invalid");

        // Update storage to take into account the forced transactions that have been proven.
        latestForcedTxCommitmentProved = forcedTxCommitmentProved;
        forcedTxPendingCount -= forcedTxCount;

        // Update the state root.
        root = newRoot;

        emit BatchProved({
            newRoot: newRoot,
            forcedTxCommitmentProved: forcedTxCommitmentProved,
            forcedTxCount: forcedTxCount,
            sequencedTxs: sequencedTxs
        });
    }
}

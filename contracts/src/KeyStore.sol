// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {IVerifier} from "./Verifier.sol";

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
    bytes32 private constant BN254_FR_MODULUS = 0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001;
    bytes32 private constant BLS12_377_FR_MODULUS = 0x12ab655e9a2ca55660b44d1e5c37b00159aa76fed00000010a11800000000001;
    bytes32 private constant BLS12_377_FP_MODULES_LSB =
        0x1a22d9f300f5138f1ef3622fba094800170b5d44300000008508c00000000001;
    bytes32 private constant BLS12_377_FP_MODULES_MSB =
        0x0000000000000000000000000000000001ae3a4617c510eac63b05c06ca1493b;

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //                                             STATE                                              //
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    /// @notice The SP1 groth16 verifier address.
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
    event ForcedTxSubmitted(
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
    /// @param txs The normal transactions proved in this batch.
    event BatchProved(
        bytes32 indexed newRoot, bytes32 indexed forcedTxCommitmentProved, uint256 forcedTxCount, Transaction[] txs
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

    function validateProof(bytes calldata proof) public pure {
        require(proof.length == 1504, "invalid proof length");

        for (uint256 i = 0; i < 24; i++) {
            bytes16 u = bytes16(proof[i * 48:i * 48 + 16]);
            bytes32 l = bytes32(proof[i * 48 + 16:i * 48 + 48]);
            _validateBls12377Fp(u, l);
        }

        for (uint256 i = 0; i < 11; i++) {
            bytes32 v = bytes32(proof[i * 32 + 1152:i * 32 + 1184]);
            _validateBls12377Fr(v);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //                                        PUBLIC FUNCTIONS                                        //
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    /// @notice Submits a forced transaction.
    /// @param forcedTx The forced transaction to submit.
    /// @param proof The record program proof wrapped in a PLONK BN254.
    function submitForcedTransaction(Transaction calldata forcedTx, bytes calldata proof) external {
        _validateBn254Fr(forcedTx.keySpaceId);
        _validateBn254Fr(forcedTx.currentValue);
        _validateBn254Fr(forcedTx.newValue);
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

        emit ForcedTxSubmitted({
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
    /// @param txs The normal transactions included in the `proof`.
    /// @param proof The Batcher proof, wrapped in a PLONK BN254.
    function prove(bytes32 newRoot, uint256 forcedTxCount, Transaction[] calldata txs, bytes calldata proof) external {
        // TODO figure out how to prevent single proof griefing attacks (require a minimum txCount that decreases over
        // time?)

        // Ensure the forced transaction count is not invalid.
        require(forcedTxCount <= forcedTxPendingCount);

        // Ensure the prover has no choice but to also prove the forced transactions.
        require(forcedTxCount >= txs.length || forcedTxCount == forcedTxPendingCount);

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
        for (uint256 i; i < txs.length; i++) {
            allTxsCommitment = keccak256(
                abi.encodePacked(
                    allTxsCommitment, txs[i].keySpaceId, txs[i].currentValue, txs[i].newValue, txs[i].zkVmVkHash
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
            txs: txs
        });
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////
    //                                        INTERNAL FUNCTIONS                                      //
    ////////////////////////////////////////////////////////////////////////////////////////////////////

    function _validateBn254Fr(bytes32 v) internal pure {
        require(v < BN254_FR_MODULUS, "value out of range");
    }

    function _validateBls12377Fr(bytes32 v) internal pure {
        require(v < BLS12_377_FR_MODULUS, "value out of range");
    }

    function _validateBls12377Fp(bytes16 u, bytes32 l) internal pure {
        if (u == BLS12_377_FP_MODULES_MSB) {
            require(l < BLS12_377_FP_MODULES_LSB, "value out of range");
        } else {
            require(u < BLS12_377_FP_MODULES_MSB, "value out of range");
        }
    }
}

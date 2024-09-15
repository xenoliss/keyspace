use alloy::sol;

sol! {
    #[sol(rpc)]
    contract KeyStore {
        #[derive(Debug)]
        struct OffchainTransaction {
            uint256 originalKey;
            uint256 newKey;
        }

        #[derive(Debug)]
        event vkSubmitted(uint256 indexed vkHash, bytes vk);

        #[derive(Debug)]
        event transaction(
            uint256 indexed originalKey,
            uint256 indexed newKey,
            uint256 currentVkHash,
            bytes currentData,
            bytes proof,
            uint256 pendingTxHash
        );

        #[derive(Debug)]
        event proved(
            uint256 indexed txHash, uint256 indexed root, uint256 onchainTxCount, OffchainTransaction[] offchainTxs
        );
    }
}

use alloy::sol;

sol! {
    #[sol(rpc)]
    contract KeyStore {
        #[derive(Debug, Default)]
        struct Transaction {
            uint256 keySpaceId;
            uint256 value;
        }

        #[derive(Debug, Default)]
        event VkRegistered(uint256 indexed vkHash, bytes vk);

        #[derive(Debug, Default)]
        event ForcedTxSubmitted(
            uint256 indexed keySpaceId,
            uint256 indexed value,
            uint256 currentVkHash,
            bytes currentData,
            bytes proof,
            uint256 pendingTxHash
        );

        #[derive(Debug, Default)]
        event BatchProved(
            uint256 indexed txHash, uint256 indexed root, uint256 forcedTxCount, Transaction[] txs
        );
    }
}

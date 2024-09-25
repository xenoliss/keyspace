// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";

import {KeyStore, Transaction} from "../src/KeyStore.sol";

contract KeyStoreTest is Test {
    bytes32 constant IMT_INITIAL_ROOT = 0xdd8c15c9791e3b56d7bf484214486d9dde59067d3ff02dd55f0336614b04e7c1;
    address constant SP1_VERIFIER_GROTH16 = 0x8dB92f28D7C30154d38E55DbA1054b5A7Fc5A829;
    bytes32 constant BATCHER_VK_HASH = 0x0039bb1d38495bef62376dfc582ff1266958c6df88f6e4a44ba03dde801b4001;

    function testForkBaseSepolia() public {
        vm.createSelectFork("https://sepolia.base.org");

        KeyStore sut = new KeyStore(IMT_INITIAL_ROOT, SP1_VERIFIER_GROTH16, BATCHER_VK_HASH);

        Transaction[] memory sequencedTxs = new Transaction[](5);
        sequencedTxs[0] = Transaction({
            keySpaceId: 0xdbe3f49e9aefd6c76d967a2b41d2f33e33ade1ebfa182382e8e44c32fa4d593c,
            currentValue: 0xdbe3f49e9aefd6c76d967a2b41d2f33e33ade1ebfa182382e8e44c32fa4d593c,
            newValue: 0xfd9c37f09512bb082270f8fb4f5e958035e8be7128cb1845723d93f73c1f1f32
        });
        sequencedTxs[1] = Transaction({
            keySpaceId: 0x75fa15006c83a249300f1fc72d6bfc2219ee1b56be4966a8c943625a069d649e,
            currentValue: 0x75fa15006c83a249300f1fc72d6bfc2219ee1b56be4966a8c943625a069d649e,
            newValue: 0xf86540c7b1c344c22b719918db6c4b1da4e641021fac02690d547ca044e12e7a
        });
        sequencedTxs[2] = Transaction({
            keySpaceId: 0x48f1e697ab436390fb56a37f7bd58123f413e9177f6e6a765feb36101a9caf02,
            currentValue: 0x48f1e697ab436390fb56a37f7bd58123f413e9177f6e6a765feb36101a9caf02,
            newValue: 0x9ba0f3ddb92dba59265801ab81ba7bb1524a153aa3dc9220ad965c10fb355908
        });
        sequencedTxs[3] = Transaction({
            keySpaceId: 0x86d1c718b92586e3cc2654ad35ef0021d28fb73a47aa18aba811ffcb6cc50f23,
            currentValue: 0x86d1c718b92586e3cc2654ad35ef0021d28fb73a47aa18aba811ffcb6cc50f23,
            newValue: 0xeec0d24800d8b4b102290bdd124d4125e936d4e5341209d62e3fe6b740d3b1b4
        });
        sequencedTxs[4] = Transaction({
            keySpaceId: 0x9cc5767c28a4a68b8ebdf8d1d3663462b7fdb5b81019dca7704955b366a2064c,
            currentValue: 0x9cc5767c28a4a68b8ebdf8d1d3663462b7fdb5b81019dca7704955b366a2064c,
            newValue: 0x1bf6e6889fa947a71759488ab61cd8bc714bd79261b6a0ce8fbd7faac82ecc53
        });

        sut.prove({
            newRoot: 0xae6af63e876648a0465c07ab2050a449c30938ba7d35ef03fef5e3e6dc3e4fa8,
            forcedTxCount: 0,
            sequencedTxs: sequencedTxs,
            proof: hex"5a1551d602591904eb716a28e9a07b96286dbd9023fb2c8eda5ffca2d62459d0b0b729791b7e4c7048de71ffaca3ee354660c91222ca95f56c9a4ac77437c5fe35b797410a72d4393b34a80b882d454c4b80a6d87731a4c4f6ce93da26427a2b466d44d723ec6d67ad39092245853a29d69c8845077e480a62958d7c74caa6041e5d73472d4d71d9aa6725b52718c4cc5ccb10ddffe2d7e0d103b4230dc26eea3a92c1ea06062427ab5fc4b793be33bdd5b1fdd3e7c7c0c86a09075d86c2d45fd84d5cd20f07497bce111afbb0c2cf07dc1169cff008a381f26e4303f90a8a673ba7c51c114b9c1fa33c13a46eb916aae9ef0a86243ab97d97ee3703dbae41ea08981446"
        });
    }
}

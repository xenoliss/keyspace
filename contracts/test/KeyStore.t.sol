// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import {Test} from "forge-std/Test.sol";

import {KeyStore, Transaction} from "../src/KeyStore.sol";

contract KeyStoreTest is Test {
    bytes32 constant IMT_INITIAL_ROOT = 0xdd8c15c9791e3b56d7bf484214486d9dde59067d3ff02dd55f0336614b04e7c1;
    address constant SP1_VERIFIER_GROTH16 = 0x8dB92f28D7C30154d38E55DbA1054b5A7Fc5A829;
    bytes32 constant BATCHER_VK_HASH = 0x003a03bfa18b75a087808b597bbafcd389b8c2ff29861fbfd38d45e1165a0060;

    function testForkBaseSepolia() public {
        vm.createSelectFork("https://sepolia.base.org");

        KeyStore sut = new KeyStore(IMT_INITIAL_ROOT, SP1_VERIFIER_GROTH16, BATCHER_VK_HASH);

        Transaction[] memory sequencedTxs = new Transaction[](5);
        sequencedTxs[0] = Transaction({
            keySpaceId: 0x432b85e947bd04da3bcca7ebc6f204ec247a8e934fafc142d8b8994c440dcbd8,
            currentValue: 0x432b85e947bd04da3bcca7ebc6f204ec247a8e934fafc142d8b8994c440dcbd8,
            newValue: 0x52f49d3fbcbecd9f3b8049eca2003f9674d369a8691eb9f72658d87c5bc37e40
        });
        sequencedTxs[1] = Transaction({
            keySpaceId: 0xadbf70b8d6202c7ea20a441d6687296dc8c9a4b6ac27dc306a57eee8fb4d59da,
            currentValue: 0xadbf70b8d6202c7ea20a441d6687296dc8c9a4b6ac27dc306a57eee8fb4d59da,
            newValue: 0xec4d771a3be17168509d3a0ca444f8ff3ac3c65f3fcbcb1e1b5074a391464533
        });
        sequencedTxs[2] = Transaction({
            keySpaceId: 0x4b707a287385291e1baf0a3b949cc79bf18fc3eed6b42cff29b07059939a181e,
            currentValue: 0x4b707a287385291e1baf0a3b949cc79bf18fc3eed6b42cff29b07059939a181e,
            newValue: 0x63c53928e3b58779114105b0725341c41d71e024778ae0668bccc0a28911f486
        });
        sequencedTxs[3] = Transaction({
            keySpaceId: 0x93ce6d3a1b9c4d984ccd46bc54c202a43a98912ddd5c80374ae8619106692fbf,
            currentValue: 0x93ce6d3a1b9c4d984ccd46bc54c202a43a98912ddd5c80374ae8619106692fbf,
            newValue: 0xc4ce3d022f7412e7b9c776efbc7eefc3531e31964a098c81302de9005a94699a
        });
        sequencedTxs[4] = Transaction({
            keySpaceId: 0x50dfedfbfec913837a7fb86f2b534db1b74d5faba9382f6d9fb7b5b134489a0e,
            currentValue: 0x50dfedfbfec913837a7fb86f2b534db1b74d5faba9382f6d9fb7b5b134489a0e,
            newValue: 0xa86b49dc831795a21f46bf55dacde48171a102a2697429112e45f0a9c6e4ff61
        });

        sut.prove({
            newRoot: 0x96cc5f5ad00d70150fed5ee511b16b91d0315c2f0a7460887174265a8191bcc4,
            forcedTxCount: 0,
            sequencedTxs: sequencedTxs,
            proof: hex"5a1551d607b12f697bc1a87af86bcdbbbf5fec1d59c1659efeb5bd96f9d3890b3ebfa95b001cb358b1ee95c520c6f71a10e8876029b327f94035bcb8dc9dcfa5de4126e20241a727e3db98ef59028db27e92ff13ea92e4e05c41a0ff8e1f8f0d3a50f91705bbd051052b869aa8d4334eaf5909df7c86a58eaadf0ca56a6ec91fb2dcf12b226299cd2e167d33bfed771df5c7143055858da3a443c7b136365ff45212a283192f7363a67a119daffe61a2c44b2398031822339b19eb228b74a1e56924954e1ba91b8609f83f189e59529471642f56e250688ce3f71f4d488a8f4aa8bb127918350e6e6e4580257be45e807974411cb34126f86e8d4e42dbad0b8c29fb861e"
        });
    }
}

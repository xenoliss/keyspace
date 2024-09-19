// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

import {KeyStore, Transaction} from "../src/KeyStore.sol";

contract ProveScript is Script {
    function run() public {
        vm.broadcast();
        KeyStore k = KeyStore(address(0x5FbDB2315678afecb367f032d93F642f64180aa3));

        Transaction[] memory txs = new Transaction[](1);
        txs[0] = Transaction({
            keySpaceId: bytes32(uint256(4)),
            currentValue: bytes32(uint256(4)),
            newValue: bytes32(uint256(4)),
            zkVmVkHash: bytes32(uint256(4))
        });

        k.prove({newRoot: 0, forcedTxCount: 0, txs: txs, proof: ""});
    }
}

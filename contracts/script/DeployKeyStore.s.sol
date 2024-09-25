// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

import {KeyStore} from "../src/KeyStore.sol";

contract DeployKeyStoreScript is Script {
    function run(bytes32 root, address sp1VerifierGroth16, bytes32 batcherVkHash) public {
        vm.broadcast();
        KeyStore k = new KeyStore(root, sp1VerifierGroth16, batcherVkHash);
        console.log("KeyStore deployed at ", address(k));
    }
}

// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

import {KeyStore} from "../src/KeyStore.sol";

contract DeployKeyStoreScript is Script {
    function run() public {
        vm.broadcast();
        KeyStore k = new KeyStore(0, address(0));

        console.log("KeyStore deployed at ", address(k));
    }
}

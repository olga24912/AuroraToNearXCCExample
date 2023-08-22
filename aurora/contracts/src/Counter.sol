pragma solidity ^0.8.0;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {AuroraSdk, NEAR, PromiseCreateArgs} from "@auroraisnear/aurora-sdk/aurora-sdk/AuroraSdk.sol";

contract Counter {
    using AuroraSdk for NEAR;    

    uint64 constant COUNTER_NEAR_GAS = 10_000_000_000_000;
    
    NEAR public near;
    string counterAccountId;

    constructor(address wnearAddress, string memory counterNearAccountId) {
        near = AuroraSdk.initNear(IERC20(wnearAddress));
        counterAccountId = counterNearAccountId;
    }

    function incrementXCC() external {
        PromiseCreateArgs memory callCounter = near.call(
            counterAccountId,
            "increment",
            "",
            0,
            COUNTER_NEAR_GAS
        );
        callCounter.transact();
    }
}

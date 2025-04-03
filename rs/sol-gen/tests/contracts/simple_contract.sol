// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

interface ITestContract {
    function createPrg(uint128 _value) external returns (bytes32);
    function svc1DoThis(uint128 _value, uint32 p1, string memory p2) external returns (bytes32);
}

contract TestContract is ITestContract {
    function createPrg(uint128 _value) external returns (bytes32) {}
    function svc1DoThis(uint128 _value, uint32 p1, string memory p2) external returns (bytes32) {}
}

interface ITestContractCallback {
    function replyOn_createPrg(bytes32 _messageId) external;
    function replyOn_svc1DoThis(bytes32 _messageId, uint32 _reply) external;
    function onErrorReply(bytes32 _messageId, bytes calldata _payload, bytes4 _replyCode) external;
}

contract TestContractCallback {
    ITestContract public immutable gearexeProgram;

    constructor(ITestContract _gearexeProgram) {
        gearexeProgram = _gearexeProgram;
    }

    modifier onlyGearexeProgram() {
        require(msg.sender == address(gearexeProgram), "Only Gear.exe program can call this function");
        _;
    }

    function replyOn_createPrg(bytes32 _messageId) external onlyGearexeProgram {
        // TODO: implement this
    }
    function replyOn_svc1DoThis(bytes32 _messageId, uint32 _reply) external onlyGearexeProgram {
        // TODO: implement this
    }
    function onErrorReply(bytes32 _messageId, bytes calldata _payload, bytes4 _replyCode) external onlyGearexeProgram {
        // TODO: implement this
    }
}

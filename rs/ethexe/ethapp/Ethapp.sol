// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

interface IEthapp {
    function createPayable(bool _callReply) external returns (bytes32 messageId);

    function createPrg(bool _callReply) external returns (bytes32 messageId);

    function svc1DoThis(bool _callReply, uint32 p1, string calldata P2) external returns (bytes32 messageId);

    function svc1DoThisPayable(bool _callReply, uint32 p1) external payable returns (bytes32 messageId);

    function svc1This(bool _callReply, address p1) external returns (bytes32 messageId);
}

contract EthappAbi is IEthapp {
    function createPayable(bool _callReply) external returns (bytes32 messageId) {}

    function createPrg(bool _callReply) external returns (bytes32 messageId) {}

    function svc1DoThis(bool _callReply, uint32 p1, string calldata P2) external returns (bytes32 messageId) {}

    function svc1DoThisPayable(bool _callReply, uint32 p1) external payable returns (bytes32 messageId) {}

    function svc1This(bool _callReply, address p1) external returns (bytes32 messageId) {}
}

interface IEthappCallbacks {
    function replyOn_createPayable(bytes32 messageId) external;

    function replyOn_createPrg(bytes32 messageId) external;

    function replyOn_svc1DoThis(bytes32 messageId, uint32 reply) external;

    function replyOn_svc1DoThisPayable(bytes32 messageId, uint32 reply) external;

    function replyOn_svc1This(bytes32 messageId, address reply) external;

    function onErrorReply(bytes32 messageId, bytes calldata payload, bytes4 replyCode) external payable;
}

contract EthappCaller is IEthappCallbacks {
    IEthapp public immutable VARA_ETH_PROGRAM;

    error UnauthorizedCaller();

    constructor(IEthapp _varaEthProgram) {
        VARA_ETH_PROGRAM = _varaEthProgram;
    }

    modifier onlyVaraEthProgram() {
        _onlyVaraEthProgram();
        _;
    }

    function _onlyVaraEthProgram() internal view {
        if (msg.sender != address(VARA_ETH_PROGRAM)) {
            revert UnauthorizedCaller();
        }
    }

    function replyOn_createPayable(bytes32 messageId) external onlyVaraEthProgram {
        // TODO: implement this
    }

    function replyOn_createPrg(bytes32 messageId) external onlyVaraEthProgram {
        // TODO: implement this
    }

    function replyOn_svc1DoThis(bytes32 messageId, uint32 reply) external onlyVaraEthProgram {
        // TODO: implement this
    }

    function replyOn_svc1DoThisPayable(bytes32 messageId, uint32 reply) external onlyVaraEthProgram {
        // TODO: implement this
    }

    function replyOn_svc1This(bytes32 messageId, address reply) external onlyVaraEthProgram {
        // TODO: implement this
    }

    function onErrorReply(bytes32 messageId, bytes calldata payload, bytes4 replyCode) external payable onlyVaraEthProgram {
        // TODO: implement this
    }
}

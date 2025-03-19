// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SimpleCTF {
    bool public won;
    bool public flag;

    function solve(uint256 value) public {
        if ((value % 10 == 0) && flag) {
            won = true;
        }
    }

    function setFlag(bool active) public {
        flag = active;
    }
}

contract SimpleInvariantCheck {
    SimpleCTF public ctf;

    constructor() {
        ctf = new SimpleCTF();
    }

    function invariant_unsolvable() public view returns (bool) {
        return !ctf.won();
    }

    function solveWrapper(uint256 value) public {
        ctf.solve(value);
    }

    function setFlagWrapper(bool active) public {
        ctf.setFlag(active);
    }
}

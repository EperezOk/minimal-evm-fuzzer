// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

contract SimpleCTF {
    bool public won;

    function solve(uint256 value, bool flag) public {
        if ((value % 10 == 0) && flag) {
            won = true;
        }
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

    function solveWrapper(uint256 value, bool flag) public {
        ctf.solve(value, flag);
    }
}

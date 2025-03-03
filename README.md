# Rust EVM Fuzzer

## Idea

Implement a minimalistic property-based fuzzer for EVM programs (ie. smart contracts).

The fuzzer should:
- [X] Take a target contract as input.

- [X] Deploy the contract on a local network.
  - The contract could have a constructor or set up function (without parameters) to initialize its state.

- [X] Inspect the contract ABI to look for:
   - Functions beginning with `invariant_` to check invariants. These should return a boolean to indicate if the invariant holds (true) or not (false).
   - Non-view functions to fuzz.

- [X] Call the functions to fuzz with random inputs, and check the invariants after each call.

- [X] If an invariant fails, the fuzzer should log the failing invariant and the sequence of function calls, along with the inputs that caused the invariant to fail.

- [X] Add a `max_steps` parameter to limit the number of function calls before stopping the fuzzer if no invariant fails.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Foundry](https://book.getfoundry.sh/getting-started/installation) (for compiling and deploying contracts on a local network)

## Usage

Run the fuzzer with the following command:

```bash
cargo run -- <target_contract_path> <target_contract_name> [max_steps]
```

For example:

```bash
cargo run -- examples/SimpleInvariantCheck.sol SimpleInvariantCheck 50
```

> You can find an example contract to fuzz in the [`examples`](./examples/) directory.

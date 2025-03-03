# Rust EVM Fuzzer

## Idea

Implement a minimalistic property-based fuzzer for EVM programs (ie. smart contracts).

The fuzzer should:
- Take a target contract as input.

- Deploy the contract on a local network.
  - The contract could have a constructor or set up function (without parameters) to initialize its state.

- Inspect the contract ABI to look for:
   - Functions beginning with `invariant_` to check invariants. These should return a boolean to indicate if the invariant holds (true) or not (false).
   - Non-view functions to fuzz.

- Call the functions to fuzz with random inputs, and check the invariants after each call.

- If an invariant fails, the fuzzer should log the failing invariant and the sequence of function calls, along with the inputs that caused the invariant to fail.
  - Add a `max_steps` parameter to limit the number of function calls before stopping the fuzzer if no invariant fails.

## Usage

Run the fuzzer with the following command:

```bash
cargo run -- <target_contract_path> <target_contract_name>
```

For example:

```bash
cargo run -- contracts/SimpleInvariantCheck.sol SimpleInvariantCheck
```

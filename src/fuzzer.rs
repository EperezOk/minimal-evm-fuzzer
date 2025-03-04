use alloy::{
    dyn_abi::DynSolValue,
    primitives::{Address, I256, U160, U256},
};
use eyre::Result;
use rand::{Rng, RngCore};

use crate::contract::{find_fuzz_targets, find_properties, TargetContract};

/// Run a fuzzing campaign against a contract
pub async fn run_campaign(contract: &TargetContract, max_steps: usize) -> Result<Option<String>> {
    // Get all properties and fuzz targets from the contract
    let properties = find_properties(contract);
    let fuzz_targets = find_fuzz_targets(contract);

    if fuzz_targets.is_empty() || properties.is_empty() {
        return Ok(None);
    }

    let mut rng = rand::thread_rng();

    for _ in 0..max_steps {
        // Execute a random target function with random inputs
        let target = &fuzz_targets[rng.gen_range(0..fuzz_targets.len())];

        let inputs: Vec<DynSolValue> = target
            .inputs
            .iter()
            .map(|param| gen_fuzz_input(&param.selector_type().to_string()))
            .collect();

        println!("Call: {}({:?})", target.name, inputs);

        contract
            .function(&target.name, &inputs)?
            .send()
            .await?
            .watch()
            .await?;

        // Check if the invariants hold
        for property in &properties {
            let holds = contract.function(&property.name, &[])?.call().await?;
            let holds = holds.first().unwrap().as_bool().unwrap();

            if !holds {
                return Ok(Some(property.name.clone()));
            }
        }
    }

    Ok(None)
}

/// Generate a random input value based on Solidity type
fn gen_fuzz_input(canonical_type: &str) -> DynSolValue {
    let mut rng = rand::thread_rng();

    match canonical_type {
        "bool" => DynSolValue::from(rng.gen_bool(0.5)),
        "address" => DynSolValue::from(Address::from(U160::from_be_slice(&get_rand_bytes(20)))),
        "uint256" => DynSolValue::from(U256::from_be_slice(&get_rand_bytes(32))),
        "int256" => DynSolValue::from(I256::try_from_be_slice(&get_rand_bytes(32)).unwrap()),
        // Add more types as needed
        other => panic!("Fuzzing type {} is not supported yet", other),
    }
}

/// Generate random bytes of specified length
fn get_rand_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);
    bytes
}

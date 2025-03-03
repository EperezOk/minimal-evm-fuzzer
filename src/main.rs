mod compiler;
mod contract;

use alloy::{
    dyn_abi::DynSolValue,
    json_abi::StateMutability,
    primitives::{Address, I256, U160, U256},
};
use eyre::Result;
use rand::{Rng, RngCore};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <contract_path> <contract_name> [max_steps]",
            args[0]
        );
        std::process::exit(1);
    }

    let contract_path = &args[1];
    let contract_name = &args[2];
    let max_steps = if args.len() > 3 { &args[3] } else { "100" };
    let max_steps = max_steps
        .parse::<usize>()
        .expect("Max steps argument must be a positive integer");

    // Compile target contract
    let (abi, bytecode) = compiler::compile_contract(contract_path, contract_name)?;

    // Deploy target contract
    let contract = contract::deploy_contract(abi, bytecode).await?;

    // Collect all properties and fuzz targets from the contract
    let properties: Vec<_> = contract
        .abi()
        .functions()
        .filter(|f| f.name.starts_with("invariant_"))
        .collect();

    let fuzz_targets: Vec<_> = contract
        .abi()
        .functions()
        .filter(|f| {
            !f.name.starts_with("invariant_")
                && f.state_mutability != StateMutability::Pure
                && f.state_mutability != StateMutability::View
        })
        .collect();

    let mut rng = rand::thread_rng();

    println!(
        "\nFuzzing contract {} (max steps = {})\n",
        contract_name, max_steps
    );

    for _ in 0..max_steps {
        // Execute a random target function with random inputs
        let target = &fuzz_targets[rng.gen_range(0..fuzz_targets.len())];

        let inputs: Vec<_> = target
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
                println!("\n{} broken 💥", property.name);
                return Ok(());
            }
        }
    }

    println!("\nAll invariants hold 🎉");

    Ok(())
}

fn gen_fuzz_input(canonical_type: &str) -> DynSolValue {
    let mut rng = rand::thread_rng();

    match canonical_type {
        "bool" => DynSolValue::from(rng.gen_bool(0.5)),
        "address" => DynSolValue::from(Address::from(U160::from_be_slice(&get_rand_bytes(20)))),
        "uint256" => DynSolValue::from(U256::from_be_slice(&get_rand_bytes(32))),
        "int256" => DynSolValue::from(I256::try_from_be_slice(&get_rand_bytes(32)).unwrap()),
        other => todo!("Fuzzing type {} is not supported yet", other),
    }
}

fn get_rand_bytes(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; length];
    rng.fill_bytes(&mut bytes);
    bytes
}

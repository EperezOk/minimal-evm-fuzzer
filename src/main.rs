use alloy::{
    contract::{ContractInstance, Interface}, dyn_abi::DynSolValue, json_abi::StateMutability, network::TransactionBuilder, primitives::{hex, Address, U160, U256, I256}, providers::{Provider, ProviderBuilder}, rpc::types::TransactionRequest
};
use eyre::Result;
use rand::{Rng, RngCore};
use std::path::PathBuf;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <contract_path> <contract_name> [max_steps]", args[0]);
        std::process::exit(1);
    }

    let contract_path = &args[1];
    let contract_name = &args[2];
    let max_steps = if args.len() > 3 { &args[3] } else { "100" };
    let max_steps = max_steps.parse::<usize>().expect("Max steps argument must be a positive integer");

    // Extract file name from contract path
    let file_name = PathBuf::from(contract_path).file_name().unwrap().to_str().unwrap().to_string();

    // Compile the contract
    let status = Command::new("forge")
        .args(&["build", contract_path, "--out", "artifacts", "--cache-path", "artifacts"])
        .status()?;

    if !status.success() {
        eprintln!("Failed to compile contract");
        std::process::exit(1);
    }

    // Spin up a local Anvil node.
    // Note: `anvil` must be installed and available in the PATH.
    let provider = ProviderBuilder::new().on_anvil_with_wallet();

    // Construct the artifact path
    let path = std::env::current_dir()?.join(format!("artifacts/{}/{}.json", file_name, contract_name));

    // Read the artifact which contains `abi`, `bytecode`, `deployedBytecode` and `metadata`
    let artifact = std::fs::read(path).expect("Failed to read artifact");
    let json: serde_json::Value = serde_json::from_slice(&artifact)?;

    // Get `abi` from the artifact
    let abi_value = json.get("abi").expect("Failed to get ABI from artifact");
    let abi = serde_json::from_str(&abi_value.to_string())?;

    // Get `bytecode` from the artifact
    let bytecode_value = json.get("bytecode").expect("Failed to get creation code from artifact");
    let bytecode_value = bytecode_value.get("object").unwrap();
    let bytecode = hex::decode(
        bytecode_value.as_str().unwrap().trim_start_matches("0x")
    )?;

    // Deploy the contract
    let tx = TransactionRequest::default().with_deploy_code(bytecode);
    let contract_address = provider.send_transaction(tx).await?.get_receipt().await?
        .contract_address.expect("Failed to get contract address");

    // Create a new `ContractInstance` from the abi
    let contract = ContractInstance::new(
        contract_address,
        provider.clone(),
        Interface::new(abi)
    );

    let properties: Vec<_> = contract.abi().functions()
        .filter(|f| f.name.starts_with("invariant_"))
        .collect();

    let target_functions: Vec<_> = contract.abi().functions()
        .filter(|f| !f.name.starts_with("invariant_")
                    && f.state_mutability != StateMutability::Pure
                    && f.state_mutability != StateMutability::View
        ).collect();

    let mut rng = rand::thread_rng();

    println!("\nFuzzing contract {} (max steps = {})\n", contract_name, max_steps);

    for _i in 0..max_steps {
        // Execute a random target function with random inputs
        let target_function = &target_functions[rng.gen_range(0..target_functions.len())];

        let args: Vec<_> = target_function.inputs.iter().map(
            |param| generate_fuzz_input(&param.selector_type().to_string())
        ).collect();

        println!("Call: {}({:?})", target_function.name, args);

        contract.function(&target_function.name, &args)?.send().await?.watch().await?;

        // Check if the invariants hold
        for property in &properties {
            let invariant_holds_value = contract.function(&property.name, &[])?.call().await?;
            let invariant_holds = invariant_holds_value.first().unwrap().as_bool().unwrap();

            if !invariant_holds {
                println!("\n{} broken 💥", property.name);
                return Ok(());
            }
        }
    }

    println!("\nAll invariants hold 🎉");

    Ok(())
}

fn generate_fuzz_input(canonical_type: &str) -> DynSolValue {
    let mut rng = rand::thread_rng();

    match canonical_type {
        "bool" => DynSolValue::from(rng.gen_bool(0.5)),
        "address" => DynSolValue::from(Address::from(U160::from_be_slice(&get_rand_bytes(20)))),
        "uint256" => DynSolValue::from(U256::from_be_slice(&get_rand_bytes(32))),
        "int256" => DynSolValue::from(I256::try_from_be_slice(&get_rand_bytes(32)).unwrap()),
        other => todo!("Fuzzing type {} is not supported yet", other),
    }
}

fn get_rand_bytes(num_bytes: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; num_bytes];
    rng.fill_bytes(&mut bytes);
    bytes
}

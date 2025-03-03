use alloy::{
    contract::{ContractInstance, Interface},
    dyn_abi::DynSolValue,
    json_abi::StateMutability,
    network::TransactionBuilder,
    primitives::{hex, Address, I256, U160, U256},
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
};
use eyre::Result;
use rand::{Rng, RngCore};
use std::path::PathBuf;
use std::process::Command;

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

    let file_name = PathBuf::from(contract_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Compile the contract using Forge
    Command::new("forge")
        .args(&[
            "build",
            contract_path,
            "--out",
            "artifacts",
            "--cache-path",
            "artifacts",
        ])
        .status()?;

    let out_path =
        std::env::current_dir()?.join(format!("artifacts/{}/{}.json", file_name, contract_name));

    // Read the artifact which contains the `abi` and `bytecode` of the target contract
    let artifact = std::fs::read(out_path).expect("Failed to read compilation artifact");
    let json: serde_json::Value = serde_json::from_slice(&artifact)?;

    let abi = json.get("abi").expect("Failed to get ABI from artifact");
    let abi = serde_json::from_str(&abi.to_string())?;

    let bytecode = json
        .get("bytecode")
        .expect("Failed to get creation code from artifact");
    let bytecode = bytecode.get("object").unwrap();
    let bytecode = hex::decode(bytecode.as_str().unwrap().trim_start_matches("0x"))?;

    // Spin up a local Anvil node and deploy the contract
    let provider = ProviderBuilder::new().on_anvil_with_wallet();

    let tx = TransactionRequest::default().with_deploy_code(bytecode);
    let contract_address = provider
        .send_transaction(tx)
        .await?
        .get_receipt()
        .await?
        .contract_address
        .expect("Failed to deploy target contract");
    let contract = ContractInstance::new(contract_address, provider.clone(), Interface::new(abi));

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

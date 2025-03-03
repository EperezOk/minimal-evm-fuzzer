use alloy::{
    contract::{ContractInstance, Interface},
    network::TransactionBuilder,
    primitives::hex,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
};
use eyre::Result;
use std::path::PathBuf;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <contract_path> <contract_name>", args[0]);
        std::process::exit(1);
    }

    let contract_path = &args[1];
    let contract_name = &args[2];

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
    let contract_address = provider
        .send_transaction(tx)
        .await?
        .get_receipt()
        .await?
        .contract_address
        .expect("Failed to get contract address");

    println!("Deployed contract at address: {}", contract_address);

    // Create a new `ContractInstance` from the abi
    let contract = ContractInstance::new(
        contract_address,
        provider.clone(),
        Interface::new(abi)
    );

    let properties = contract.abi().functions()
        .filter(|f| f.name.starts_with("invariant_"));

    // Check if the invariants hold
    for property in properties {
        let invariant_holds_value = contract.function(&property.name, &[])?.call().await?;
        let invariant_holds = invariant_holds_value.first().unwrap().as_bool().unwrap();

        println!("Check: {} {}", property.name, if invariant_holds { "holds" } else { "broken" });
    }

    Ok(())
}

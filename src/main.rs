mod cli;
mod compiler;
mod contract;
mod fuzzer;

use eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let (contract_path, contract_name, max_steps) = cli::parse_args()?;

    // Compile target contract
    let (abi, bytecode) = compiler::compile(&contract_path, &contract_name)?;

    // Deploy target contract
    let contract = contract::deploy(abi, bytecode).await?;

    println!(
        "\nFuzzing contract {} (max steps = {})\n",
        contract_name, max_steps
    );

    // Run fuzzer
    let broken_property = fuzzer::run_campaign(&contract, max_steps).await?;

    // Report results
    if broken_property.is_none() {
        println!("\nAll invariants hold 🎉");
    } else {
        println!("\n{} broken 💥", broken_property.unwrap());
    }

    Ok(())
}

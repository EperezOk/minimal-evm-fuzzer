use alloy::{json_abi::JsonAbi, primitives::hex};
use eyre::{eyre, Result};
use std::path::PathBuf;
use std::process::Command;

/// Compiles a Solidity contract using Forge and returns its ABI and bytecode
pub fn compile(contract_path: &str, contract_name: &str) -> Result<(JsonAbi, Vec<u8>)> {
    let file_name = PathBuf::from(contract_path)
        .file_name()
        .ok_or_else(|| eyre!("Invalid contract path"))?
        .to_str()
        .ok_or_else(|| eyre!("Invalid UTF-8 in file name"))?
        .to_string();

    // Compile the contract using Forge
    let status = Command::new("forge")
        .args(&[
            "build",
            contract_path,
            "--out",
            "artifacts",
            "--cache-path",
            "artifacts",
        ])
        .status()?;

    if !status.success() {
        return Err(eyre!("Forge build failed with status: {}", status));
    }

    let out_path =
        std::env::current_dir()?.join(format!("artifacts/{}/{}.json", file_name, contract_name));

    // Read the artifact which contains the `abi` and `bytecode` of the target contract
    let artifact = std::fs::read(&out_path).map_err(|e| eyre!("Failed to read artifact: {}", e))?;
    let json: serde_json::Value = serde_json::from_slice(&artifact)?;

    let abi = json
        .get("abi")
        .ok_or_else(|| eyre!("Failed to get ABI from artifact"))?;
    let abi = serde_json::from_str(&abi.to_string())?;

    let bytecode = json
        .get("bytecode")
        .ok_or_else(|| eyre!("Failed to get creation code from artifact"))?
        .get("object")
        .unwrap();
    let bytecode = hex::decode(bytecode.as_str().unwrap().trim_start_matches("0x"))?;

    Ok((abi, bytecode))
}

use alloy::{
    contract::{ContractInstance, Interface},
    json_abi::{Function, JsonAbi},
    network::TransactionBuilder,
    providers::{fillers, layers, Identity, Provider, ProviderBuilder, RootProvider},
    rpc::types::TransactionRequest,
};
use eyre::{eyre, Result};

/// Deploys a contract to a local Anvil instance and returns a contract instance
pub async fn deploy(abi: JsonAbi, bytecode: Vec<u8>) -> Result<TargetContract> {
    // Spin up a local Anvil node and deploy the contract
    let provider = ProviderBuilder::new().on_anvil_with_wallet();

    let tx = TransactionRequest::default().with_deploy_code(bytecode);
    let contract_address = provider
        .send_transaction(tx)
        .await?
        .get_receipt()
        .await?
        .contract_address
        .ok_or_else(|| eyre!("Failed to deploy target contract"))?;

    let contract = ContractInstance::new(contract_address, provider.clone(), Interface::new(abi));

    Ok(contract)
}

/// Extracts invariant functions from a contract
pub fn find_properties(contract: &TargetContract) -> Vec<&Function> {
    contract
        .abi()
        .functions()
        .filter(|f| f.name.starts_with("invariant_"))
        .collect()
}

/// Extracts fuzzable functions from a contract
pub fn find_fuzz_targets(contract: &TargetContract) -> Vec<&Function> {
    use alloy::json_abi::StateMutability;

    contract
        .abi()
        .functions()
        .filter(|f| {
            !f.name.starts_with("invariant_")
                && f.state_mutability != StateMutability::Pure
                && f.state_mutability != StateMutability::View
        })
        .collect()
}

#[rustfmt::skip]
pub type TargetContract = ContractInstance<fillers::FillProvider<fillers::JoinFill<fillers::JoinFill<Identity, fillers::JoinFill<fillers::GasFiller, fillers::JoinFill<fillers::BlobGasFiller, fillers::JoinFill<fillers::NonceFiller, fillers::ChainIdFiller>>>>, fillers::WalletFiller<alloy::network::EthereumWallet>>, layers::AnvilProvider<RootProvider>>>;

use alloy::{
    contract::{ContractInstance, Interface},
    json_abi::JsonAbi,
    providers::{Provider, ProviderBuilder, Identity, RootProvider, fillers, layers},
    network::TransactionBuilder,
    rpc::types::TransactionRequest,
};
use eyre::Result;

/// Deploys a contract to a local Anvil instance and returns a contract instance
pub async fn deploy_contract(
    abi: JsonAbi, 
    bytecode: Vec<u8>
) -> Result<TargetContract> {
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
        
    let contract = ContractInstance::new(
        contract_address, 
        provider.clone(), 
        Interface::new(abi)
    );
    
    Ok(contract)
}

type TargetContract = ContractInstance<fillers::FillProvider<fillers::JoinFill<fillers::JoinFill<Identity, fillers::JoinFill<fillers::GasFiller, fillers::JoinFill<fillers::BlobGasFiller, fillers::JoinFill<fillers::NonceFiller, fillers::ChainIdFiller>>>>, fillers::WalletFiller<alloy::network::EthereumWallet>>, layers::AnvilProvider<RootProvider>>>;

use anyhow::{anyhow, Result as AnyhowResult};
use ethers::contract::Abigen;
use risc0_zkvm::{Groth16Receipt, Receipt, ReceiptClaim};
use tracing::info;

use ethers::types::U256;
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
    types::TransactionReceipt,
};
use std::sync::Arc;
use std::time::Duration;

use crate::quantum_contract::{Proof, Quantum};

pub fn gen_quantum_structs() -> Result<(), Box<dyn std::error::Error>> {
    Abigen::new("Quantum", "quantum_contract/src/abi/Quantum.json")?
        .generate()?
        .write_to_file("quantum_contract/src/quantum_contract.rs")?;
    Ok(())
}

// TODO: remove unwrap
pub fn get_quantum_contract(
) -> AnyhowResult<Quantum<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>> {
    let private_key = std::env::var("PRIVATE_KEY")?;
    let rpc_endpoint = std::env::var("RPC_ENDPOINT")?;
    let chain_id = std::env::var("CHAIN_ID")?.parse::<u32>().unwrap();
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
    let provider =
        Provider::<Http>::try_from(&rpc_endpoint)?.interval(Duration::from_millis(10u64));
    let signer = Arc::new(SignerMiddleware::new(provider, wallet));
    let quantum_contract_address = &std::env::var("QUANTUM_CONTRACT_ADDRESS")?[2..];
    Ok(Quantum::new(
        quantum_contract_address.parse::<Address>().unwrap(),
        Arc::new(signer.clone()),
    ))
}

pub async fn update_quantum_contract_state(
    contract: &Quantum<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    batch_root: [u8;32],
    groth16_proof: &Groth16Receipt<ReceiptClaim>
) -> AnyhowResult<TransactionReceipt> {
    let proof = get_proof_from_groth16_proof(&groth16_proof)?;

    println!("--------------------------------------------------------------------------------");
    println!("calldata");
    println!("proof: {:?}", proof);
    println!("batch root: {:?}", batch_root);
    println!("--------------------------------------------------------------------------------");
    info!("calling verify_superproof");
    let receipt = contract.verify_superproof(proof, batch_root)
        .send()
        .await?
        .await?
        .ok_or(anyhow!("could not verify superproof"));
    return receipt;
}

pub fn get_proof_from_groth16_proof(groth16_proof: &Groth16Receipt<ReceiptClaim>) -> AnyhowResult<Proof> {
    let mut offset = 0;
    let a0 = U256::from_big_endian(&groth16_proof.seal[offset..offset + 32]);
    offset += 32;
    let a1 = U256::from_big_endian(&groth16_proof.seal[offset..offset + 32]);
    offset += 32;

    let b00 = U256::from_big_endian(&groth16_proof.seal[offset..offset + 32]);
    offset += 32;
    let b01 = U256::from_big_endian(&groth16_proof.seal[offset..offset + 32]);
    offset += 32;

    let b10 = U256::from_big_endian(&groth16_proof.seal[offset..offset + 32]);
    offset += 32;
    let b11 = U256::from_big_endian(&groth16_proof.seal[offset..offset + 32]);
    offset += 32;

    let c0 = U256::from_big_endian(&groth16_proof.seal[offset..offset + 32]);
    offset += 32;
    let c1 = U256::from_big_endian(&groth16_proof.seal[offset..offset + 32]);

    let proof = [a0, a1, b00, b01, b10, b11, c0, c1];

    Ok(Proof {
        proof,
    })
}

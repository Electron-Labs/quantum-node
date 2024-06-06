use anyhow::{anyhow, Result as AnyhowResult};
use ethers::contract::Abigen;
use quantum_types::types::gnark_groth16::GnarkGroth16Proof;
use tracing::{info, error};

use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use ethers::types::{Bytes, U256};
use ethers::{
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    types::Address,
    types::TransactionReceipt,
};

use crate::contract_utils::get_bytes_from_hex_string;
use crate::quantum_contract::{Proof, Quantum};

pub fn gen_quantum_structs() -> Result<(), Box<dyn std::error::Error>> {
    Abigen::new("Quantum", "quantum_contract/src/abi/Quantum.json")?
        .generate()?
        .write_to_file("quantum_contract/src/quantum_contract.rs")?;
    Ok(())
}

// TODO: remove unwrap
pub fn get_quantum_contract() -> AnyhowResult<Quantum<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>> {
    let private_key = std::env::var("PRIVATE_KEY")?;
    let rpc_endpoint = std::env::var("RPC_ENDPOINT")?;
    let chain_id = std::env::var("CHAIN_ID")?.parse::<u32>().unwrap();
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()?.with_chain_id(chain_id);
    let provider = Provider::<Http>::try_from(&rpc_endpoint)?.interval(Duration::from_millis(10u64));
    let signer = Arc::new(SignerMiddleware::new(provider, wallet));
    let quantum_contract_address = &std::env::var("QUANTUM_CONTRACT_ADDRESS")?[2..];
    Ok(Quantum::new(
        quantum_contract_address.parse::<Address>().unwrap(),
        Arc::new(signer.clone()),
    ))
}

pub async fn update_quantum_contract_state(
    contract: &Quantum<Arc<SignerMiddleware<Provider<Http>, LocalWallet>>>,
    new_state: &str,
    gnark_proof: &GnarkGroth16Proof
) -> AnyhowResult<Option<TransactionReceipt>> {

    let new_state_bytes = get_bytes_from_hex_string(new_state)?;
    let new_state_bytes: [u8;32] = new_state_bytes.as_slice()[..32].try_into()?;

    let proof = get_proof_from_gnark_groth16_proof(&gnark_proof)?;

    let receipt = contract
        .update_quantum_state(new_state_bytes, proof)
        .send().await?.await?;
    return Ok(receipt);
}

pub fn get_proof_from_gnark_groth16_proof(gnark_proof: &GnarkGroth16Proof) -> AnyhowResult<Proof> {

    let arx = U256::from_dec_str(&gnark_proof.Ar.X)?;
    let arx1 = U256::from_str(&gnark_proof.Ar.X)?;
    info!("arx from using form _dec_string: {:?}", arx);
    info!("arx from using from_string: {:?}",arx1 );
    
    let ary = U256::from_dec_str(&gnark_proof.Ar.Y)?;
    let bsx1 = U256::from_dec_str(&gnark_proof.Bs.X.A0)?;
    let bsx2 = U256::from_dec_str(&gnark_proof.Bs.X.A1)?;
    let bsy1 = U256::from_dec_str(&gnark_proof.Bs.Y.A0)?;
    let bsy2 = U256::from_dec_str(&gnark_proof.Bs.Y.A1)?;
    let krsx = U256::from_dec_str(&gnark_proof.Krs.X)?;
    let krsy = U256::from_dec_str(&gnark_proof.Krs.Y)?;


    let commitments_x = U256::from_dec_str(&gnark_proof.Commitments[0].X)?;
    let commitments_y = U256::from_dec_str(&gnark_proof.Commitments[0].Y)?;

    let commitment_pok_x =  U256::from_dec_str(&gnark_proof.CommitmentPok.X)?;
    let commitment_pok_y =  U256::from_dec_str(&gnark_proof.CommitmentPok.Y)?;

    let proof = [arx, ary, bsx2, bsx1, bsy2, bsy1, krsx, krsy];
    let commitments = [commitments_x, commitments_y];
    let commitment_pok = [commitment_pok_x, commitment_pok_y];

    Ok(Proof {
        proof,
        commitments,
        commitment_pok
    })
}

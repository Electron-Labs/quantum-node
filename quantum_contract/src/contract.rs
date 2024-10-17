use anyhow::{anyhow, Result as AnyhowResult};
use ethers::contract::Abigen;
use quantum_types::types::gnark_groth16::SuperproofGnarkGroth16Proof;
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
    gnark_proof: &SuperproofGnarkGroth16Proof,
) -> AnyhowResult<TransactionReceipt> {
    let proof = get_proof_from_gnark_groth16_proof(&gnark_proof)?;

    println!("--------------------------------------------------------------------------------");
    println!("final proof: {:?}", proof);
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

pub fn get_proof_from_gnark_groth16_proof(gnark_proof: &SuperproofGnarkGroth16Proof) -> AnyhowResult<Proof> {
    info!("gmarl+[ {:?}", gnark_proof);

    let arx = U256::from_dec_str(&gnark_proof.Ar.X).expect("arx");
    let arx1 = U256::from_dec_str(&gnark_proof.Ar.X).expect("arx1");
    info!("arx from using form _dec_string: {:?}", arx);
    info!("arx from using from_string: {:?}", arx1);

    let ary = U256::from_dec_str(&gnark_proof.Ar.Y)?;
    let bsx1 = U256::from_dec_str(&gnark_proof.Bs.X.A0)?;
    let bsx2 = U256::from_dec_str(&gnark_proof.Bs.X.A1)?;
    let bsy1 = U256::from_dec_str(&gnark_proof.Bs.Y.A0)?;
    let bsy2 = U256::from_dec_str(&gnark_proof.Bs.Y.A1)?;
    let krsx = U256::from_dec_str(&gnark_proof.Krs.X)?;
    let krsy = U256::from_dec_str(&gnark_proof.Krs.Y)?;

    // println!(
    //     "inputs {:?}",
    //     [
    //         &gnark_proof.Ar.X,
    //         &gnark_proof.Ar.Y,
    //         &gnark_proof.Bs.X.A1,
    //         &gnark_proof.Bs.X.A0,
    //         &gnark_proof.Bs.Y.A1,
    //         &gnark_proof.Bs.Y.A0,
    //         &gnark_proof.Krs.X,
    //         &gnark_proof.Krs.Y
    //     ]
    // );

    let commitments_x = U256::from_dec_str(&gnark_proof.Commitments[0].X)?;
    let commitments_y = U256::from_dec_str(&gnark_proof.Commitments[0].Y)?;

    let commitment_pok_x = U256::from_dec_str(&gnark_proof.CommitmentPok.X)?;
    let commitment_pok_y = U256::from_dec_str(&gnark_proof.CommitmentPok.Y)?;

    let proof = [arx, ary, bsx2, bsx1, bsy2, bsy1, krsx, krsy];
    let commitments = [commitments_x, commitments_y];
    let commitment_pok = [commitment_pok_x, commitment_pok_y];

    println!(
        "proof ->  {:?}",
        Proof {
            proof,
            commitments,
            commitment_pok
        }
    );

    Ok(Proof {
        proof,
        commitments,
        commitment_pok,
    })
}

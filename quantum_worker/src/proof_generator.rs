use std::{fs, str::FromStr, time::Duration};
use anyhow::{anyhow, Ok, Result as AnyhowResult};
use ark_bn254::{Bn254, Config, Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_groth16::{verifier, VerifyingKey, Proof as ArkProof};
use ark_serialize::CanonicalSerialize;
use num_bigint::BigUint;
use quantum_circuits_interface::ffi::interactor::QuantumV2CircuitInteractor;
use bonsai_sdk::blocking::Client;
// use bonsai_sdk::
use quantum_db::repository::{
    proof_repository::{update_reduction_data},
    reduction_circuit_repository::get_reduction_circuit_data_by_id,
    user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
};
use quantum_types::{
    enums::{proving_schemes::ProvingSchemes, task_type::TaskType},
    traits::{
        circuit_interactor::{CircuitInteractorFFI, GenerateReductionProofResult},
        pis::Pis,
        proof::Proof,
        vkey::Vkey,
    },
    types::{
        config::ConfigData, db::{proof::Proof as DBProof, task::Task, user_circuit_data}, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}, gnark_plonk::{GnarkPlonkSolidityProof, GnarkPlonkPis, GnarkPlonkVkey}, halo2_plonk::{Halo2PlonkPis, Halo2PlonkProof, Halo2PlonkVkey}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey}
    },
};
use quantum_utils::{error_line, file::{dump_object, read_bytes_from_file}};
use risc0_zkvm::{compute_image_id, default_prover, serde::to_vec, ExecutorEnv, Receipt};
use sqlx::{MySql, Pool};
use tokio::time::Instant;
use tracing::info;
use quantum_db::repository::proof_repository::get_proof_by_proof_id;
use quantum_types::types::db::reduction_circuit::ReductionCircuit;
use quantum_types::types::db::user_circuit_data::UserCircuitData;
use crate::{connection::get_pool, AVAIL_BH};
use crate::utils::dump_reduction_proof_data;

pub async fn handle_proof_generation_and_updation(
    proof_id: u64,
    proof_hash: &str,
    user_circuit_hash: &str,
    config: &ConfigData,
) -> AnyhowResult<()> {

    let (prove_result, reduction_time) = handle_proof_generation(proof_id).await?;

    // Dump reduced proof and public inputs
    // TODO change proof bytes and pis bytes values
    // let (reduced_proof_path, reduced_pis_path) = dump_reduction_proof_data(
    //     config,
    //     user_circuit_hash,
    //     &proof_hash,
    //     prove_result.reduced_proof,
    //     prove_result.reduced_pis,
    // )?;
    // info!("Dumped reduced proof and pis");

    // // update reduction data corresponding to proof
    // update_reduction_data(
    //     get_pool().await,
    //     proof_id,
    //     &reduced_proof_path,
    //     &reduced_pis_path,
    //     reduction_time,
    // )
    // .await?;
    info!("Updated reduction data to corresponding proof");
    Ok(())
}

async fn handle_proof_generation(proof_id: u64) ->AnyhowResult<(GenerateReductionProofResult, u64)>{
    let proof_data = get_proof_by_proof_id(get_pool().await, proof_id).await?;
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &proof_data.user_circuit_hash).await?;

    // Call proof generation to quantum_reduction_circuit
    let (prove_result, reduction_time) = generate_reduced_proof(&user_circuit_data, &proof_data).await?;

    if !prove_result.success {
        return Err(anyhow::Error::msg(error_line!(prove_result.msg)));
    }
    return Ok((prove_result, reduction_time))
}

async fn generate_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof ) -> AnyhowResult<(GenerateReductionProofResult, u64)> {
    // // Get outer_pk
    // let vk_path = &user_circuit_data.vk_path;
    // let vk = SnarkJSGroth16Vkey::read_vk(vk_path).unwrap();
    // println!("outer_pk_path :: {:?}", outer_pk_path);
    //
    // // Get outer_vk
    // let outer_vk_path = &reduction_circuit_data.vk_path[..];
    // let outer_vk = GnarkGroth16Vkey::read_vk(&outer_vk_path)?;
    // println!("outer_vk_path :: {:?}", outer_vk_path);

    // let reduction_start_time = Instant::now();
    let prove_result: GenerateReductionProofResult;
    let reduction_time: u64;

    info!("Calling gnark groth16 proof generation");
    // if user_circuit_data.proving_scheme == ProvingSchemes::GnarkGroth16 {
        // (prove_result, reduction_time) = generate_gnark_groth16_reduced_proof(user_circuit_data, proof_data, outer_pk_bytes, outer_vk).await?;
    // } else 
    if user_circuit_data.proving_scheme == ProvingSchemes::Groth16 {
        (prove_result, reduction_time) = generate_snarkjs_groth16_reduced_proof(user_circuit_data, proof_data).await?;
    } 
    // else if user_circuit_data.proving_scheme == ProvingSchemes::Halo2Plonk {
        // (prove_result, reduction_time) = generate_halo2_plonk_reduced_proof(user_circuit_data, proof_data, outer_pk_bytes, outer_vk).await?;
    // } 
    else {
        return Err(anyhow!(error_line!("unsupported proving scheme in proof reduction")));
    }

    // let reduction_time = reduction_start_time.elapsed().as_secs();
    info!("Reduced Proof successfully generated in {:?}", reduction_time);

    Ok((prove_result, reduction_time))
}

// async fn generate_gnark_groth16_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, outer_pk_bytes: Vec<u8>, outer_vk: GnarkGroth16Vkey) -> AnyhowResult<(GenerateReductionProofResult, u64)> {
//     // Get inner_proof
//     let inner_proof_path = &proof_data.proof_path;
//     println!("inner_proof_path :: {:?}", inner_proof_path);

//     // Get inner_vk
//     let inner_vk_path = &user_circuit_data.vk_path;
//     println!("inner_vk_path :: {:?}", inner_vk_path);

//     // Get inner_pis
//     let inner_pis_path = &proof_data.pis_path;
//     println!("inner_pis_path :: {:?}", inner_pis_path);
//     // 1.Reconstruct inner proof
//     let gnark_inner_proof: GnarkGroth16Proof =
//         GnarkGroth16Proof::read_proof(&inner_proof_path)?;
//     let gnark_inner_vk: GnarkGroth16Vkey = GnarkGroth16Vkey::read_vk(&inner_vk_path)?;
//     let gnark_inner_pis: GnarkGroth16Pis = GnarkGroth16Pis::read_pis(&inner_pis_path)?;

//     let reduction_start_time = Instant::now();

//     // 2.Call reduced proof generator for gnark inner proof
//     let prove_result = QuantumV2CircuitInteractor::generate_gnark_groth16_reduced_proof(
//         gnark_inner_proof,
//         gnark_inner_vk.clone(),
//         gnark_inner_pis.clone(),
//         outer_vk,
//         outer_pk_bytes,
//     );
//     let reduction_time = reduction_start_time.elapsed().as_secs();

//     // verify_proof_reduction_result(&prove_result, &user_circuit_data, gnark_inner_vk, gnark_inner_pis)?;
//     Ok((prove_result, reduction_time))
// }

async fn generate_snarkjs_groth16_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(GenerateReductionProofResult, u64)> {

    let vk = SnarkJSGroth16Vkey::read_vk(&user_circuit_data.vk_path)?;
    let proof = SnarkJSGroth16Proof::read_proof(&proof_data.proof_path)?;
    let public_inputs = SnarkJSGroth16Pis::read_pis(&proof_data.pis_path)?;

    let ark_vk = get_snark_js_ark_vk(vk)?;
    let pvk = verifier::prepare_verifying_key(&ark_vk);

    let ark_proof = get_snark_ark_proof(proof)?;
    let ark_public_inputs = get_snark_pis(public_inputs)?;


    let mut pvk_bytes = vec![];
    pvk.serialize_uncompressed(&mut pvk_bytes).unwrap();

    let mut proof_bytes = vec![];
    ark_proof.serialize_uncompressed(&mut proof_bytes).unwrap();

    let mut public_inputs_bytes = vec![];
    ark_public_inputs.serialize_uncompressed(&mut public_inputs_bytes).unwrap();


    let input_data = to_vec(&pvk_bytes).unwrap();
    let mut input_data_vec = bytemuck::cast_slice(&input_data).to_vec();

    let input_data = to_vec(&proof_bytes).unwrap();
    input_data_vec.extend_from_slice( bytemuck::cast_slice(&input_data));


    let input_data = to_vec(&public_inputs_bytes).unwrap();
    input_data_vec.extend_from_slice( bytemuck::cast_slice(&input_data));


    let client = Client::from_env(risc0_zkvm::VERSION)?;

    let input_id = client.upload_input(input_data_vec)?;

    let assumptions: Vec<String> = vec![];

    // Wether to run in execute only mode
    let execute_only = false;

    let image_id = "".to_string();
    // client.upload_img(&image_id, METHOD_ELF.to_vec())?;

    let session = client.create_session(image_id, input_id, assumptions, execute_only)?;
    loop {
        let res = session.status(&client)?;
        if res.status == "RUNNING" {
            eprintln!(
                "Current status: {} - state: {} - continue polling...",
                res.status,
                res.state.unwrap_or_default()
            );
            std::thread::sleep(Duration::from_secs(15));
            continue;
        }
        if res.status == "SUCCEEDED" {
            // Download the receipt, containing the output
            let receipt_url = res
                .receipt_url
                .expect("API error, missing receipt on completed session");

            let receipt_buf = client.download(&receipt_url)?;
            let receipt: Receipt = bincode::deserialize(&receipt_buf)?;
            // receipt
            //     .verify(METHOD_ID)
            //     .expect("Receipt verification failed");

            dump_object(receipt, "dump_receipt", "receipt_1").unwrap();
        } else {
            panic!(
                "Workflow exited: {} - | err: {}",
                res.status,
                res.error_msg.unwrap_or_default()
            );
        }

        break;
    }
    let resp = GenerateReductionProofResult {
        success: true,
        msg: "done".to_string(),
        reduced_proof: todo!(),
        reduced_pis: todo!(),   
    };


    // let prover = default_prover();

    // let prove_info = prover.prove(env, GROTH16_VERIFIER_ELF).unwrap();

    Ok((resp ,24 as u64))

    // // Get inner_proof
    // let inner_proof_path = &proof_data.proof_path;
    // println!("inner_proof_path :: {:?}", inner_proof_path);
    //
    // // Get inner_vk
    // let inner_vk_path = &user_circuit_data.vk_path;
    // println!("inner_vk_path :: {:?}", inner_vk_path);
    //
    // // Get inner_pis
    // let inner_pis_path = &proof_data.pis_path;
    // println!("inner_pis_path :: {:?}", inner_pis_path);
    // // 1.Reconstruct inner proof
    // let snarkjs_inner_proof: SnarkJSGroth16Proof =
    //     SnarkJSGroth16Proof::read_proof(&inner_proof_path)?;
    // let snarkjs_inner_vk: SnarkJSGroth16Vkey = SnarkJSGroth16Vkey::read_vk(&inner_vk_path)?;
    // let snarkjs_inner_pis: SnarkJSGroth16Pis = SnarkJSGroth16Pis::read_pis(&inner_pis_path)?;
    // // 2. Call reduced proof generator for circom inner proof
    //
    // let reduction_start_time = Instant::now();
    // let reduction_time = reduction_start_time.elapsed().as_secs();
    // Ok((prove_result, reduction_time))
}

async fn generate_halo2_plonk_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, outer_pk_bytes: Vec<u8>, outer_vk: GnarkGroth16Vkey) -> AnyhowResult<(GenerateReductionProofResult, u64)> {
    // Get inner_proof
    let inner_proof_path = &proof_data.proof_path;
    println!("inner_proof_path :: {:?}", inner_proof_path);

    // Get inner_vk
    let inner_vk_path = &user_circuit_data.vk_path;
    println!("inner_vk_path :: {:?}", inner_vk_path);

    // Get inner_pis
    let inner_pis_path = &proof_data.pis_path;
    println!("inner_pis_path :: {:?}", inner_pis_path);

    let inner_proof = Halo2PlonkProof::read_proof(&inner_proof_path)?;
    let inner_vk = Halo2PlonkVkey::read_vk(&inner_vk_path)?;
    let inner_pis = Halo2PlonkPis::read_pis(&inner_pis_path)?;

    let reduction_start_time = Instant::now();
    let prove_result = QuantumV2CircuitInteractor::generate_halo2_plonk_reduced_proof(
        inner_pis.clone(),
        inner_proof,
        inner_vk.clone(),
        outer_vk,
        outer_pk_bytes,
    );
    let reduction_time = reduction_start_time.elapsed().as_secs();

    // verify_proof_reduction_result(&prove_result, &user_circuit_data, inner_vk, inner_pis)?;
    Ok((prove_result, reduction_time))
}

async fn generate_gnark_plonk_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof, outer_pk_bytes: Vec<u8>, outer_vk: GnarkGroth16Vkey) -> AnyhowResult<(GenerateReductionProofResult, u64)> {
    // Get inner_proof
    let inner_proof_path = &proof_data.proof_path;
    println!("inner_proof_path :: {:?}", inner_proof_path);

    // Get inner_vk
    let inner_vk_path = &user_circuit_data.vk_path;
    println!("inner_vk_path :: {:?}", inner_vk_path);

    // Get inner_pis
    let inner_pis_path = &proof_data.pis_path;
    println!("inner_pis_path :: {:?}", inner_pis_path);
    // 1.Reconstruct inner proof
    let inner_proof = GnarkPlonkSolidityProof::read_proof(&inner_proof_path)?;
    let inner_vk = GnarkPlonkVkey::read_vk(&inner_vk_path)?;
    let inner_pis = GnarkPlonkPis::read_pis(&inner_pis_path)?;

    let reduction_start_time = Instant::now();

    let prove_result = QuantumV2CircuitInteractor::generate_gnark_plonk_reduced_proof(
        inner_proof,
        inner_vk.clone(),
        inner_pis.clone(),
        outer_vk,
        outer_pk_bytes,
        AVAIL_BH
    );
    let reduction_time = reduction_start_time.elapsed().as_secs();

    verify_proof_reduction_result(&prove_result, &user_circuit_data, inner_vk, inner_pis)?;
    Ok((prove_result, reduction_time))
}

// fn verify_proof_reduction_result<V: Vkey, P: Pis>(prove_result: &GenerateReductionProofResult, user_circuit_data: &UserCircuitData, inner_vk: V, inner_pis: P) -> AnyhowResult<()>{
//     let mut keccak_ip = Vec::<u8>::new();
//     let vkey_hash = inner_vk.extended_keccak_hash(user_circuit_data.n_commitments)?;
//     println!("vkey_hash {:?}", vkey_hash);
//     keccak_ip.extend(vkey_hash);
//     let pis_hash = inner_pis.extended_keccak_hash()?;
//     println!("pis_hash {:?}", pis_hash);
//     keccak_ip.extend(pis_hash);
//     let hash = keccak_hash::keccak(keccak_ip).0;
//     let pis1 = BigUint::from_bytes_be(&hash[0..16]).to_string();
//     let pis2 = BigUint::from_bytes_be(&hash[16..32]).to_string();
//     println!("pis1 {:?}", pis1);
//     println!("pis2 {:?}", pis2);
//     if prove_result.success {
//         // worker will panic here
//         assert_eq!(pis1, prove_result.reduced_pis.0[0]);
//         assert_eq!(pis2, prove_result.reduced_pis.0[1]);
// fn verify_proof_reduction_result<V: Vkey, P: Pis>(prove_result: &GenerateReductionProofResult, user_circuit_data: &UserCircuitData, inner_vk: V, inner_pis: P) -> AnyhowResult<()>{
//     let mut keccak_ip = Vec::<u8>::new();
//     let vkey_hash = inner_vk.extended_keccak_hash(user_circuit_data.n_commitments)?;
//     println!("vkey_hash {:?}", vkey_hash);
//     keccak_ip.extend(vkey_hash);
//     let pis_hash = inner_pis.extended_keccak_hash()?;
//     println!("pis_hash {:?}", pis_hash);
//     keccak_ip.extend(pis_hash);
//     let hash = keccak_hash::keccak(keccak_ip).0;
//     let pis1 = BigUint::from_bytes_be(&hash[0..16]).to_string();
//     let pis2 = BigUint::from_bytes_be(&hash[16..32]).to_string();
//     println!("pis1 {:?}", pis1);
//     println!("pis2 {:?}", pis2);
//     if prove_result.success {
//         // worker will panic here
//         assert_eq!(pis1, prove_result.reduced_pis.0[0]);
//         assert_eq!(pis2, prove_result.reduced_pis.0[1]);
//     }
//     Ok(())
// }


fn get_snark_js_ark_vk(vk: SnarkJSGroth16Vkey) -> AnyhowResult<VerifyingKey<Bn254>> {
    let alpha_g1 = G1Affine::new(
        Fq::from_str(
            vk.vk_alpha_1.get(0).unwrap(),
        )
        .unwrap(),
        Fq::from_str(
            vk.vk_alpha_1.get(1).unwrap(),
        )
        .unwrap(),
    );
    let beta_g2 = G2Affine::new(
        Fq2::new(
            Fq::from_str(
                vk.vk_beta_2.get(0).unwrap().get(0).unwrap(),
            )
            .unwrap(),
            Fq::from_str(
                vk.vk_beta_2.get(0).unwrap().get(1).unwrap(),
            )
            .unwrap(),
        ),
        Fq2::new(
            Fq::from_str(
                vk.vk_beta_2.get(1).unwrap().get(0).unwrap(),
            )
            .unwrap(),
            Fq::from_str(
                vk.vk_beta_2.get(1).unwrap().get(1).unwrap(),
            )
            .unwrap(),
        ),
    );
    let gamma_g2 = G2Affine::new(
        Fq2::new(
            Fq::from_str(
                vk.vk_gamma_2.get(0).unwrap().get(0).unwrap(),
            )
            .unwrap(),
            Fq::from_str(
                vk.vk_gamma_2.get(0).unwrap().get(1).unwrap(),
            )
            .unwrap(),
        ),
        Fq2::new(
            Fq::from_str(
                vk.vk_gamma_2.get(1).unwrap().get(0).unwrap(),
            )
            .unwrap(),
            Fq::from_str(
                vk.vk_gamma_2.get(1).unwrap().get(1).unwrap(),
            )
            .unwrap(),
        ),
    );
    let delta_g2 = G2Affine::new(
        Fq2::new(
            Fq::from_str(
                vk.vk_delta_2.get(0).unwrap().get(0).unwrap(),
            )
            .unwrap(),
            Fq::from_str(
                vk.vk_delta_2.get(0).unwrap().get(1).unwrap(),
            )
            .unwrap(),
        ),
        Fq2::new(
            Fq::from_str(
                vk.vk_delta_2.get(1).unwrap().get(0).unwrap(),
            )
            .unwrap(),
            Fq::from_str(
                vk.vk_delta_2.get(1).unwrap().get(1).unwrap(),
            )
            .unwrap(),
        ),
    );

    let mut gamma_abc_g1 = vec![];
    for ic in vk.IC {
        let g1 = G1Affine::new(
            Fq::from_str(
                ic.get(0).unwrap()
            ).unwrap(),
            Fq::from_str(
                ic.get(1).unwrap()
            )
            .unwrap(),
        );
        gamma_abc_g1.push(g1);
    }

    let ark_vk = VerifyingKey::<Bn254>{
        alpha_g1,
        beta_g2,
        gamma_g2,
        delta_g2,
        gamma_abc_g1
    };
    Ok(ark_vk)
}

fn get_snark_ark_proof(proof: SnarkJSGroth16Proof) -> AnyhowResult<ArkProof<Bn254>> {
    let a = G1Affine::new(
        Fq::from_str(
            proof.pi_a.get(0).unwrap()
        )
        .unwrap(),
        Fq::from_str(
            proof.pi_a.get(1).unwrap()
        )
        .unwrap(),
    );
    let b = G2Affine::new(
        Fq2::new(
            Fq::from_str(
                proof.pi_b.get(0).unwrap().get(0).unwrap()
            )
            .unwrap(),
            Fq::from_str(
                proof.pi_b.get(0).unwrap().get(1).unwrap()
            )
            .unwrap(),
        ),
        Fq2::new(
            Fq::from_str(
                proof.pi_b.get(1).unwrap().get(0).unwrap()
            )
            .unwrap(),
            Fq::from_str(
                proof.pi_b.get(1).unwrap().get(1).unwrap()
            )
            .unwrap(),
        ),
    );
    let c = G1Affine::new(
        Fq::from_str(
            proof.pi_c.get(0).unwrap()
        )
        .unwrap(),
        Fq::from_str(
            proof.pi_c.get(1).unwrap()
        )
        .unwrap(),
    );
    let ark_proof = ArkProof::<Bn254> { a, b, c };
    Ok(ark_proof)
}

fn get_snark_pis(pis: SnarkJSGroth16Pis) -> AnyhowResult<Vec<Fr>> {
    let mut ark_pis = vec![];
    for p in pis.0 {
        ark_pis.push(Fr::from_str(&p).unwrap())
    }
    Ok(ark_pis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    #[tokio::test]
    #[ignore]
    pub async fn test_proof_reduction_by_proof_hash() {
        // NOTE: it connect to database mentioned in the env file, to connect to the test db use .env.test file
        // dotenv::from_filename("../.env.test").ok();
        dotenv().ok();
        let proof_id = 234; // change the proof id
        let (result, reduction_time) = handle_proof_generation(proof_id).await.unwrap();
        println!("{:?}", result);
        assert_eq!(result.success, true);
    }
}

use anyhow::{anyhow, Ok, Result as AnyhowResult};
use ark_groth16::verifier;
use ark_serialize::CanonicalSerialize;
use quantum_db::repository::{
    proof_repository::update_reduction_data,
    user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
};
use quantum_types::{
    enums::proving_schemes::ProvingSchemes,
    traits::{
        pis::Pis,
        proof::Proof,
        vkey::Vkey,
    },
    types::{
        config::ConfigData, db::{proof::Proof as DBProof, user_circuit_data::UserCircuitData}, gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey}, gnark_plonk::{GnarkPlonkPis, GnarkPlonkSolidityProof, GnarkPlonkVkey}, halo2_plonk::{Halo2PlonkPis, Halo2PlonkProof, Halo2PlonkVkey}, halo2_poseidon::{Halo2PoseidonPis, Halo2PoseidonProof, Halo2PoseidonVkey}, plonk2::{Plonky2Proof, Plonky2Vkey}, riscs0::{Risc0Proof, Risc0Vkey}, snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Proof, SnarkJSGroth16Vkey}, sp1::{Sp1Proof, Sp1Vkey}, nitro_att::{NitroAttProof, NitroAttVkey}
    },
};
use quantum_utils::error_line;
use risc0_zkvm::{serde::to_vec, Receipt};
// use sp1_core::structs::SP1ReductionInput;
// use sp1_core::structs::SP1ReductionInput;
use tokio::time::Instant;
use tracing::info;
use quantum_db::repository::proof_repository::get_proof_by_proof_id;
use crate::{bonsai::{execute_proof_reduction_with_retry, upload_receipt}, connection::get_pool};
use crate::utils::dump_reduction_proof_data;
pub const SP1_CIRCUIT_VERSION: &str = "v3.0.0-rc1";

pub async fn handle_proof_generation_and_updation(
    proof_id: u64,
    proof_hash: &str,
    user_circuit_hash: &str,
    config: &ConfigData,
) -> AnyhowResult<()> {

    let (receipt, reduction_time) = handle_proof_generation(proof_id).await?;

    let receipt_path= dump_reduction_proof_data(
        config,
        user_circuit_hash,
        &proof_hash,
        receipt,
    )?;
    info!("Dumped reduced proof receipt");

    // update reduction data corresponding to proof
    update_reduction_data(
        get_pool().await,
        proof_id,
        &receipt_path,
        reduction_time,
    )
    .await?;
    info!("Updated reduction data to corresponding proof");
    Ok(())
}

async fn handle_proof_generation(proof_id: u64) ->AnyhowResult<(Receipt, u64)>{
    let proof_data = get_proof_by_proof_id(get_pool().await, proof_id).await?;
    let user_circuit_data = get_user_circuit_data_by_circuit_hash(get_pool().await, &proof_data.user_circuit_hash).await?;

    // Call proof generation to quantum_reduction_circuit
    let (receipt, reduction_time) = generate_reduced_proof(&user_circuit_data, &proof_data).await?;
    let receipt = receipt.unwrap();
    return Ok((receipt, reduction_time))
}

async fn generate_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof ) -> AnyhowResult<(Option<Receipt>, u64)> {

    let receipt: Option<Receipt>;
    let reduction_time: u64;

    if user_circuit_data.proving_scheme == ProvingSchemes::GnarkGroth16 {
        (receipt, reduction_time) = generate_gnark_groth16_reduced_proof(user_circuit_data, proof_data).await?;
    } else if user_circuit_data.proving_scheme == ProvingSchemes::Groth16 {
        (receipt, reduction_time) = generate_snarkjs_groth16_reduced_proof(user_circuit_data, proof_data).await?;
    } else if user_circuit_data.proving_scheme == ProvingSchemes::Halo2Plonk {
        (receipt, reduction_time) = generate_halo2_plonk_reduced_proof(user_circuit_data, proof_data).await?;
    } else if user_circuit_data.proving_scheme == ProvingSchemes::GnarkPlonk {
        (receipt, reduction_time) = generate_gnark_plonk_reduced_proof(user_circuit_data, proof_data).await?;
    } else if user_circuit_data.proving_scheme == ProvingSchemes::Halo2Poseidon {
        (receipt, reduction_time) = generate_halo2_poseidon_reduced_proof(user_circuit_data, proof_data).await?;
    } else if user_circuit_data.proving_scheme == ProvingSchemes::Plonky2 {
        (receipt, reduction_time) = generate_plonky2_reduced_proof(user_circuit_data, proof_data).await?;
    } else if user_circuit_data.proving_scheme == ProvingSchemes::Risc0 {
        (receipt, reduction_time) = generate_risc0_reduced_proof(user_circuit_data, proof_data).await?;
    } else if user_circuit_data.proving_scheme == ProvingSchemes::NitroAtt {
        (receipt, reduction_time) = generate_nitro_att_reduced_proof(user_circuit_data, proof_data).await?;
    }
    //else if user_circuit_data.proving_scheme == ProvingSchemes::Sp1 {
        // (receipt, reduction_time) = generate_sp1_reduced_proof(user_circuit_data, proof_data).await?;
    //}
    else {
        return Err(anyhow!(error_line!("unsupported proving scheme in proof reduction")));
    }

    // let reduction_time = reduction_start_time.elapsed().as_secs();
    info!("Reduced Proof successfully generated in {:?}", reduction_time);
    Ok((receipt, reduction_time))
}

fn form_snarkjs_groth16_bonsai_inputs(vk: SnarkJSGroth16Vkey, proof: SnarkJSGroth16Proof, pis: SnarkJSGroth16Pis) ->  AnyhowResult<Vec<u8>>{
    let ark_vk = vk.get_ark_vk_for_snarkjs_groth16()?;
    let pvk = verifier::prepare_verifying_key(&ark_vk);

    let ark_proof = proof.get_ark_proof_for_snarkjs_groth16_proof()?;
    let ark_public_inputs = pis.get_ark_pis_for_snarkjs_groth16_pis()?;


    let mut pvk_bytes = vec![];
    pvk.serialize_uncompressed(&mut pvk_bytes)?;

    let mut proof_bytes = vec![];
    ark_proof.serialize_uncompressed(&mut proof_bytes)?;

    let mut public_inputs_bytes = vec![];
    ark_public_inputs.serialize_uncompressed(&mut public_inputs_bytes)?;


    let input_data = to_vec(&pvk_bytes)?;
    let mut input_data_vec: Vec<u8> = bytemuck::cast_slice(&input_data).to_vec();

    let input_data = to_vec(&proof_bytes)?;
    input_data_vec.extend_from_slice( bytemuck::cast_slice(&input_data));


    let input_data = to_vec(&public_inputs_bytes)?;
    input_data_vec.extend_from_slice( bytemuck::cast_slice(&input_data));

    Ok(input_data_vec)
}

async fn  generate_snarkjs_groth16_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    let vk = SnarkJSGroth16Vkey::read_vk(&user_circuit_data.vk_path)?;
    let proof = SnarkJSGroth16Proof::read_proof(&proof_data.proof_path)?;
    let public_inputs = SnarkJSGroth16Pis::read_pis(&proof_data.pis_path)?;

    let input_data_vec = form_snarkjs_groth16_bonsai_inputs(vk, proof, public_inputs)?;

    let reduction_start_time = Instant::now();
    let assumptions = vec![];
    let (receipt, _) = execute_proof_reduction_with_retry(&input_data_vec, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap(), &assumptions).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();
    Ok((receipt,reduction_time))
}

fn form_halo2_plonk_bonsai_inputs(proof: &Halo2PlonkProof, vk: &Halo2PlonkVkey, pis: &Halo2PlonkPis) -> AnyhowResult<Vec<u8>> {
    let protocol = vk.get_protocol()?;
    let s_g2 = vk.get_sg2()?;
    let instances = pis.get_instance()?;
    let proof = &proof.proof_bytes;

    let protocol_bytes = to_vec(&protocol)?;
    let s_g2_bytes = to_vec(&s_g2)?;
    let instances_bytes = to_vec(&instances)?;
    let proof_bytes = to_vec(&proof)?;

    let mut input_data_vec: Vec<u8> = bytemuck::cast_slice(&protocol_bytes).to_vec();
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&s_g2_bytes));
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&instances_bytes));
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&proof_bytes));

    Ok(input_data_vec)
}

async fn generate_halo2_plonk_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    // Get inner_proof
    let proof_path = &proof_data.proof_path;
    println!("proof_path :: {:?}", proof_path);

    // Get inner_vk
    let vk_path = &user_circuit_data.vk_path;
    println!("vk_path :: {:?}", vk_path);

    // Get inner_pis
    let pis_path = &proof_data.pis_path;
    println!("pis_path :: {:?}", pis_path);

    let proof = Halo2PlonkProof::read_proof(&proof_path)?;
    let vk = Halo2PlonkVkey::read_vk(&vk_path)?;
    let pis = Halo2PlonkPis::read_pis(&pis_path)?;

    let input_data = form_halo2_plonk_bonsai_inputs(&proof, &vk, &pis)?;
    let assumptions = vec![];

    let reduction_start_time = Instant::now();
    let (receipt, _) = execute_proof_reduction_with_retry(&input_data, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap(), &assumptions).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();

    Ok((receipt, reduction_time))
}


fn form_halo2_poseidon_bonsai_inputs(proof: &Halo2PoseidonProof, vk: &Halo2PoseidonVkey, pis: &Halo2PoseidonPis) -> AnyhowResult<Vec<u8>> {
    let protocol = vk.get_protocol()?;
    let s_g2 = vk.get_sg2()?;
    let instances = pis.get_instance()?;
    let proof = &proof.proof_bytes;

    let protocol_bytes = to_vec(&protocol)?;
    let s_g2_bytes = to_vec(&s_g2)?;
    let instances_bytes = to_vec(&instances)?;
    let proof_bytes = to_vec(&proof)?;

    let mut input_data_vec: Vec<u8> = bytemuck::cast_slice(&protocol_bytes).to_vec();
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&s_g2_bytes));
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&instances_bytes));
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&proof_bytes));

    Ok(input_data_vec)
}

async fn generate_halo2_poseidon_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    // Get inner_proof
    let proof_path = &proof_data.proof_path;
    println!("proof_path :: {:?}", proof_path);

    // Get inner_vk
    let vk_path = &user_circuit_data.vk_path;
    println!("vk_path :: {:?}", vk_path);

    // Get inner_pis
    let pis_path = &proof_data.pis_path;
    println!("pis_path :: {:?}", pis_path);

    let proof = Halo2PoseidonProof::read_proof(&proof_path)?;
    let vk = Halo2PoseidonVkey::read_vk(&vk_path)?;
    let pis = Halo2PoseidonPis::read_pis(&pis_path)?;

    let input_data = form_halo2_poseidon_bonsai_inputs(&proof, &vk, &pis)?;
    let assumptions = vec![];

    let reduction_start_time = Instant::now();
    let (receipt, _) = execute_proof_reduction_with_retry(&input_data, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap(), &assumptions).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();

    Ok((receipt, reduction_time))
}


fn form_plonk2_bonsai_inputs(proof: &Plonky2Proof, vk: &Plonky2Vkey) -> AnyhowResult<Vec<u8>> {

    let common_bytes = to_vec(&vk.common_bytes)?;
    let verifier_only_bytes = to_vec(&vk.verifier_only_bytes)?;
    let proof_bytes = to_vec(&proof.proof_bytes)?;

    let mut input_data_vec: Vec<u8> = bytemuck::cast_slice(&common_bytes).to_vec();
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&verifier_only_bytes));
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&proof_bytes));

    Ok(input_data_vec)
}

async fn generate_plonky2_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    // Get inner_proof
    let proof_path = &proof_data.proof_path;
    println!("proof_path :: {:?}", proof_path);

    // Get inner_vk
    let vk_path = &user_circuit_data.vk_path;
    println!("vk_path :: {:?}", vk_path);

    let proof = Plonky2Proof::read_proof(&proof_path)?;
    let vk = Plonky2Vkey::read_vk(&vk_path)?;

    let input_data = form_plonk2_bonsai_inputs(&proof, &vk)?;
    let assumptions = vec![];

    let reduction_start_time = Instant::now();
    let (receipt, _) = execute_proof_reduction_with_retry(&input_data, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap(), &assumptions).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();

    Ok((receipt, reduction_time))
}

fn form_risc0_bonsai_inputs(proof: &Risc0Proof, vk: &Risc0Vkey) -> AnyhowResult<Vec<u8>> {

    // TODO: to check whether this to_vec is needed, vkey is already u32 type
    let image_id = to_vec(&vk.vkey_bytes)?;
    let pis_bytes = to_vec(&proof.get_receipt()?.journal.bytes)?;

    let mut input_data_vec: Vec<u8> = bytemuck::cast_slice(&image_id).to_vec();
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&pis_bytes));

    Ok(input_data_vec)
}

// TODO: add assumption also
async fn generate_risc0_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    // Get inner_proof
    let proof_path = &proof_data.proof_path;
    println!("proof_path :: {:?}", proof_path);

    // Get inner_vk
    let vk_path = &user_circuit_data.vk_path;
    println!("vk_path :: {:?}", vk_path);

    let proof = Risc0Proof::read_proof(&proof_path)?;
    let vk = Risc0Vkey::read_vk(&vk_path)?;

    let input_data = form_risc0_bonsai_inputs(&proof, &vk)?;

    let receipt_id = upload_receipt(proof.get_receipt()?).await?;
    println!("uploaded recepit_id: {:?}", receipt_id);
    let assumptions = vec![receipt_id];
    let reduction_start_time = Instant::now();
    let (receipt, _) = execute_proof_reduction_with_retry(&input_data, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap(), &assumptions).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();

    Ok((receipt, reduction_time))
}

fn form_nitro_att_bonsai_inputs(proof: &NitroAttProof, vk: &NitroAttVkey) -> AnyhowResult<Vec<u8>> {
    Ok(proof.att_doc_bytes.clone())
}

async fn generate_nitro_att_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    // Get inner_proof
    let proof_path = &proof_data.proof_path;
    println!("proof_path :: {:?}", proof_path);

    // Get inner_vk
    let vk_path = &user_circuit_data.vk_path;
    println!("vk_path :: {:?}", vk_path);

    let proof = NitroAttProof::read_proof(&proof_path)?;
    let vk = NitroAttVkey::read_vk(&vk_path)?;

    let input_data = form_nitro_att_bonsai_inputs(&proof, &vk)?;

    let assumptions = vec![];
    let reduction_start_time = Instant::now();
    let (receipt, _) = execute_proof_reduction_with_retry(&input_data, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap(), &assumptions).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();

    Ok((receipt, reduction_time))
}

// fn form_sp1_bonsai_inputs(proof: &Sp1Proof, vk: &Sp1Vkey) -> AnyhowResult<Vec<u8>> {

//     // TODO: to check whether this to_vec is needed, vkey is already u32 type
//     // let image_id = to_vec(&vk.vkey_bytes)?;
//     // let pis_bytes = to_vec(&proof.receipt.journal.bytes)?;

//     let sp1_reduction_input = SP1ReductionInput {
//             vk: vk.get_verifying_key()?.vk.clone(),
//             //TODO: remove unwrap
//             compressed_proof: proof.get_proof_with_public_inputs()?.proof.clone().try_as_compressed().unwrap().deref().clone(),
//             public_values: proof.get_proof_with_public_inputs()?.public_values.clone(),
//             sp1_version: proof.get_proof_with_public_inputs()?.sp1_version.clone(),
//         };

//     let sp1_reduction_input_bytes = to_vec(&sp1_reduction_input)?;
//     let input_data_vec: Vec<u8> = bytemuck::cast_slice(&sp1_reduction_input_bytes).to_vec();
//     Ok(input_data_vec)
// }


// async fn generate_sp1_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
//     // Get inner_proof
//     let proof_path = &proof_data.proof_path;
//     println!("proof_path :: {:?}", proof_path);

//     // Get inner_vk
//     let vk_path = &user_circuit_data.vk_path;
//     println!("vk_path :: {:?}", vk_path);

//     let proof = Sp1Proof::read_proof(&proof_path)?;
//     let vk = Sp1Vkey::read_vk(&vk_path)?;

//     let input_data = form_sp1_bonsai_inputs(&proof, &vk)?;
//     let assumptions = vec![];

//     let reduction_start_time = Instant::now();
//     let (receipt, _) = execute_proof_reduction(input_data, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap(), assumptions).await?;
//     let reduction_time = reduction_start_time.elapsed().as_secs();

//     Ok((receipt, reduction_time))
// }

fn form_gnark_plonk_bonsai_inputs(proof: &GnarkPlonkSolidityProof, vk: &GnarkPlonkVkey, pis: &GnarkPlonkPis)-> AnyhowResult<Vec<u8>> {
    let proof_bytes = to_vec(&proof.proof_bytes)?;
    let vk_bytes = to_vec(&vk.vkey_bytes)?;

    let ark_public_inputs = pis.get_ark_pis_for_gnark_plonk_pis()?;
    let mut public_inputs_bytes = vec![];
    ark_public_inputs.serialize_uncompressed(&mut public_inputs_bytes)?;
    let public_inputs_bytes = to_vec(&public_inputs_bytes)?;

    let mut input_data_vec: Vec<u8> = bytemuck::cast_slice(&vk_bytes).to_vec();
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&proof_bytes));
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&public_inputs_bytes));

    Ok(input_data_vec)
}

async fn generate_gnark_plonk_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    // Get inner_proof
    let proof_path = &proof_data.proof_path;
    println!("proof_path :: {:?}", proof_path);

    // Get inner_vk
    let vk_path = &user_circuit_data.vk_path;
    println!("vk_path :: {:?}", vk_path);

    // Get inner_pis
    let pis_path = &proof_data.pis_path;
    println!("pis_path :: {:?}", pis_path);
    // 1.Reconstruct inner proof
    let proof = GnarkPlonkSolidityProof::read_proof(&proof_path)?;
    let vk = GnarkPlonkVkey::read_vk(&vk_path)?;
    let pis = GnarkPlonkPis::read_pis(&pis_path)?;

    let input_data = form_gnark_plonk_bonsai_inputs(&proof, &vk, &pis)?;
    let assumptions = vec![];

    let reduction_start_time = Instant::now();
    let (receipt, _) = execute_proof_reduction_with_retry(&input_data, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap(), &assumptions).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();

    Ok((receipt, reduction_time))
}

fn form_gnark_groth16_bonsai_inputs(proof: &GnarkGroth16Proof, vk: &GnarkGroth16Vkey, pis: &GnarkGroth16Pis)-> AnyhowResult<Vec<u8>> {
    let proof_bytes = to_vec(&proof.proof_bytes)?;
    let vk_bytes = to_vec(&vk.vkey_bytes)?;

    let ark_public_inputs = pis.get_ark_pis_for_gnark_groth16_pis()?;
    let mut public_inputs_bytes = vec![];
    ark_public_inputs.serialize_uncompressed(&mut public_inputs_bytes)?;
    let public_inputs_bytes = to_vec(&public_inputs_bytes)?;

    let mut input_data_vec: Vec<u8> = bytemuck::cast_slice(&vk_bytes).to_vec();
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&proof_bytes));
    input_data_vec.extend_from_slice(bytemuck::cast_slice(&public_inputs_bytes));

    Ok(input_data_vec)
}

async fn generate_gnark_groth16_reduced_proof(user_circuit_data: &UserCircuitData, proof_data: &DBProof) -> AnyhowResult<(Option<Receipt>, u64)> {
    // Get inner_proof
    let proof_path = &proof_data.proof_path;
    println!("proof_path :: {:?}", proof_path);

    // Get inner_vk
    let vk_path = &user_circuit_data.vk_path;
    println!("vk_path :: {:?}", vk_path);

    // Get inner_pis
    let pis_path = &proof_data.pis_path;
    println!("pis_path :: {:?}", pis_path);
    // 1.Reconstruct inner proof
    let proof = GnarkGroth16Proof::read_proof(&proof_path)?;
    let vk = GnarkGroth16Vkey::read_vk(&vk_path)?;
    let pis = GnarkGroth16Pis::read_pis(&pis_path)?;

    let input_data = form_gnark_groth16_bonsai_inputs(&proof, &vk, &pis)?;
    let assumptions = vec![];

    let reduction_start_time = Instant::now();
    let (receipt, _) = execute_proof_reduction_with_retry(&input_data, &user_circuit_data.bonsai_image_id, proof_data.id.unwrap(), &assumptions).await?;
    let reduction_time = reduction_start_time.elapsed().as_secs();

    Ok((receipt, reduction_time))
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
        let proof_id = 2; // change the proof id
        let (result, reduction_time) = handle_proof_generation(proof_id).await.unwrap();
        println!("{:?}", result);
        // assert_eq!(result.success, true);
    }
}

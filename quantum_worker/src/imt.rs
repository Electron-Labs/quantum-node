// use std::time::{Duration, Instant};

// use anyhow::{anyhow, Ok, Result as AnyhowResult};
// use quantum_circuits_interface::amqp::interactor::QuantumV2CircuitInteractor;
// use quantum_db::repository::{
//     reduction_circuit_repository::get_reduction_circuit_for_user_circuit,
//     superproof_repository::{
//         update_imt_pis_path, update_imt_proof_path, update_previous_superproof_root,
//         update_superproof_leaves_path, update_superproof_root,
//     },
//     user_circuit_data_repository::get_user_circuit_data_by_circuit_hash,
// };
// use quantum_types::{
//     enums::proving_schemes::ProvingSchemes,
//     traits::{circuit_interactor::CircuitInteractorAMQP, pis::Pis, proof::Proof, vkey::Vkey},
//     types::{
//         config::{AMQPConfigData, ConfigData},
//         db::proof::Proof as DBProof,
//         gnark_groth16::{GnarkGroth16Pis, GnarkGroth16Proof, GnarkGroth16Vkey},
//         halo2_plonk::{Halo2PlonkPis, Halo2PlonkVkey},
//         imt::ImtTree,
//         snarkjs_groth16::{SnarkJSGroth16Pis, SnarkJSGroth16Vkey},
//     },
// };
// use quantum_utils::{error_line, keccak::encode_keccak_hash, paths::get_superproof_leaves_path};
// use sqlx::{MySql, Pool};
// use tracing::info;
// use quantum_types::traits::circuit_interactor::GenerateImtProofResult;
// use crate::connection::get_pool;
// use crate::utils::{dump_imt_proof_data, get_last_superproof_leaves};

// pub const IMT_DEPTH: usize = 10;

// pub async fn handle_imt_proof_generation_and_updation(
//     proofs: Vec<DBProof>,
//     superproof_id: u64,
//     config: &ConfigData,
// ) -> AnyhowResult<()> {

//     let (imt_prove_result, time) = handle_imt_proof_generation(proofs, superproof_id, config).await?;
//     // Dump superproof_leaves and add to the DB
//     let superproof_leaves = ImtTree {
//         leaves: imt_prove_result.new_leaves,
//     };
//     let superproof_leaves_path = get_superproof_leaves_path(
//         &config.storage_folder_path,
//         &config.supperproof_path,
//         superproof_id,
//     );
//     superproof_leaves.dump_tree(&superproof_leaves_path)?;
//     update_superproof_leaves_path(get_pool().await, &superproof_leaves_path, superproof_id).await?;

//     // Dump imt proof and pis and add to the DB
//     // let (imt_proof_path, imt_pis_path) = dump_imt_proof_data(
//     //     &config,
//     //     superproof_id,
//     //     imt_prove_result.aggregated_proof,
//     //     GnarkGroth16Pis(imt_prove_result.pub_inputs),
//     // )?;
//     // update_imt_proof_path(get_pool().await, &imt_proof_path, superproof_id).await?;
//     // update_imt_pis_path(get_pool().await, &imt_pis_path, superproof_id).await?;

//     // Add previous superproof root to the db
//     let old_root = encode_keccak_hash(&imt_prove_result.old_root.0)?;
//     update_previous_superproof_root(get_pool().await, &old_root, superproof_id).await?;

//     // Add superproof root to the db
//     let new_root = encode_keccak_hash(&imt_prove_result.new_root.0)?;
//     update_superproof_root(get_pool().await, &new_root, superproof_id).await?;

//     Ok(())
// }

// async fn handle_imt_proof_generation(
//     proofs: Vec<DBProof>,
//     superproof_id: u64,
//     config: &ConfigData,
// ) -> AnyhowResult<(GenerateImtProofResult, Duration)> {

//     let amqp_config = AMQPConfigData::get_config();
//     let mut reduced_proofs = Vec::<GnarkGroth16Proof>::new();
//     let mut reduced_pis_vec = Vec::<GnarkGroth16Pis>::new();
//     let mut reduced_circuit_vkeys = Vec::<GnarkGroth16Vkey>::new();

//     for proof in &proofs {
//         let reduced_proof_path = proof.reduction_proof_path.clone().unwrap();
//         let reduced_proof = GnarkGroth16Proof::read_proof(&reduced_proof_path)?;
//         reduced_proofs.push(reduced_proof);
//         let reduced_pis_path = proof.reduction_proof_pis_path.clone().unwrap();
//         let reduced_pis = GnarkGroth16Pis::read_pis(&reduced_pis_path)?;
//         reduced_pis_vec.push(reduced_pis);
//         let reduced_circuit_vkey_path =
//             get_reduction_circuit_for_user_circuit(get_pool().await, &proof.user_circuit_hash)
//                 .await?
//                 .vk_path;
//         let reduced_vkey = GnarkGroth16Vkey::read_vk(&reduced_circuit_vkey_path)?;
//         reduced_circuit_vkeys.push(reduced_vkey);
//     }
//     info!("superproof_id {:?}", superproof_id);
//     let last_leaves = get_last_superproof_leaves(config).await?;

//     let mut protocol_vkey_hashes: Vec<Vec<u8>> = vec![];
//     let mut protocol_pis_hashes: Vec<Vec<u8>> = vec![];
//     let mut reduced_vkey_hashes: Vec<Vec<u8>> = vec![];

//     for proof in &proofs {
//         let reduced_circuit_vkey_path =
//             get_reduction_circuit_for_user_circuit(get_pool().await, &proof.user_circuit_hash)
//                 .await?
//                 .vk_path;
//         let reduced_vkey = GnarkGroth16Vkey::read_vk(&reduced_circuit_vkey_path)?;
//         reduced_vkey_hashes.push(reduced_vkey.keccak_hash()?.to_vec());

//         let user_circuit_data =
//             get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash).await?;
//         let protocol_circuit_vkey_path =
//             get_user_circuit_data_by_circuit_hash(get_pool().await, &proof.user_circuit_hash)
//                 .await?
//                 .vk_path;
//         let protocol_pis_path = proof.pis_path.clone();

//         match user_circuit_data.proving_scheme {
//             ProvingSchemes::GnarkGroth16 => {
//                 let protocol_vkey = GnarkGroth16Vkey::read_vk(&protocol_circuit_vkey_path)?;
//                 protocol_vkey_hashes.push(protocol_vkey.extended_keccak_hash(user_circuit_data.n_commitments)?.to_vec());

//                 let protocol_pis = GnarkGroth16Pis::read_pis(&protocol_pis_path)?;
//                 protocol_pis_hashes.push(protocol_pis.extended_keccak_hash()?.to_vec());
//             }
//             ProvingSchemes::Groth16 => {
//                 let protocol_vkey = SnarkJSGroth16Vkey::read_vk(&protocol_circuit_vkey_path)?;
//                 protocol_vkey_hashes.push(protocol_vkey.extended_keccak_hash(user_circuit_data.n_commitments)?.to_vec());

//                 let protocol_pis = SnarkJSGroth16Pis::read_pis(&protocol_pis_path)?;
//                 protocol_pis_hashes.push(protocol_pis.extended_keccak_hash()?.to_vec());
//             }
//             ProvingSchemes::Halo2Plonk => {
//                 let protocol_vkey = Halo2PlonkVkey::read_vk(&protocol_circuit_vkey_path)?;
//                 protocol_vkey_hashes.push(protocol_vkey.extended_keccak_hash(user_circuit_data.n_commitments)?.to_vec());

//                 let protocol_pis = Halo2PlonkPis::read_pis(&protocol_pis_path)?;
//                 protocol_pis_hashes.push(protocol_pis.extended_keccak_hash()?.to_vec());
//             }
//             _ => todo!(),
//         }
//     }

//     let imt_start = Instant::now();

//     let imt_prove_result = QuantumV2CircuitInteractor::generate_imt_proof(
//         &amqp_config,
//         config.batch_size,
//         last_leaves.leaves,
//         reduced_vkey_hashes,
//         protocol_vkey_hashes,
//         protocol_pis_hashes,
//         superproof_id,
//     )?;
//     let imt_proving_time = imt_start.elapsed();
//     info!("imt_prove_result {:?}", imt_prove_result.success);
//     info!("imt_prove_result success {:?}", imt_prove_result.success);
//     info!("imt_proving_time {:?}", imt_proving_time);

//     if !imt_prove_result.success {
//         return Err(anyhow::Error::msg(imt_prove_result.msg));
//     }
//     Ok((imt_prove_result, imt_proving_time))
// }

// #[cfg(test)]
// mod tests {
//     use quantum_circuits_interface::imt::get_init_tree_data;
//     use quantum_db::repository::proof_repository::get_proofs_in_superproof_id;
//     use quantum_db::repository::superproof_repository::get_superproof_by_id;
//     use quantum_types::types::config::ConfigData;
//     use quantum_utils::keccak::{decode_keccak_hex, encode_keccak_hash};
//     use crate::connection::get_pool;
//     use crate::imt::{handle_imt_proof_generation, IMT_DEPTH};

//     #[test]
//     pub fn yo() {
//         let (x, y) = get_init_tree_data(IMT_DEPTH as u8).unwrap();
//         let h = encode_keccak_hash(&y.0).unwrap();
//         let a = decode_keccak_hex(&h).unwrap();
//         assert_eq!(y.0, a);
//         println!("h {:?}", h);
//     }

//     #[tokio::test]
//     #[ignore]
//     pub async fn test_imt_proof_by_superproof_id() {
//         // NOTE: it connect to database mentioned in the env file, to connect to the test db use .env.test file
//         // dotenv::from_filename("../.env.test").ok();
//         // dotenv().ok();
//         let config_data = ConfigData::new("../../config.yaml"); // change the path
//         let superproof_id = 90; // insert your circuit hash
//         let superproof = get_superproof_by_id(get_pool().await, superproof_id).await.unwrap();
//         let proofs = get_proofs_in_superproof_id(get_pool().await,superproof_id).await.unwrap();
//         let (result, reduction_time) = handle_imt_proof_generation(proofs, superproof_id, &config_data).await.unwrap();
//         println!("{:?}", result);
//         assert_eq!(result.success, true);
//     }

// }

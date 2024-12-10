use constant::{GNARK_GROTH16_PROOF_ID, GNARK_GROTH16_TASK_ID, GNARK_PLONK_PROOF_ID, GNARK_PLONK_TASK_ID, GROTH16_PROOF_ID, GROTH16_TASK_ID, HALO2_PLONK_PROOF_ID, HALO2_PLONK_TASK_ID, HALO2_POSEIDON_PROOF_ID, HALO2_POSEIDON_TASK_ID, PLONKY2_PROOF_ID, PLONKY2_TASK_ID, RISC0_PROOF_ID, RISC0_TASK_ID};
use quantum_db::repository::{cost_saved_repository::udpate_cost_saved_data, proof_repository::update_proof_status, task_repository::update_task_status};
use quantum_types::{enums::{proof_status::ProofStatus, task_status::TaskStatus}, types::config::ConfigData};
use quantum_worker::{connection::get_pool, worker::handle_proof_generation_task};
use repository::get_task_by_task_id;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

mod constant;
mod repository;
async fn before_test(proof_id: u64, task_id: u64) -> ConfigData{

    dotenv::from_filename(".env.test").ok();

    let stdout_log = tracing_subscriber::fmt::layer().compact();
    tracing_subscriber::registry().with(filter::LevelFilter::INFO).with(stdout_log).init();
    
    let config_data = ConfigData::new("../test_config.yaml");

    update_proof_status(get_pool().await, proof_id, ProofStatus::Registered).await.unwrap();
    update_task_status(get_pool().await, task_id, TaskStatus::NotPicked).await.unwrap();
    // udpate_cost_saved_data(get_pool().await, 234.5674365 as u64, 74685.878 as f64).await.unwrap();
    config_data
}


//done
#[tokio::test]
async fn test_groth16_proof_generation(){
    let config_data = before_test(GROTH16_PROOF_ID, GROTH16_TASK_ID).await;

    let proof_task = get_task_by_task_id(get_pool().await, GROTH16_TASK_ID).await.unwrap();

    handle_proof_generation_task(proof_task, &config_data).await.unwrap();

    let proof_task = get_task_by_task_id(get_pool().await, GROTH16_TASK_ID).await.unwrap();
    assert!(proof_task.task_status == TaskStatus::Completed);
}

#[tokio::test]
async fn test_gnark_groth16_proof_generation() {
    let config_data = before_test(GNARK_GROTH16_PROOF_ID, GNARK_GROTH16_TASK_ID).await;

    let proof_task = get_task_by_task_id(get_pool().await, GNARK_GROTH16_TASK_ID).await.unwrap();

    handle_proof_generation_task(proof_task, &config_data).await.unwrap();

    let proof_task = get_task_by_task_id(get_pool().await, GNARK_GROTH16_TASK_ID).await.unwrap();
    assert!(proof_task.task_status == TaskStatus::Completed);
}


//done
#[tokio::test]
async fn test_plonky2_proof_generation(){
    let config_data = before_test(PLONKY2_PROOF_ID, PLONKY2_TASK_ID).await;

    let proof_task = get_task_by_task_id(get_pool().await, PLONKY2_TASK_ID).await.unwrap();

    handle_proof_generation_task(proof_task, &config_data).await.unwrap();

    let proof_task = get_task_by_task_id(get_pool().await, PLONKY2_TASK_ID).await.unwrap();
    assert!(proof_task.task_status == TaskStatus::Completed);
}

//done
#[tokio::test]
async fn test_gnark_plonk_proof_generation(){
    let config_data = before_test(GNARK_PLONK_PROOF_ID, GNARK_PLONK_TASK_ID).await;

    let proof_task = get_task_by_task_id(get_pool().await, GNARK_PLONK_TASK_ID).await.unwrap();

    handle_proof_generation_task(proof_task, &config_data).await.unwrap();

    let proof_task = get_task_by_task_id(get_pool().await, GNARK_PLONK_TASK_ID).await.unwrap();
    assert!(proof_task.task_status == TaskStatus::Completed);
}


//done
#[tokio::test]
async fn test_risc0_proof_generation(){
    let config_data = before_test(RISC0_PROOF_ID, RISC0_TASK_ID).await;

    let proof_task = get_task_by_task_id(get_pool().await, RISC0_TASK_ID).await.unwrap();

    handle_proof_generation_task(proof_task, &config_data).await.unwrap();

    let proof_task = get_task_by_task_id(get_pool().await, RISC0_TASK_ID).await.unwrap();
    assert!(proof_task.task_status == TaskStatus::Completed);
}


//done
#[tokio::test]
async fn test_halo2_plonk_proof_generation(){
    let config_data = before_test(HALO2_PLONK_PROOF_ID, HALO2_PLONK_TASK_ID).await;

    let proof_task = get_task_by_task_id(get_pool().await, HALO2_PLONK_TASK_ID).await.unwrap();

    handle_proof_generation_task(proof_task, &config_data).await.unwrap();

    let proof_task = get_task_by_task_id(get_pool().await, HALO2_PLONK_TASK_ID).await.unwrap();
    assert!(proof_task.task_status == TaskStatus::Completed);
}

//done
#[tokio::test]
async fn test_halo2_poseidon_proof_generation(){
    let config_data = before_test(HALO2_POSEIDON_PROOF_ID, HALO2_POSEIDON_TASK_ID).await;

    let proof_task = get_task_by_task_id(get_pool().await, HALO2_POSEIDON_TASK_ID).await.unwrap();

    handle_proof_generation_task(proof_task, &config_data).await.unwrap();

    let proof_task = get_task_by_task_id(get_pool().await, HALO2_POSEIDON_TASK_ID).await.unwrap();
    assert!(proof_task.task_status == TaskStatus::Completed);
}
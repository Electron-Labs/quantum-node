mod constant;
use constant::AGGREGATION_PROOF_IDS;
use quantum_db::repository::proof_repository::{get_reduced_proofs_r0, update_proof_status};
use quantum_types::{enums::proof_status::ProofStatus, types::config::ConfigData};
use quantum_worker::{connection::get_pool, worker::aggregate_and_generate_new_superproof};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

async fn before_test(proof_ids: &[u64] ) -> ConfigData {

    dotenv::from_filename(".env.test").ok();

    let stdout_log = tracing_subscriber::fmt::layer().compact();
    tracing_subscriber::registry().with(filter::LevelFilter::INFO).with(stdout_log).init();

    let config_data = ConfigData::new("../test_config.yaml");

    for proof_id in proof_ids {
        update_proof_status(get_pool().await, proof_id.clone(), ProofStatus::Reduced).await.unwrap();
    }
    config_data
}


#[tokio::test]
async fn test_proof_aggregation_generation(){
    let config_data = before_test(&AGGREGATION_PROOF_IDS).await;

    let aggregation_awaiting_proofs_r0 = get_reduced_proofs_r0(get_pool().await).await.unwrap();

    aggregate_and_generate_new_superproof(aggregation_awaiting_proofs_r0, &config_data).await.unwrap();

}
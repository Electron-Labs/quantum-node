use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Superproof {
    pub id: Option<u64>,
    pub proof_ids: Option<String>,
    pub superproof_proof_path: Option<String>,
    pub superproof_pis_path: Option<String>,
    pub transaction_hash: Option<String>,
    pub gas_cost: Option<f64>,
    pub agg_time: Option<u64>
}
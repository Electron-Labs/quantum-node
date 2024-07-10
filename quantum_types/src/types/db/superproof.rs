use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::enums::superproof_status::SuperproofStatus;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Superproof {
    pub id: Option<u64>,
    pub proof_ids: Option<String>,
    pub superproof_proof_path: Option<String>,
    pub transaction_hash: Option<String>,
    pub gas_cost: Option<Decimal>,
    pub agg_time: Option<u64>,
    pub status: SuperproofStatus,
    pub superproof_root: Option<String>,
    pub superproof_leaves_path: Option<String>,
    pub onchain_submission_time: Option<NaiveDateTime>,
    pub eth_price: Option<Decimal>,
    pub imt_proof_path: Option<String>,
    pub imt_pis_path: Option<String>,
}
use serde::{Deserialize, Serialize};
use crate::enums::proving_schemes::ProvingSchemes;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BonsaiImage {
    pub image_id: String,
    pub elf_file_path: String,
    pub circuit_verifying_id: String,
    pub proving_scheme: ProvingSchemes,
    pub is_aggregation_image_id: u8
}
use serde::{Deserialize, Serialize};
use crate::enums::proving_schemes::ProvingSchemes;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BonsaiImage {
    pub image_id: String,
    pub elf_file_path: String,
    pub circuit_verifying_id: [u32;8],
    pub proving_scheme: Option<ProvingSchemes>,
    pub is_aggregation_image_id: u8
}
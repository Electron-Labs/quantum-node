use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ProvingSchemes {
    GnarkGroth16,
    Groth16,
    Plonky2,
    Halo2KZG
}

impl FromStr for ProvingSchemes {
    type Err = String;
    /*
        GnarkGroth16,
        Groth16,
        Plonky2,
        Halo2KZG
     */
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gnarkgroth16" => Ok(ProvingSchemes::GnarkGroth16),
            "groth16" => Ok(ProvingSchemes::Groth16),
            "plonky2" => Ok(ProvingSchemes::Plonky2),
            "halo2kzg" => Ok(ProvingSchemes::Halo2KZG),
            _ => Err(format!("Invalid proving scheme: {}", s)),
        }
    }
}
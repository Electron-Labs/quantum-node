use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, )]
pub enum ProvingSchemes {
    GnarkGroth16,
    Groth16,
    Plonky2,
    Halo2Plonk
}

impl FromStr for ProvingSchemes {
    type Err = String;
    /*
        GnarkGroth16,
        Groth16,
        Plonky2,
        Halo2Plonk
     */
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "gnarkgroth16" => Ok(ProvingSchemes::GnarkGroth16),
            "groth16" => Ok(ProvingSchemes::Groth16),
            "plonky2" => Ok(ProvingSchemes::Plonky2),
            "halo2plonk" => Ok(ProvingSchemes::Halo2Plonk),
            _ => Err(format!("Invalid proving scheme: {}", s)),
        }
    }
}

impl ToString for ProvingSchemes {
    fn to_string(&self) -> String {
        match self {
            ProvingSchemes::GnarkGroth16 => String::from("GnarkGroth16"),
            ProvingSchemes::Groth16 => String::from("Groth16"),
            ProvingSchemes::Halo2Plonk => String::from("Halo2Plonk"),
            ProvingSchemes::Plonky2 => String::from("Plonky2")
        }
    }
}
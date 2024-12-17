use crate::traits::{pis::Pis, proof::Proof, vkey::Vkey};
use aggregation::inputs::compute_combined_vkey_hash;
use anyhow::anyhow;
use anyhow::Result as AnyhowResult;

use borsh::{BorshDeserialize, BorshSerialize};

use halo2_kzg_evm_verifier::halo2_kzg_vkey_hash;
use halo2_kzg_evm_verifier::halo2_public_inputs_hash;
use num_bigint::BigUint;
use quantum_utils::error_line;
use quantum_utils::file::read_bytes_from_file;
use quantum_utils::file::write_bytes_to_file;

use serde::{Deserialize, Serialize};
use snark_verifier::halo2_base::halo2_proofs::halo2curves::bn256::Fr;
use snark_verifier::halo2_base::halo2_proofs::halo2curves::bn256::G1Affine;
use snark_verifier::halo2_base::halo2_proofs::halo2curves::bn256::G2Affine;
use snark_verifier::halo2_base::utils::ScalarField;
use snark_verifier::verifier::plonk::PlonkProtocol;
use utils::hash::Keccak256Hasher;
// use utils::halo2_kzg_vkey_hash;
// use utils::halo2_public_inputs_hash;

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Halo2PoseidonVkey {
    pub protocol_bytes: Vec<u8>,
    pub sg2_bytes: Vec<u8>,
}

impl Vkey for Halo2PoseidonVkey {
    fn serialize_vkey(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(buffer)
    }

    fn deserialize_vkey(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Halo2PoseidonVkey =
            BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }

    fn dump_vk(&self, path: &str) -> AnyhowResult<()> {
        let vkey_bytes = self.serialize_vkey()?;
        write_bytes_to_file(&vkey_bytes, path)?;
        Ok(())
    }

    fn read_vk(full_path: &str) -> AnyhowResult<Self> {
        let vkey_bytes = read_bytes_from_file(full_path)?;
        let vkey = Halo2PoseidonVkey::deserialize_vkey(&mut vkey_bytes.as_slice())?;
        Ok(vkey)
    }

    fn validate(&self) -> AnyhowResult<()> {
        let _ = self.get_sg2()?;
        let _ = self.get_protocol()?;
        Ok(())
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let protocol = self.get_protocol()?;
        let sg2 = self.get_sg2()?;
        let protocol_hash = halo2_kzg_vkey_hash::<Keccak256Hasher>(&protocol, &sg2);
        Ok(protocol_hash)
    }

    fn compute_circuit_hash(&self, circuit_verifying_id: [u32; 8]) -> AnyhowResult<[u8; 32]> {
        let protocol_hash = self.keccak_hash()?;
        let circuit_hash = compute_combined_vkey_hash::<Keccak256Hasher>(&protocol_hash, &circuit_verifying_id)?;
        Ok(circuit_hash)
    }
}

impl Halo2PoseidonVkey {
    pub fn get_protocol(&self) -> AnyhowResult<PlonkProtocol<G1Affine>> {
        let protocol: PlonkProtocol<G1Affine> = serde_json::from_slice(&self.protocol_bytes)?;
        Ok(protocol)
    }

    pub fn get_sg2(&self) -> AnyhowResult<G2Affine> {
        let s_g2: G2Affine = serde_json::from_slice(&self.sg2_bytes)?;
        Ok(s_g2)
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Halo2PoseidonProof {
    // TODO: change it to protocol_bytes
    pub proof_bytes: Vec<u8>,
}

impl Proof for Halo2PoseidonProof {
    fn serialize_proof(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    fn deserialize_proof(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Halo2PoseidonProof =
            BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }

    fn dump_proof(&self, path: &str) -> AnyhowResult<()> {
        let proof_bytes = self.serialize_proof()?;
        write_bytes_to_file(&proof_bytes, path)?;
        Ok(())
    }

    fn read_proof(full_path: &str) -> AnyhowResult<Self> {
        let proof_bytes = read_bytes_from_file(full_path)?;
        let gnark_proof = Halo2PoseidonProof::deserialize_proof(&mut proof_bytes.as_slice())?;
        Ok(gnark_proof)
    }

    fn validate_proof(&self, _vkey_path: &str,mut _pis_bytes: &[u8]) -> AnyhowResult<()> {
        // let vkey = Halo2PoseidonVkey::read_vk(vkey_path)?;
        // let pis = Halo2PoseidonPis::deserialize_pis(&mut pis_bytes)?;

        // let s_g2 = vkey.get_sg2()?;
        // let protocol = vkey.get_protocol()?;
        // let instances = pis.get_instance()?;

        // let dk = (G1Affine::generator(), G2Affine::generator(), s_g2).into();
        // let loader = NativeLoader;
        // let protocol = protocol.loaded(&loader);
        // let mut transcript = EvmTranscript::<_, NativeLoader, _, _>::new(self.proof_bytes.as_slice());

        // let proof_ = PlonkVerifier::<SHPLONK>::read_proof(&dk, &protocol, &instances, &mut transcript).map_err(|e| {anyhow!(error_line!(format!("error in halo2-plonk proof validation {:?}", e)))})?;
        // PlonkVerifier::<SHPLONK>::verify(&dk, &protocol, &instances, &proof_).map_err(|e| {anyhow!(error_line!(format!("Halo2Plonk proof validation failed: {:?}", e)))})?;
        Ok(())
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug, PartialEq)]
pub struct Halo2PoseidonPis(pub Vec<u8>);

impl Pis for Halo2PoseidonPis {
    fn serialize_pis(&self) -> AnyhowResult<Vec<u8>> {
        Ok(self.0.clone())
    }

    fn deserialize_pis(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let key: Halo2PoseidonPis =
            BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(key)
    }

    fn dump_pis(&self, path: &str) -> AnyhowResult<()> {
        let pis_bytes = self.serialize_pis()?;
        write_bytes_to_file(&pis_bytes, path)?;
        Ok(())
    }

    fn read_pis(full_path: &str) -> AnyhowResult<Self> {
        let pis_bytes = read_bytes_from_file(full_path)?;
        Ok(Halo2PoseidonPis(pis_bytes))
    }

    fn keccak_hash(&self) -> AnyhowResult<[u8; 32]> {
        let instances = self.get_instance()?;
        let hash = halo2_public_inputs_hash::<Keccak256Hasher>(&instances);
        Ok(hash)
    }

    // TODO: need to check that
    fn get_data(&self) -> AnyhowResult<Vec<String>> {
        let a: Vec<Vec<Fr>> = serde_json::from_str(&String::from_utf8(self.0.clone()).map_err(|err| anyhow!(error_line!(err)))?).map_err(|e| anyhow!(error_line!(e)))?;
        let pis = a
            .iter()
            .flat_map(|fr| {
                fr.iter()
                    .map(|elm| {
                        let bytes = elm.to_bytes_le();
                        BigUint::from_bytes_le(&bytes).to_string()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Ok(pis)
    }
}

impl Halo2PoseidonPis {
    pub fn get_instance(&self) -> AnyhowResult<Vec<Vec<Fr>>> {
        let instances: Vec<Vec<Fr>> = serde_json::from_slice(&self.0)?;
        Ok(instances)
    }
}

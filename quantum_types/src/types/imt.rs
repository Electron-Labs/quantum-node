use anyhow::{anyhow, Result as AnyhowResult};
use borsh::{BorshDeserialize, BorshSerialize};
use keccak_hash::keccak;
use quantum_utils::{
    error_line,
    file::{read_bytes_from_file, write_bytes_to_file},
};
use serde::{Deserialize, Serialize};
use tiny_merkle::{proof::Position, MerkleTree};

use super::hash::{KeccakHashOut, KeccakHasher};

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
pub struct QuantumLeaf {
    pub value: KeccakHashOut,
    pub next_value: KeccakHashOut,
    pub next_idx: [u8; 8],
}

impl QuantumLeaf {
    pub fn serialize(&self) -> Vec<u8> {
        let mut serialized = vec![];
        serialized.extend(self.value.clone().0);
        serialized.extend(self.next_value.clone().0);
        serialized.extend(self.next_idx.clone());
        serialized
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ImtTree {
    pub leaves: Vec<QuantumLeaf>,
}

impl ImtTree {
    pub fn serialise_imt_tree(&self) -> AnyhowResult<Vec<u8>> {
        let mut buffer: Vec<u8> = Vec::new();
        BorshSerialize::serialize(&self, &mut buffer)?;
        Ok(buffer)
    }

    pub fn deserialise_imt_tree(bytes: &mut &[u8]) -> AnyhowResult<Self> {
        let imt_tree: ImtTree =
            BorshDeserialize::deserialize(bytes).map_err(|err| anyhow!(error_line!(err)))?;
        Ok(imt_tree)
    }

    pub fn dump_tree(&self, path: &str) -> AnyhowResult<()> {
        let imt_bytes = self.serialise_imt_tree()?;
        write_bytes_to_file(&imt_bytes, path)?;
        Ok(())
    }

    pub fn read_tree(path: &str) -> AnyhowResult<Self> {
        let imt_bytes = read_bytes_from_file(path)?;
        let imt_tree = ImtTree::deserialise_imt_tree(&mut imt_bytes.as_slice())?;
        Ok(imt_tree)
    }

    pub fn get_mtree(&self) -> MerkleTree<KeccakHasher> {
        let leaves_structs = self.leaves.clone();
        let leaves = leaves_structs
            .iter()
            .map(|leaf_struct| keccak(&leaf_struct.serialize()).0)
            .collect::<Vec<[u8; 32]>>();
        MerkleTree::<KeccakHasher>::from_leaves(leaves, None)
    }

    pub fn get_imt_proof(
        &self,
        leaf_val: KeccakHashOut,
    ) -> AnyhowResult<(Vec<Vec<u8>>, Vec<u8>, QuantumLeaf)> {
        let leafs = self.leaves.clone();
        let mut leaf_asked: Option<QuantumLeaf> = None;
        for leaf in leafs {
            if leaf.value == leaf_val {
                leaf_asked = Some(leaf.clone());
                break;
            }
        }
        if leaf_asked.is_none() {
            return Err(anyhow!(error_line!("Couldnt find a value in leaves")));
        }
        let leaf = leaf_asked.unwrap();
        let mtree = self.get_mtree();
        let imt_proof = mtree.proof(keccak(&leaf.serialize()).0);
        if imt_proof.is_none() {
            return Err(anyhow::Error::msg("Couldnt find a valid merkle proof"));
        }
        let mut proof = Vec::<Vec<u8>>::new();
        let mut proof_helper = Vec::<u8>::new();

        imt_proof.unwrap().proofs.iter().for_each(|elm| {
            proof.push(elm.data.to_vec());
            let posn = &elm.position;
            match posn {
                Position::Left => proof_helper.push(0),
                Position::Right => proof_helper.push(1),
            }
        });

        // return proof = ([next_leaf_val, next_idx, merkle_proof ...], merkle_proof_helper)
        Ok((proof, proof_helper, leaf))
    }
}

use borsh::{BorshDeserialize, BorshSerialize};
use keccak_hash::keccak;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct KeccakHasher;

impl tiny_merkle::Hasher for KeccakHasher {
    type Hash = [u8; 32];

    fn hash(value: &[u8]) -> Self::Hash {
        keccak(value).0
    }
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
pub struct KeccakHashOut(pub [u8; 32]);

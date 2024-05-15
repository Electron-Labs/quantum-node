use borsh::{BorshSerialize, BorshDeserialize};
/*
type VerifyingKey struct {
	// [α]₁, [Kvk]₁
	G1 struct {
		Alpha       curve.G1Affine
		Beta, Delta curve.G1Affine   // unused, here for compatibility purposes
		K           []curve.G1Affine // The indexes correspond to the public wires
	}

	// [β]₂, [δ]₂, [γ]₂,
	// -[δ]₂, -[γ]₂: see proof.Verify() for more details
	G2 struct {
		Beta, Delta, Gamma curve.G2Affine
		// contains filtered or unexported fields
	}

	CommitmentKey                pedersen.VerifyingKey
	PublicAndCommitmentCommitted [][]int // indexes of public/commitment committed variables
	// contains filtered or unexported fields
}
 */
// We will represent 1 Fr Element by String
#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Fq {
    X: String, // Since we dont wanna do any field operations on this serve, String should work
    Y: String
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct Fq2 {
    X: Fq,
    Y: Fq
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct G1Struct {
    Alpha: Fq,
    Beta: Fq,
    Delta: Fq,
    K: Vec<Fq>
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct G2Struct {
    Beta: Fq2,
    Delta: Fq2,
    Gamma: Fq2
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct PedersenCommitmentKey {
	G: Fq2,
	GRootSigmaNeg: Fq2
}

#[derive(Clone, BorshSerialize, BorshDeserialize)]
pub struct GnarkVkey {
	G1: G1Struct,
	G2: G2Struct,
	CommitmentKey: PedersenCommitmentKey,
	// We wont support gnark proofs which have PublicAndCommitmentCommitted non-empty
	// PublicAndCommitmentCommitted: []
}



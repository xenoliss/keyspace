use serde::{Deserialize, Serialize};

use super::{exclusion::ExclusionProof, inclusion::InclusionProof};

/// A node proof mutation that can either be an inclusion or an exclusion proof.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum NodeProof<K, V> {
    /// An [InclusionProof].
    Inclusion(InclusionProof<K, V>),
    /// An [ExclusionProof].
    Exclusion(ExclusionProof<K, V>),
}

// TODO
// Crate Documentation.

/*

# Useful facts
# (Should go in the crate documentation)

- `get_domain (Fork, Epoch, genesis_validators_root, Domain_type)`
  - with Fork               and Epoch                   -> fork_version     `(LH) fork.get_fork_version`
    - with fork_version     and genesis_validators_root -> fork_data_root   `compute_fork_data_root`
      - with fork_data_root and Domain_type             -> domain           `compute_domain`

- with domain         and obj_root                      -> signing_root!

We need to send through the wire, then:

- Fork                      https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#fork
- Epoch                     https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#custom-types
- genesis_validators_root   Hash256

- and, of course

- bls_domain       "beacon_proposer" "beacon_attestation", "randao" (to deduce which Domain type we need, and validate the data)
- data             if we are sending a "block" or "attestation"

# Useful note

We are passing the `spec` parameter to leverage the `get_domain` function from LH

## BLS DOMAINS

- https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#domain-types

DOMAIN_BEACON_PROPOSER  DomainType('0x00000000')
DOMAIN_BEACON_ATTESTER  DomainType('0x01000000')
DOMAIN_RANDAO   DomainType('0x02000000')
DOMAIN_DEPOSIT  DomainType('0x03000000')
DOMAIN_VOLUNTARY_EXIT   DomainType('0x04000000')
DOMAIN_SELECTION_PROOF  DomainType('0x05000000')
DOMAIN_AGGREGATE_AND_PROOF  DomainType('0x06000000')

*/

mod http_client;

#[cfg(test)]
mod testing;

pub use http_client::RemoteSignerHttpClient;
use reqwest::StatusCode;
pub use reqwest::Url;
use serde::{Deserialize, Serialize};
use types::{AttestationData, BeaconBlock, Domain, Epoch, EthSpec, Fork, Hash256, SignedRoot};

#[derive(Debug)]
pub enum Error {
    /// The `reqwest` client raised an error.
    Reqwest(reqwest::Error),
    /// The server returned an error message where the body was able to be parsed.
    ServerMessage(String),
    /// The server returned an error message where the body was unable to be parsed.
    StatusCode(StatusCode),
    /// The supplied URL is badly formatted. It should look something like `http://127.0.0.1:5052`.
    InvalidUrl(Url),
    /// The supplied parameter is invalid.
    InvalidParameter(String),
}

#[derive(Serialize)]
struct RemoteSignerRequestBody<T> {
    // TODO Document
    bls_domain: String,

    // TODO Document
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,

    // TODO Document
    fork: Fork,

    // TODO Document
    epoch: Epoch,

    // TODO Document
    genesis_validators_root: Hash256,

    // As we are iterating over EIP-3030, we will see that this
    // field will be quickly deprecated.
    #[serde(rename(serialize = "signingRoot"))]
    signing_root: Hash256,
}

#[derive(Deserialize)]
struct RemoteSignerResponseBody {
    signature: String,
}

pub trait RemoteSignerObject: SignedRoot + Serialize {
    fn get_bls_domain_str(&self, domain: Domain) -> Result<String, Error>;
}

impl<E: EthSpec> RemoteSignerObject for BeaconBlock<E> {
    fn get_bls_domain_str(&self, domain: Domain) -> Result<String, Error> {
        match domain {
            Domain::BeaconProposer => Ok("beacon_proposer".to_string()),
            _ => Err(Error::InvalidParameter(format!(
                "Domain mismatch for block. Expected BeaconProposer, got {:?}",
                domain
            ))),
        }
    }
}

impl RemoteSignerObject for AttestationData {
    fn get_bls_domain_str(&self, domain: Domain) -> Result<String, Error> {
        match domain {
            Domain::BeaconAttester => Ok("beacon_attester".to_string()),
            _ => Err(Error::InvalidParameter(format!(
                "Domain mismatch for attestation. Expected BeaconAttester, got {:?}",
                domain
            ))),
        }
    }
}

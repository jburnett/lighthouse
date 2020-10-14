//! Enables the [Lighthouse Ethereum 2.0 Client] to consume signatures from the
//! [BLS Remote Signer].
//!
//! ## About
//!
//! The lighthouse client needs to include this crate, and implement the
//! adequate bypasses and CLI flags needed to find the remote signer and perform
//! the HTTP requests.
//!
//! At the current version, `0.1.0`, the remote signer will only check the
//! correctness of the JSON payload, and the inclusion of the `signatureRoot`
//! field within. There is **no** validation of the pre-image. Future versions
//! of the signer (as per the [EIP-3030] specification) will enforce the
//! pre-image fields, deprecating the `signatureRoot` field. It is the role of
//! the signer to verify the message fields for slashing validation and other
//! controls.`
//!
//! ## Usage
//!
//! ### RemoteSignerHttpClient
//!
//! Just provide an `Url` and a timeout
//!
//! ```
//! use remote_signer::RemoteSignerHttpClient;
//! use reqwest::{ClientBuilder, Url};
//! use tokio::time::Duration;
//!
//! let url: Url = "http://127.0.0.1:9000".parse().unwrap();
//! let reqwest_client = ClientBuilder::new()
//!       .timeout(Duration::from_secs(12))
//!       .build()
//!       .unwrap();
//!
//! let signer = RemoteSignerHttpClient::from_components(url, reqwest_client);
//!
//! ```
//!
//! ## sign API
//!
//! `POST /sign/:public-key`
//!
//! ### Arguments
//!
//! #### `public_key`
//!
//! Goes within the url to identify the key we want to use as signer.
//!
//! #### `bls_domain`
//!
//! [BLS Signature domain]. Supporting `BeaconProposer`, `BeaconAttester`,
//! `Randao`.
//!
//! #### `data`
//!
//! An `Option<T>` wrapping a `BeaconBlock`, an  `AttestationData`, or `None`.
//!
//! #### `fork`
//!
//! A [`Fork`] object, containing previous and current versions.
//!
//! #### `epoch`
//!
//! An [`Epoch`] object wrapping the epoch represented in `u64`.
//!
//! #### `genesis_validators_root`
//!
//! A [`Hash256`] for domain separation and chain versioning.
//!
//! #### `spec`
//!
//! The chain spec in use: `sign()` leverages its functions to compute the
//! domain.
//!
//! ### Behavior
//!
//! Upon receiving and validating the parameters, the signer sends through the
//! wire a serialized `RemoteSignerRequestBody`. Receiving a `200` message with
//! the `signature` field inside a JSON payload, or an error.
//!
//! ## How it works
//!
//! The production of a _local_ signature (i.e. inside the Lighthouse client)
//! has slight variations among the kind of objects (block, attestation,
//! randao).
//!
//! To sign a message, the following procedures are needed:
//!
//! * Get the `fork_version` - From the objects `Fork` and `Epoch`.
//! * Compute the [`fork_data_root`] - From the `fork_version` and the
//!   `genesis_validators_root`.
//! * Compute the [`domain`] - From the `fork_data_root` and the `bls_domain`.
//! * With the `domain`, the object (or `epoch` in the case of [`randao`])
//!   can be merkelized into its [`signing_root`] to be signed.
//!
//! In short, to obtain a signature from the remote signer, we need to produce
//! (and serialize) the following objects:
//!
//! * `fork`
//! * `epoch`
//! * `genesis_validators_root`
//! * `bls_domain`
//! * `data` of the object, if this is a Block proposal, or an attestation.
//!
//! And, of course, the identifier of the secret key, the `public_key`.
//!
//! ## Future Work
//!
//! ### EIP-3030
//!
//! Work is being done to [standardize the API of the remote signers].
//!
//! [`domain`]: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#compute_domain
//! [`Epoch`]: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#custom-types
//! [`fork_data_root`]: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#compute_fork_data_root
//! [`Fork`]: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#fork
//! [`Hash256`]: https://docs.rs/ethereum-types/0.9.2/ethereum_types/struct.H256.html
//! [`randao`]: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#randao
//! [`signing_root`]: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#compute_signing_root
//! [BLS Remote Signer]: https://github.com/sigp/rust-bls-remote-signer
//! [BLS Signature domain]: https://github.com/ethereum/eth2.0-specs/blob/dev/specs/phase0/beacon-chain.md#domain-types
//! [EIP-3030]: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3030.md
//! [Lighthouse Ethereum 2.0 Client]: https://github.com/sigp/lighthouse
//! [standardize the API of the remote signers]: https://github.com/ethereum/EIPs/blob/master/EIPS/eip-3030.md

mod http_client;

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
    /// BLS Signature domain. Supporting `BeaconProposer`, `BeaconAttester`,`Randao`.
    bls_domain: String,

    /// An `Option` wrapping a `BeaconBlock`, an  `AttestationData`, or `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,

    /// A `Fork` object containing previous and current versions.
    fork: Fork,

    /// An `Epoch` object wrapping the epoch represented in `u64`.
    epoch: Epoch,

    /// A `Hash256` for domain separation and chain versioning.
    genesis_validators_root: Hash256,

    /// As we are iterating over EIP-3030, we will see that this
    /// field will be quickly deprecated.
    #[serde(rename(serialize = "signingRoot"))]
    signing_root: Hash256,
}

#[derive(Deserialize)]
struct RemoteSignerResponseBody {
    signature: String,
}

/// Allows the verification of the BeaconBlock and AttestationData objects
/// to be sent through the wire, against their BLS Domains.
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

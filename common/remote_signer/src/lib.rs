// TODO
// Crate Documentation.
use reqwest::StatusCode;
pub use reqwest::Url;
use serde::{Deserialize, Serialize};
use types::{BeaconBlock, Domain, Epoch, EthSpec, Fork, Hash256, SignedRoot};

mod http_client;

pub use http_client::RemoteSignerHttpClient;

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
}

#[derive(Serialize)]
struct RemoteSignerRequestBody<T> {
    #[serde(rename(serialize = "type"))]
    data_type: String,

    fork: Fork,

    domain: String,

    data: T,

    #[serde(rename(serialize = "signingRoot"))]
    signing_root: Hash256,
}

#[derive(Deserialize)]
struct RemoteSignerResponseBody {
    signature: String,
}

pub trait RemoteSignerObject: Serialize + SignedRoot {
    fn epoch(&self) -> Epoch;
    fn get_bls_domain(&self) -> Domain;
    fn get_type_str(&self) -> String;
}

impl<E: EthSpec> RemoteSignerObject for BeaconBlock<E> {
    fn epoch(&self) -> Epoch {
        self.epoch()
    }

    fn get_bls_domain(&self) -> Domain {
        Domain::BeaconProposer
    }

    fn get_type_str(&self) -> String {
        "block".to_string()
    }
}

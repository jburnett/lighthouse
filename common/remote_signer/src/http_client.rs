use crate::{
    Error, RemoteSignerObject, RemoteSignerRequestBody, RemoteSignerResponseBodyError,
    RemoteSignerResponseBodyOK,
};
use reqwest::StatusCode;
pub use reqwest::Url;
use types::{ChainSpec, Domain, Fork, Hash256};

/// A wrapper around `reqwest::Client` which provides convenience methods
/// to interface with a BLS Remote Signer.
pub struct RemoteSignerHttpClient {
    client: reqwest::Client,
    server: Url,
}

impl RemoteSignerHttpClient {
    pub fn from_components(server: Url, client: reqwest::Client) -> Self {
        Self { client, server }
    }

    /// `POST /sign/:public-key`
    ///
    /// # Arguments
    ///
    /// * `public_key`              - Goes within the url to identify the key we want to use as signer.
    /// * `bls_domain`              - BLS Signature domain. Supporting `BeaconProposer`, `BeaconAttester`,`Randao`.
    /// * `data`                    - A `BeaconBlock`, `AttestationData`, or `Epoch`.
    /// * `fork`                    - A `Fork` object containing previous and current versions.
    /// * `genesis_validators_root` - A `Hash256` for domain separation and chain versioning.
    /// * `spec`                    - The chain spec in use: `sign()` leverages its functions to compute the domain.
    ///
    /// It sends through the wire a serialized `RemoteSignerRequestBody`.
    pub async fn sign<R: RemoteSignerObject>(
        &self,
        public_key: &str,
        bls_domain: Domain,
        data: R,
        fork: Fork,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> Result<String, Error> {
        if public_key.is_empty() {
            return Err(Error::InvalidParameter(
                "Empty parameter public_key".to_string(),
            ));
        }

        let mut path = self.server.clone();
        path.path_segments_mut()
            .map_err(|()| Error::InvalidUrl(self.server.clone()))?
            .push("sign")
            .push(public_key);

        let get_domain = |epoch| spec.get_domain(epoch, bls_domain, &fork, genesis_validators_root);

        let get_signing_root = |obj: &R| -> Result<(String, Hash256), Error> {
            // Validate that `bls_domain` maps to the right object given in `data`.
            let bls_domain: String = obj.get_bls_domain_str(bls_domain)?;
            let signing_root = obj.signing_root(get_domain(obj.get_epoch()));
            Ok((bls_domain, signing_root))
        };

        let (bls_domain, signing_root) = match bls_domain {
            Domain::BeaconProposer => get_signing_root(&data),

            Domain::BeaconAttester => get_signing_root(&data),

            Domain::Randao => get_signing_root(&data),

            _ => Err(Error::InvalidParameter(format!(
                "Unsupported BLS Domain: {:?}",
                bls_domain
            ))),
        }?;

        let body = RemoteSignerRequestBody {
            bls_domain,
            data,
            fork,
            genesis_validators_root,
            signing_root,
        };

        let response = self
            .client
            .post(path)
            .json(&body)
            .send()
            .await
            .map_err(Error::Reqwest)?;

        match response.status() {
            StatusCode::OK => match response.json::<RemoteSignerResponseBodyOK>().await {
                Ok(resp_json) => Ok(resp_json.signature),
                Err(e) => Err(Error::Reqwest(e)),
            },
            _ => match response.json::<RemoteSignerResponseBodyError>().await {
                Ok(resp_json) => Err(Error::ServerMessage(resp_json.error)),
                Err(e) => Err(Error::Reqwest(e)),
            },
        }
    }
}

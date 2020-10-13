use crate::{Error, RemoteSignerObject, RemoteSignerRequestBody, RemoteSignerResponseBody};
use reqwest::StatusCode;
pub use reqwest::Url;
use reqwest::{IntoUrl, Response};
use serde::Serialize;
use types::{ChainSpec, Domain, Epoch, Fork, Hash256, SignedRoot};

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
    /// * `data`                    - An `Option` wrapping a `BeaconBlock`, an  `AttestationData`, or `None`.
    /// * `fork`                    - A `Fork` object containing previous and current versions.
    /// * `epoch`                   - A `Epoch` object wrapping the epoch represented in `u64`.
    /// * `genesis_validators_root` - A `Hash256` for domain separation and chain versioning
    /// * `spec`                    - The chain spec in use: `sign()` leverages its functions to compute the domain.
    ///
    /// It sends through the wire a serialized `RemoteSignerRequestBody`.
    #[allow(clippy::too_many_arguments)]
    pub async fn sign<T: RemoteSignerObject>(
        &self,
        public_key: &str,
        bls_domain: Domain,
        data: Option<&T>,
        fork: &Fork,
        epoch: Epoch,
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

        let get_domain = || spec.get_domain(epoch, bls_domain, fork, genesis_validators_root);

        let (bls_domain, signing_root) = match bls_domain {
            Domain::BeaconProposer => match &data {
                Some(obj) => {
                    let bls_domain: String = obj.get_bls_domain_str(bls_domain)?;
                    let signing_root = obj.signing_root(get_domain());
                    Ok((bls_domain, signing_root))
                }
                None => return Err(Error::InvalidParameter("".to_string())),
            },
            Domain::BeaconAttester => match &data {
                Some(obj) => {
                    let bls_domain: String = obj.get_bls_domain_str(bls_domain)?;
                    let signing_root = obj.signing_root(get_domain());
                    Ok((bls_domain, signing_root))
                }
                None => return Err(Error::InvalidParameter("".to_string())),
            },
            Domain::Randao => Ok(("randao".to_string(), epoch.signing_root(get_domain()))),
            _ => Err(Error::InvalidParameter(format!(
                "Unsupported BLS Domain: {:?}",
                bls_domain
            ))),
        }?;

        let body = RemoteSignerRequestBody {
            bls_domain,
            data,
            fork: *fork, // TODO. 1) Ugly? 2) How good this serializes? 3) Serializing API specs?
            epoch,
            genesis_validators_root,
            signing_root,
        };

        let response = self.post(path, &body).await?;

        let signature = match response.json::<RemoteSignerResponseBody>().await {
            Ok(resp_json) => Ok(resp_json.signature),
            Err(e) => Err(Error::Reqwest(e)),
        }?;

        Ok(signature)
    }

    /// Performs an HTTP POST request.
    async fn post<T: Serialize, U: IntoUrl>(&self, url: U, body: &T) -> Result<Response, Error> {
        let response = self
            .client
            .post(url)
            .json(body)
            .send()
            .await
            .map_err(Error::Reqwest)?;

        ok_or_error(response).await
    }
}

/// Returns `Ok(response)` if the response is a `200 OK` response. Otherwise, creates an
/// appropriate error message.
async fn ok_or_error(response: Response) -> Result<Response, Error> {
    let status = response.status();

    if status == StatusCode::OK {
        Ok(response)
    } else if let Ok(message) = response.json().await {
        Err(Error::ServerMessage(message))
    } else {
        Err(Error::StatusCode(status))
    }
}

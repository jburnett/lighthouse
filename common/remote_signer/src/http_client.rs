use crate::{Error, RemoteSignerObject, RemoteSignerRequestBody, RemoteSignerResponseBody};
use reqwest::StatusCode;
pub use reqwest::Url;
use reqwest::{IntoUrl, Response};
use serde::Serialize;
use types::{ChainSpec, Fork, Hash256};

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
    pub async fn sign<T: RemoteSignerObject>(
        &self,
        obj: &T,
        public_key: &str,
        fork: &Fork,
        genesis_validators_root: Hash256,
        spec: &ChainSpec,
    ) -> Result<String, Error> {
        let mut path = self.server.clone();
        path.path_segments_mut()
            .map_err(|()| Error::InvalidUrl(self.server.clone()))?
            .push("sign")
            .push(public_key);

        let domain = spec.get_domain(
            obj.epoch(),
            obj.get_bls_domain(),
            fork,
            genesis_validators_root,
        );
        let signing_root = obj.signing_root(domain);

        let body = RemoteSignerRequestBody {
            data_type: obj.get_type_str(),
            fork: *fork, // TODO. Ugly?
            domain: domain.to_string(),
            data: obj,
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

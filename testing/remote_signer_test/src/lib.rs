mod constants;
mod local_signer_test_data;
mod remote_signer_test_data;
mod test_objects;

pub use constants::*;
pub use local_signer_test_data::*;
use remote_signer::{Error, RemoteSignerHttpClient, RemoteSignerObject, Url};
pub use remote_signer_test_data::*;
use reqwest::ClientBuilder;
use tokio::runtime::Builder;
use tokio::time::Duration;
use types::{EthSpec, MainnetEthSpec};

type E = MainnetEthSpec;

pub fn set_up_test_client(test_signer_address: &str) -> RemoteSignerHttpClient {
    let url: Url = test_signer_address.parse().unwrap();
    let reqwest_client = ClientBuilder::new()
        .timeout(Duration::from_secs(12))
        .build()
        .unwrap();

    RemoteSignerHttpClient::from_components(url, reqwest_client)
}

pub fn do_sign_request<E: EthSpec, T: RemoteSignerObject>(
    test_client: &RemoteSignerHttpClient,
    test_input: RemoteSignerTestData<E, T>,
) -> Result<String, Error> {
    let mut runtime = Builder::new()
        .threaded_scheduler()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(test_client.sign(
        &test_input.public_key,
        test_input.bls_domain,
        test_input.data,
        test_input.fork,
        test_input.genesis_validators_root,
        &test_input.spec,
    ))
}

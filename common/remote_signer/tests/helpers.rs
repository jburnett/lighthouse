use remote_signer::{RemoteSignerHttpClient, Url};
use reqwest::ClientBuilder;
use tokio::time::Duration;

pub const HAPPY_PATH_BLOCK_SIGNATURE: &str = "0xaa38076a7f03ecd0f5dbb9a0ea966c1f2a859a6f11820eb498f73e0ec91f41e176947361e39cbd7661bccc827db7d8f400bd06d0a7fdd8a4a40683b7e704f72c8461e63f61c6b204c2debe7ef16e399a882ce8433c700b6bceca7b33e60a3f5f";

pub fn set_up_test_client(test_signer_address: &str) -> RemoteSignerHttpClient {
    let url: Url = test_signer_address.parse().unwrap();
    let reqwest_client = ClientBuilder::new()
        .timeout(Duration::from_secs(12))
        .build()
        .unwrap();

    RemoteSignerHttpClient::from_components(url, reqwest_client)
}

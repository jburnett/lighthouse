use remote_signer::{RemoteSignerHttpClient, Url};
use reqwest::ClientBuilder;
use tokio::time::Duration;

// TODO
// Find a better place to put this constant, to avoid this "allow dead_code" annotation.
#[allow(dead_code)]
pub const HAPPY_PATH_BLOCK_SIGNATURE: &str = "0xaa38076a7f03ecd0f5dbb9a0ea966c1f2a859a6f11820eb498f73e0ec91f41e176947361e39cbd7661bccc827db7d8f400bd06d0a7fdd8a4a40683b7e704f72c8461e63f61c6b204c2debe7ef16e399a882ce8433c700b6bceca7b33e60a3f5f";

#[allow(dead_code)]
pub const HAPPY_PATH_ATT_SIGNATURE: &str = "0xa3d6872b2f87422e86396e937d7a26e3cdaae5fc9686dbe139b28227fdfbba34347a3b69f8821625be4796f12cd3a9f911db87d0681064d5e38ae94da824160e799549d7f781bf657676f1d7c68af98d7aef3121f452f5aeeb03887c8b91702a";

pub fn set_up_test_client(test_signer_address: &str) -> RemoteSignerHttpClient {
    let url: Url = test_signer_address.parse().unwrap();
    let reqwest_client = ClientBuilder::new()
        .timeout(Duration::from_secs(12))
        .build()
        .unwrap();

    RemoteSignerHttpClient::from_components(url, reqwest_client)
}

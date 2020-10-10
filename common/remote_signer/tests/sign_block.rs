mod sign_block {

    use helpers::*;
    use remote_signer::{Error, RemoteSignerHttpClient, Url};
    use reqwest::ClientBuilder;
    use tokio::runtime::Builder;
    use tokio::time::Duration;
    use types::{BeaconBlock, ChainSpec, EthSpec, Fork, Hash256, MainnetEthSpec};

    const HAPPY_PATH_SIGNATURE: &str = "0xaa38076a7f03ecd0f5dbb9a0ea966c1f2a859a6f11820eb498f73e0ec91f41e176947361e39cbd7661bccc827db7d8f400bd06d0a7fdd8a4a40683b7e704f72c8461e63f61c6b204c2debe7ef16e399a882ce8433c700b6bceca7b33e60a3f5f";

    type E = MainnetEthSpec;

    fn set_up_test_client(test_signer_address: &str) -> RemoteSignerHttpClient {
        let url: Url = test_signer_address.parse().unwrap();
        let reqwest_client = ClientBuilder::new()
            .timeout(Duration::from_secs(12))
            .build()
            .unwrap();

        RemoteSignerHttpClient::from_components(url, reqwest_client)
    }

    struct SignBlockTestData<'a, E: EthSpec> {
        block: BeaconBlock<E>,
        public_key: &'a str,
        fork: Fork,
        genesis_validators_root: Hash256,
        spec: ChainSpec,
    }

    impl<E: EthSpec> SignBlockTestData<'_, E> {
        fn new(spec: &ChainSpec) -> Self {
            Self {
                block: BeaconBlock::empty(spec),
                public_key: PUBLIC_KEY_1,
                fork: Fork {
                    previous_version: spec.genesis_fork_version,
                    current_version: spec.genesis_fork_version,
                    epoch: E::genesis_epoch(),
                },
                genesis_validators_root: Hash256::from_low_u64_be(0xc137),
                spec: spec.clone(),
            }
        }
    }

    fn do_sign_request<E: EthSpec>(
        test_client: &RemoteSignerHttpClient,
        test_input: &SignBlockTestData<E>,
    ) -> Result<String, Error> {
        let mut runtime = Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(test_client.sign(
            &test_input.block,
            &test_input.public_key,
            &test_input.fork,
            test_input.genesis_validators_root,
            &test_input.spec,
        ))
    }

    #[test]
    fn happy_path() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);
        let test_input: SignBlockTestData<E> =
            SignBlockTestData::new(&MainnetEthSpec::default_spec());

        let signature = do_sign_request(&test_client, &test_input);

        assert_eq!(signature.unwrap(), HAPPY_PATH_SIGNATURE);

        test_signer.shutdown();
    }
}

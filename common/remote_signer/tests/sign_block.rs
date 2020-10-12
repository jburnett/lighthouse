mod helpers;

mod sign_block {
    use crate::helpers::*;
    use remote_signer::{Error, RemoteSignerHttpClient};
    use server_helpers::*;
    use tokio::runtime::Builder;
    use types::{BeaconBlock, ChainSpec, EthSpec, Fork, Hash256, MainnetEthSpec};

    type E = MainnetEthSpec;

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

        assert_eq!(signature.unwrap(), HAPPY_PATH_BLOCK_SIGNATURE);

        test_signer.shutdown();
    }
}

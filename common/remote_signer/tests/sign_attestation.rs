mod helpers;

mod sign_attestation {
    use crate::helpers::*;
    use remote_signer::{Error, RemoteSignerHttpClient};
    use server_helpers::*;
    use tokio::runtime::Builder;
    use types::{
        AttestationData, ChainSpec, Checkpoint, Epoch, EthSpec, Fork, Hash256, MainnetEthSpec, Slot,
    };

    type E = MainnetEthSpec;

    struct SignAttestationTestData<'a> {
        attestation: AttestationData,
        public_key: &'a str,
        fork: Fork,
        genesis_validators_root: Hash256,
        spec: ChainSpec,
    }

    impl SignAttestationTestData<'_> {
        fn new(spec: &ChainSpec) -> Self {
            let source = build_checkpoint(42);
            let target = build_checkpoint(73);
            let index = 0u64;
            let slot = Slot::from(0u64);
            let attestation = AttestationData {
                slot,
                index,
                beacon_block_root: Hash256::zero(),
                source,
                target,
            };

            Self {
                attestation,
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

    fn build_checkpoint(epoch_num: u64) -> Checkpoint {
        Checkpoint {
            epoch: Epoch::from(epoch_num),
            root: Hash256::zero(),
        }
    }

    fn do_sign_request(
        test_client: &RemoteSignerHttpClient,
        test_input: &SignAttestationTestData,
    ) -> Result<String, Error> {
        let mut runtime = Builder::new()
            .threaded_scheduler()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(test_client.sign(
            &test_input.attestation,
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

        let test_input: SignAttestationTestData =
            SignAttestationTestData::new(&MainnetEthSpec::default_spec());

        let signature = do_sign_request(&test_client, &test_input);

        assert_eq!(signature.unwrap(), HAPPY_PATH_ATT_SIGNATURE);

        test_signer.shutdown();
    }
}

// Quick and dirty solution to have your crate tests in a separate place,
// sharing a library, and not polluting the workspace.
#[cfg(test)]
mod helpers {
    use crate::{Error, RemoteSignerHttpClient, RemoteSignerObject, Url};
    use reqwest::ClientBuilder;
    use serde_derive::Serialize;
    use server_helpers::*;
    use std::marker::PhantomData;
    use tokio::runtime::Builder;
    use tokio::time::Duration;
    use tree_hash::TreeHash;
    use types::{ChainSpec, Checkpoint, Domain, Epoch, EthSpec, Fork, Hash256, SignedRoot};

    pub const HAPPY_PATH_BLOCK_SIGNATURE: &str = "0xaa38076a7f03ecd0f5dbb9a0ea966c1f2a859a6f11820eb498f73e0ec91f41e176947361e39cbd7661bccc827db7d8f400bd06d0a7fdd8a4a40683b7e704f72c8461e63f61c6b204c2debe7ef16e399a882ce8433c700b6bceca7b33e60a3f5f";

    pub const HAPPY_PATH_ATT_SIGNATURE: &str = "0xa3d6872b2f87422e86396e937d7a26e3cdaae5fc9686dbe139b28227fdfbba34347a3b69f8821625be4796f12cd3a9f911db87d0681064d5e38ae94da824160e799549d7f781bf657676f1d7c68af98d7aef3121f452f5aeeb03887c8b91702a";

    pub const HAPPY_PATH_RANDAO_SIGNATURE: &str = "0xa8c86071a4ced56ed1b39256c2befd25f229d4c6e11151ba2de314c681e5c39f86f5e9e00d0cd66e46156dafe2e464a80d619786b63b0e0942c7609b5a24aaa33ca3bfbe5d09f043d0c0c21d0153f71ab255844253fb6203d0db534462de971b";

    pub fn set_up_test_client(test_signer_address: &str) -> RemoteSignerHttpClient {
        let url: Url = test_signer_address.parse().unwrap();
        let reqwest_client = ClientBuilder::new()
            .timeout(Duration::from_secs(12))
            .build()
            .unwrap();

        RemoteSignerHttpClient::from_components(url, reqwest_client)
    }

    #[derive(Serialize)]
    pub struct DummyRandao {}
    impl SignedRoot for DummyRandao {}
    impl TreeHash for DummyRandao {
        fn tree_hash_type() -> tree_hash::TreeHashType {
            todo!()
        }
        fn tree_hash_packed_encoding(&self) -> std::vec::Vec<u8> {
            todo!()
        }
        fn tree_hash_packing_factor() -> usize {
            todo!()
        }
        fn tree_hash_root(&self) -> Hash256 {
            todo!()
        }
    }
    impl RemoteSignerObject for DummyRandao {
        fn get_bls_domain_str(
            &self,
            _: types::Domain,
        ) -> std::result::Result<std::string::String, Error> {
            todo!()
        }
    }

    pub struct SignTestData<'a, E: EthSpec, T: RemoteSignerObject> {
        public_key: String,
        bls_domain: Domain,
        data: Option<&'a T>,
        fork: Fork,
        epoch: Epoch,
        genesis_validators_root: Hash256,
        spec: ChainSpec,

        _phantom: PhantomData<E>,
    }

    impl<'a, E: EthSpec, T: RemoteSignerObject> SignTestData<'a, E, T> {
        pub fn new(spec: &ChainSpec, data: Option<&'a T>, bls_domain: Domain) -> Self {
            Self {
                public_key: PUBLIC_KEY_1.to_string(),
                bls_domain,
                data,
                fork: Fork {
                    previous_version: spec.genesis_fork_version,
                    current_version: spec.genesis_fork_version,
                    epoch: E::genesis_epoch(),
                },
                epoch: Epoch::new(42),
                genesis_validators_root: Hash256::from_low_u64_be(0xc137),
                spec: spec.clone(),

                _phantom: PhantomData,
            }
        }
    }

    pub fn build_checkpoint(epoch_num: u64) -> Checkpoint {
        Checkpoint {
            epoch: Epoch::from(epoch_num),
            root: Hash256::zero(),
        }
    }

    pub fn do_sign_request<E: EthSpec, T: RemoteSignerObject>(
        test_client: &RemoteSignerHttpClient,
        test_input: &SignTestData<E, T>,
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
            &test_input.fork,
            test_input.epoch,
            test_input.genesis_validators_root,
            &test_input.spec,
        ))
    }
}

#[cfg(test)]
mod sign_block {
    use crate::testing::helpers::*;
    use server_helpers::*;
    use types::{BeaconBlock, Domain, EthSpec, MainnetEthSpec};

    #[test]
    fn happy_path() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);

        // TODO
        // This bit can be cleaner.
        let spec = &MainnetEthSpec::default_spec();
        let block = BeaconBlock::empty(spec);
        let test_input: SignTestData<MainnetEthSpec, BeaconBlock<MainnetEthSpec>> =
            SignTestData::new(spec, Some(&block), Domain::BeaconProposer);
        // end warning

        let signature = do_sign_request(&test_client, &test_input);

        assert_eq!(signature.unwrap(), HAPPY_PATH_BLOCK_SIGNATURE);

        test_signer.shutdown();
    }
}

#[cfg(test)]
mod sign_attestation {
    use crate::testing::helpers::*;
    use server_helpers::*;
    use types::{AttestationData, Domain, EthSpec, Hash256, MainnetEthSpec, Slot};

    #[test]
    fn happy_path() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);

        // TODO
        // In case you are wondering, this is not clean at all!
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
        // end warning

        let spec = &MainnetEthSpec::default_spec();

        let test_input: SignTestData<MainnetEthSpec, AttestationData> =
            SignTestData::new(spec, Some(&attestation), Domain::BeaconAttester);

        let signature = do_sign_request(&test_client, &test_input);

        assert_eq!(signature.unwrap(), HAPPY_PATH_ATT_SIGNATURE);

        test_signer.shutdown();
    }
}

#[cfg(test)]
mod randao {
    use crate::testing::helpers::*;
    use server_helpers::*;
    use types::{Domain, EthSpec, MainnetEthSpec};

    #[test]
    fn happy_path() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);

        // TODO
        // This bit can be cleaner.
        let spec = &MainnetEthSpec::default_spec();
        let test_input: SignTestData<MainnetEthSpec, DummyRandao> =
            SignTestData::new(spec, None, Domain::Randao);
        // end warning

        // TODO
        // Test that the data field is actually empty!
        // i.e. the hack works.

        let signature = do_sign_request(&test_client, &test_input);

        assert_eq!(signature.unwrap(), HAPPY_PATH_RANDAO_SIGNATURE);

        test_signer.shutdown();
    }
}

// TODO
// - Test from_components with an invalid url.
// - We will hunt for errors that we are not having
//   using the conventional sign API, and
//   test them here.
// - Check that I am building the right json for the three cases.
// - Tests cases the remote signer accounts for:
//   missing field signingRoot (400)
//   bad request
//   etc
// - Another silly idea:
//   Let's do the very test agains the "real" `.sign()` functions
// - Test that serde is effectively NOT sending data with None.

use crate::test_objects::*;
use crate::E;
use remote_signer::RemoteSignerObject;
use server_helpers::*;
use std::marker::PhantomData;
use types::{AttestationData, BeaconBlock, ChainSpec, Domain, Epoch, EthSpec, Fork, Hash256};

pub struct RemoteSignerTestData<E: EthSpec, T: RemoteSignerObject> {
    pub public_key: String,
    pub bls_domain: Domain,
    pub data: T,
    pub fork: Fork,
    pub genesis_validators_root: Hash256,
    pub spec: ChainSpec,

    _phantom: PhantomData<E>,
}

impl<'a, E: EthSpec, T: RemoteSignerObject> RemoteSignerTestData<E, T> {
    pub fn new(public_key: &str, spec: &ChainSpec, data: T, bls_domain: Domain) -> Self {
        let epoch = data.get_epoch();

        Self {
            public_key: public_key.to_string(),
            bls_domain,
            data,
            fork: Fork {
                previous_version: [1; 4],
                current_version: [2; 4],
                epoch,
            },
            genesis_validators_root: Hash256::from_low_u64_be(0xc137),
            spec: spec.clone(),

            _phantom: PhantomData,
        }
    }
}

pub fn get_input_data_block(seed: u64) -> RemoteSignerTestData<E, BeaconBlock<E>> {
    let spec = &mut E::default_spec();
    let block: BeaconBlock<E> = get_block(seed);
    RemoteSignerTestData::new(PUBLIC_KEY_1, spec, block, Domain::BeaconProposer)
}

pub fn get_input_data_attestation(seed: u64) -> RemoteSignerTestData<E, AttestationData> {
    let spec = &E::default_spec();
    let attestation: AttestationData = get_attestation::<E>(seed);
    RemoteSignerTestData::new(PUBLIC_KEY_1, spec, attestation, Domain::BeaconAttester)
}

pub fn get_input_data_randao(seed: u64) -> RemoteSignerTestData<E, Epoch> {
    let spec = &E::default_spec();
    let epoch = Epoch::new(seed);
    RemoteSignerTestData::new(PUBLIC_KEY_1, spec, epoch, Domain::Randao)
}

pub fn get_test_input_and_set_domain<E: EthSpec, T: RemoteSignerObject>(
    f: fn(u64) -> RemoteSignerTestData<E, T>,
    bls_domain: Domain,
) -> RemoteSignerTestData<E, T> {
    let mut test_input = f(0xc137);
    test_input.bls_domain = bls_domain;

    test_input
}

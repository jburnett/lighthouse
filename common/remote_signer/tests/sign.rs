mod sign {
    use remote_signer::Error;
    use remote_signer_test::*;
    use server_helpers::*;
    use types::Domain;

    #[test]
    fn beacon_block_and_bls_domain_mismatch() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);

        macro_rules! test_case {
            ($f: expr, $bls_domain: expr, $msg: expr) => {
                match do_sign_request(&test_client, get_test_input_and_set_domain($f, $bls_domain))
                    .unwrap_err()
                {
                    Error::InvalidParameter(message) => assert_eq!(message, $msg),
                    _ => panic!(),
                }
            };
        }

        test_case!(
            get_input_data_block,
            Domain::BeaconAttester,
            "Domain mismatch for the BeaconBlock object. Expected BeaconProposer, got BeaconAttester"
        );
        test_case!(
            get_input_data_block,
            Domain::Randao,
            "Domain mismatch for the BeaconBlock object. Expected BeaconProposer, got Randao"
        );

        test_case!(
            get_input_data_attestation,
            Domain::BeaconProposer,
            "Domain mismatch for the AttestationData object. Expected BeaconAttester, got BeaconProposer"
        );

        test_case!(
            get_input_data_attestation,
            Domain::Randao,
            "Domain mismatch for the AttestationData object. Expected BeaconAttester, got Randao"
        );
        test_case!(
            get_input_data_randao,
            Domain::BeaconProposer,
            "Domain mismatch for the Epoch object. Expected Randao, got BeaconProposer"
        );
        test_case!(
            get_input_data_randao,
            Domain::BeaconAttester,
            "Domain mismatch for the Epoch object. Expected Randao, got BeaconAttester"
        );

        test_signer.shutdown();
    }
}
/*

*/

// # Test Strategy (TODO)
//
// ## Message preparation
// * Somebody sends a Domain type X, but data from a different type, or none (if it applies)
// * public_key field is empty
// * no_public_key_in_path (5 cases)
// * unsupported bls_domain
// * data: People implementing a new RemoteSignerObject: SignedRoot + Serialize
//   * what happens? Should pass? no?
// * bad fork field (establish what can make this a bad parameter)
// * bad epoch field (establish what can make this a bad parameter)
// * bad genesis validators root field (establish what can make this a bad parameter)
// * bad spec (establish what can make this a bad parameter)
//
// ## Errors that the remote signer shoould catch, but we don't trust and check anyways
// * additional_path_segments (3 cases)
// * invalid_public_key (6 cases)
// * invalid json (4 cases)
// * signing_root_in_json_not_a_string (4 cases)
//
// ## POST
// * Server unavailable / off
// * Server error (do the classic problem with the directories)
// * Malformed URL
// * Bad URL (to get 404s)
// * Bad response (use mock)
//   * no json
//   * missing_signing_root_in_json
//   * json, empty_signing_root_in_json#
//   * invalid_string_signing_root
//   * json, signature, but extra fields (should be ignored?)
// * secret key problems
//   * key_not_found
//   * invalid_secret_key
//   * key mismatch
// * Weird status code (418)
// * Timeout
//

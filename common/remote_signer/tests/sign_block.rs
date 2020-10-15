mod sign_block {
    use rand::Rng;
    use remote_signer_test::*;
    use server_helpers::*;

    #[test]
    fn sanity_check_deterministic() {
        let test_input_local = get_input_local_signer_block(0xc137);
        let local_signature = test_input_local.sign();

        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);
        let test_input = get_input_data_block(0xc137);

        let remote_signature = do_sign_request(&test_client, test_input);

        assert_eq!(local_signature, remote_signature.unwrap());
        assert_eq!(local_signature, HAPPY_PATH_BLOCK_SIGNATURE_C137);
    }

    #[test]
    fn sanity_check_random() {
        let mut rng = rand::thread_rng();
        let seed = rng.gen::<u64>() / 1024;

        let test_input_local = get_input_local_signer_block(seed);
        let local_signature = test_input_local.sign();

        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);
        let test_input = get_input_data_block(seed);

        let remote_signature = do_sign_request(&test_client, test_input);

        assert_eq!(local_signature, remote_signature.unwrap());
    }

    #[test]
    fn happy_path() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);
        let test_input = get_input_data_block(0xc137);

        let signature = do_sign_request(&test_client, test_input);

        assert_eq!(signature.unwrap(), HAPPY_PATH_BLOCK_SIGNATURE_C137);

        test_signer.shutdown();
    }
}

// # Test Strategy (TODO)
//
// ## JSON serialization
// * block
// * attestation
// * None when is randao. (Let's enforce it before the serialization as well.)
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

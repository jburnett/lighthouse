mod post {
    use remote_signer::Error;
    use remote_signer_test::*;
    use server_helpers::*;

    #[test]
    fn server_unavailable() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);

        test_signer.shutdown();

        let test_input = get_input_data_block(0xc137);
        let signature = do_sign_request(&test_client, test_input);

        match signature.unwrap_err() {
            Error::Reqwest(e) => {
                let error_msg = e.to_string();
                assert!(error_msg.contains("error sending request for url"));
                assert!(error_msg.contains(PUBLIC_KEY_1));
                assert!(error_msg.contains("error trying to connect"));
                assert!(error_msg.contains("tcp connect error"));
                assert!(error_msg.contains("Connection refused"));
            }
            e => panic!("{:?}", e),
        }
    }

    #[test]
    fn server_error() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);

        test_signer.shutdown();

        let test_input = get_input_data_block(0xc137);
        let signature = do_sign_request(&test_client, test_input);

        match signature.unwrap_err() {
            Error::Reqwest(e) => {
                let error_msg = e.to_string();
                assert!(error_msg.contains("error sending request for url"));
                assert!(error_msg.contains(PUBLIC_KEY_1));
                assert!(error_msg.contains("error trying to connect"));
                assert!(error_msg.contains("tcp connect error"));
                assert!(error_msg.contains("Connection refused"));
            }
            e => panic!("{:?}", e),
        }
    }
}

// # Test Strategy (TODO)
//
// ## POST
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
// ## Stuff to do with a mock
// * We can evaluate the json payload we are sending
// * We can instruct the mock to give lousy response, to test our resilience.
//   * In particular, how we handle errors at the end of the `sign()` function.

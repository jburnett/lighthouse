mod sign_randao {
    use remote_signer_test::*;
    use server_helpers::*;

    #[test]
    fn happy_path() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();
        let test_client = set_up_test_client(&test_signer.address);
        let test_input = get_input_data_randao();

        // TODO
        // Test that the data field is actually empty!
        // i.e. that this DummyRandao hack (only for tests) works.

        let signature = do_sign_request(&test_client, test_input);

        assert_eq!(signature.unwrap(), HAPPY_PATH_RANDAO_SIGNATURE);

        test_signer.shutdown();
    }
}

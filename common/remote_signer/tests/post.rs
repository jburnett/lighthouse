mod post {
    use remote_signer::{Error, RemoteSignerHttpClient};
    use remote_signer_test::*;
    use reqwest::{ClientBuilder, Url};
    use server_helpers::*;
    use tokio::time::Duration;

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
        let (test_signer, tmp_dir) = set_up_api_test_signer_to_sign_message();
        set_permissions(tmp_dir.path(), 0o40311);
        set_permissions(&tmp_dir.path().join(PUBLIC_KEY_1), 0o40311);

        let test_client = set_up_test_client(&test_signer.address);
        let test_input = get_input_data_block(0xc137);
        let signature = do_sign_request(&test_client, test_input);

        set_permissions(tmp_dir.path(), 0o40755);
        set_permissions(&tmp_dir.path().join(PUBLIC_KEY_1), 0o40755);

        match signature.unwrap_err() {
            Error::ServerMessage(message) => assert_eq!(message, "Storage error: PermissionDenied"),
            e => panic!("{:?}", e),
        }

        test_signer.shutdown();
    }

    #[test]
    fn invalid_url() {
        let (test_signer, _tmp_dir) = set_up_api_test_signer_to_sign_message();

        let run_testcase = |u: &str| -> Result<String, String> {
            // TODO
            // Sacale el tipo de error aca, se ve un poco mas decente asi
            let url: Url = u.parse().map_err(|e| format!("[ParseError] {:?}", e))?;

            let reqwest_client = ClientBuilder::new()
                .timeout(Duration::from_secs(12))
                .build()
                .unwrap();

            let test_client = RemoteSignerHttpClient::from_components(url, reqwest_client);

            let test_input = get_input_data_block(0xc137);
            let signature = do_sign_request(&test_client, test_input);

            signature.map_err(|e| match e {
                Error::InvalidUrl(message) => format!("[InvalidUrl] {:?}", message),
                Error::Reqwest(re) => {
                    if re.is_builder() {
                        format!("[Reqwest - Builder] {:?}", re.url().unwrap())
                    } else if re.is_request() {
                        format!("[Reqwest - Request] {:?}", re.url().unwrap())
                    } else {
                        format!("[Reqwest] {:?}", re)
                    }
                }
                _ => format!("[HERMAN] {:?}", e),
            })
        };

        let testcase = |u: &str, msg: &str| assert_eq!(run_testcase(u).unwrap_err(), msg);

        // url::parser::ParseError.
        // They don't even make it to build a RemoteSignerHttpClient.
        testcase("", "[ParseError] RelativeUrlWithoutBase");
        testcase("/4/8/15/16/23/42", "[ParseError] RelativeUrlWithoutBase");
        testcase("localhost", "[ParseError] RelativeUrlWithoutBase");
        testcase(":", "[ParseError] RelativeUrlWithoutBase");
        testcase("0.0:0", "[ParseError] RelativeUrlWithoutBase");
        testcase(":aa", "[ParseError] RelativeUrlWithoutBase");
        testcase("0:", "[ParseError] RelativeUrlWithoutBase");
        testcase("ftp://", "[ParseError] EmptyHost");
        testcase("http://", "[ParseError] EmptyHost");
        testcase("http://127.0.0.1:abcd", "[ParseError] InvalidPort");
        testcase("http://280.0.0.1", "[ParseError] InvalidIpv4Address");

        // `Error::InvalidUrl`.
        // The RemoteSignerHttpClient is created, but fails at `path_segments_mut()`.
        testcase("localhost:abcd", "[InvalidUrl] \"localhost:abcd\"");
        testcase("localhost:", "[InvalidUrl] \"localhost:\"");

        // `Reqwest::Error` of the `Builder` kind.
        // POST is not made.
        testcase(
            "unix:/run/foo.socket",
            &format!(
                "[Reqwest - Builder] \"unix:/run/foo.socket/sign/{}\"",
                PUBLIC_KEY_1
            ),
        );
        // `Reqwest::Error` of the `Request` kind.
        testcase(
            "http://127.0.0.1:0",
            &format!(
                "[Reqwest - Request] \"http://127.0.0.1:0/sign/{}\"",
                PUBLIC_KEY_1
            ),
        );

        test_signer.shutdown();
    }
}

// # Test Strategy (TODO)
//
// ## POST
// * Bad URL (to get 404s)
// * secret key problems
//   * key_not_found
//   * invalid_secret_key
//   * key mismatch
// * Weird status code (418)
// * Timeout
//
// ## MOCK
// * Bad response (use mock)
//   * no json
//   * missing_signing_root_in_json
//   * json, empty_signing_root_in_json#
//   * invalid_string_signing_root
//   * json, signature, but extra fields (should be ignored?)
// * We can evaluate the json payload we are sending
// * We can instruct the mock to give lousy response, to test our resilience.
//   * In particular, how we handle errors at the end of the `sign()` function.

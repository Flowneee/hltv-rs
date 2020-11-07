use std::error::Error;

/// HTTPS capable synchronous client.
pub trait HttpsClient {
    fn get(&self, url: &str) -> Result<String, Box<dyn Error>>;
}

#[test]
fn assert_https_client_object_safety() {
    struct NoneHttpsClient;

    impl HttpsClient for NoneHttpsClient {
        fn get(&self, _url: &str) -> Result<String, Box<dyn Error>> {
            Ok(String::new())
        }
    }

    let _: Box<dyn HttpsClient> = Box::new(NoneHttpsClient);
}

pub mod impls {
    use super::*;

    #[cfg(feature = "attohttpc_client")]
    pub mod attohttpc_impl {
        use super::*;

        /// `HttpsClient` implementation for `attohttpc` crate.
        pub struct AttoHttpcImpl {}

        impl HttpsClient for AttoHttpcImpl {
            fn get(&self, url: &str) -> Result<String, Box<dyn Error>> {
                Ok(attohttpc::get(url).send()?.error_for_status()?.text()?)
            }
        }

        #[test]
        fn get() {
            assert!(AttoHttpcImpl {}.get("http://example.com").is_ok())
        }

        #[test]
        fn get_ssl() {
            assert!(AttoHttpcImpl {}.get("https://example.com").is_ok())
        }

        #[test]
        fn get_err() {
            assert!(AttoHttpcImpl {}.get("http://example.com/unknown").is_err())
        }

        #[test]
        fn get_ssl_err() {
            assert!(AttoHttpcImpl {}.get("https://example.com/unknown").is_err())
        }
    }
}

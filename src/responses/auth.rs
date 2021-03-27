/// A response providing an access token from /api/v1/access_token which can be used for the
/// OAuth-based authenticators
pub use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TokenResponseData {
    pub access_token: String,
    pub expires_in: u64,
    pub scope: String,
    pub token_type: String,
}

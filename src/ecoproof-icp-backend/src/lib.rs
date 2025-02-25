use ic_cdk::api;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HttpRequest {
    pub url: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// A query method to perform an outbound HTTPS GET call.
#[ic_cdk_macros::query]
async fn fetch_https(url: String) -> Result<String, String> {
    // Build the HTTP request structure.
    let request = HttpRequest {
        url: url.clone(),
        method: "GET".to_string(),
        headers: vec![],
        body: vec![],
    };

    // Call the management canister's "http_request" method.
    let result: Result<HttpResponse, (i32, String)> =
        api::call::call("rrkah-fqaaa-aaaaa-aaaaq-cai", "http_request", (request,))
            .await
            .map_err(|(code, msg)| format!("Call error {}: {}", code, msg))?;

    // Check the response and convert the body from bytes to String.
    if result.status == 200 {
        String::from_utf8(result.body).map_err(|e| format!("UTF8 conversion error: {}", e))
    } else {
        Err(format!("HTTP request failed with status: {}", result.status))
    }
}

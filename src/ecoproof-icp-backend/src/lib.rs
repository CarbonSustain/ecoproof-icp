use ic_cdk::api::management_canister::http_request::{
    HttpMethod, CanisterHttpRequestArgument
};
use num_traits::cast::ToPrimitive;
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;
use ic_cdk_macros::{init, update};

/// Initialization function that sets a timer to call the API every 30 seconds.
#[init]
fn init() {
    ic_cdk::println!("Canister initialized. Starting timer to call API every 30 seconds.");
    // Set a timer interval for every 30 seconds.
    set_timer_interval(Duration::from_secs(300), || {
        // Spawn an asynchronous task for the HTTP request.
        ic_cdk::spawn(async {
            // TODO: need to update API for our real API !
            let url = "https://api.exchange.coinbase.com/products/ICP-USD/ticker".to_string();
            match fetch_https(url).await {
                Ok(response) => ic_cdk::println!("API response: {}", response),
                Err(error) => ic_cdk::println!("API request error: {}", error),
            }
        });
    });
}

/// Update function that performs an HTTP GET request to the specified URL.
#[update]
async fn fetch_https(url: String) -> Result<String, String> {
    // Create the HTTP request argument.
    let request = CanisterHttpRequestArgument {
        url: url.clone(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: vec![],
    };

    // Make the HTTP request using the IC management canister.
    match ic_cdk::api::management_canister::http_request::http_request(
        request,
        1_000_000_000_000, // 1T cycles
    )
    .await
    {
        Ok((response,)) => {
            // Convert the status code to a u64.
            let status_code = match response.status.0.to_u64() {
                Some(code) => code,
                None => 0,
            };

            // If the status code indicates success, decode the response body as UTF-8.
            if status_code >= 200 && status_code < 300 {
                String::from_utf8(response.body)
                    .map_err(|e| format!("Failed to decode response as UTF-8: {}", e))
            } else {
                Err(format!("HTTP request failed with status code: {}", status_code))
            }
        }
        Err((code, msg)) => Err(format!(
            "HTTP request failed with code {:?} and message: {}",
            code, msg
        )),
    }
}

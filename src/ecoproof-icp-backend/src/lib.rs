use ic_cdk::api::management_canister::http_request::{
    HttpMethod, CanisterHttpRequestArgument
};
use ic_cdk_macros::update;
use num_traits::cast::ToPrimitive;

/// `fetch_https` function sends an HTTP GET request to the given URL and returns the response.
#[update]
async fn fetch_https(url: String) -> Result<String, String> {
    // HTTP 요청 구성
    let request = CanisterHttpRequestArgument {
        url: url.clone(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: vec![],
    };

    // Call the IC management canister for HTTP request
    match ic_cdk::api::management_canister::http_request::http_request(
        request,
        1_000_000_000_000, // 1T cycles
    )
    .await
    {
        Ok((response,)) => {
            // BigUint를 u64로 변환하여 비교
            let status_code = match response.status.0.to_u64() {
                Some(code) => code,
                None => 0,
            };
            
            if status_code >= 200 && status_code < 300 {
                String::from_utf8(response.body)
                    .map_err(|e| format!("Failed to decode response as UTF-8: {}", e))
            } else {
                Err(format!(
                    "HTTP request failed with status code: {}",
                    status_code
                ))
            }
        }
        Err((code, msg)) => Err(format!(
            "HTTP request failed with code {:?} and message: {}",
            code, msg
        )),
    }
}

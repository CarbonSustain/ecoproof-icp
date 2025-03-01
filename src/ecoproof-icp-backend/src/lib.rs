use ic_cdk::api::management_canister::http_request::{
    HttpMethod, CanisterHttpRequestArgument
};
use ic_cdk_timers::set_timer_interval;
use std::time::Duration;
use ic_cdk_macros::{init, update, query};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use candid::CandidType; 
use ic_cdk::api::management_canister::http_request::HttpHeader;
use urlencoding::encode;
use base64;

/// Initialization function that sets a timer to call the API every 30 seconds.
const TARGET_LAT: f64 = 39.7791;
const TARGET_LON: f64 = -104.9707;
const WEATHER_KEY: &str = "8b3374f9895f6d1899a8c7cc203dd33d";
const NOA_KEY: &str = "cBMiwbVuHAjtvHaqmEgoAklxThMqwRzL";
const USERNAME: &str = "carbonsustaininc_bryzek_paul";
const PASSWORD: &str = "60L83TvqsO";
const START_DATE: &str = "2025-02-23";
const END_DATE: &str = "2025-03-01";
const DATASET_ID: &str = "GHCND";
const DATATYPE_ID: &str = "TAVG";
const UNITS: &str = "metric";
const LIMIT: &str = "10";
const EXTENT: &str = "40,-105,39,-104"; // Correct order: north, south, east, west

// Thread-local storage for weather data
thread_local! {
    static WEATHER_DATA: RefCell<HashMap<String, WeatherStorage>> = RefCell::new(HashMap::new());
    static LOCATION_STORE: RefCell<HashMap<String, LocationData>> = RefCell::new(HashMap::new());
}

#[derive(Clone, Serialize, Deserialize, Default, Debug)]
struct WeatherStorage {
    openweather: Vec<Value>, // Store raw JSON as serde_json::Value
    noaa: Vec<Value>,
    meteomatics: Vec<Value>,
}

#[derive(Clone, Serialize, Deserialize, CandidType, Default, Debug)]
struct LocationData {
    lat_long: Vec<(f64, f64)>,
}

#[init]
fn init() {
    ic_cdk::println!("Canister initialized. Starting timer to call API every 30 seconds.");
    // Set a timer interval for every 30 seconds.
    set_timer_interval(Duration::from_secs(300), || {
        // Spawn an asynchronous task for the HTTP request.
        ic_cdk::spawn(async {
            let url = format!(
                "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&units=metric",
                TARGET_LAT, TARGET_LON, WEATHER_KEY //later Pass the API Key at Deployment
            );
            match save_weather_data(TARGET_LAT, TARGET_LON).await {
                Ok(response) => ic_cdk::println!("OpenWeather API response: {}", response),
                Err(error) => ic_cdk::println!("OpenWeather API request error: {}", error),
            }
            match save_noaa_data(TARGET_LAT, TARGET_LON).await { // Fix typo here
                Ok(response) => ic_cdk::println!("NOAA API response: {}", response),
                Err(error) => ic_cdk::println!("NOAA API request error: {}", error),
            }
            match save_meteomatics_data(TARGET_LAT, TARGET_LON).await {
                Ok(response) => ic_cdk::println!("Meteomatics API response: {}", response),
                Err(error) => ic_cdk::println!("Meteomatics API request error: {}", error),
            }
        });
    });
}

//**** OPEN_WEATHER API ****//

#[update]
async fn save_weather_data(lat: f64, lon: f64) -> Result<String, String> {
    let url = format!(
        "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&appid={}&units=metric",
        lat, lon, WEATHER_KEY
    );

    let request = CanisterHttpRequestArgument {
        url: url.clone(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: vec![],
    };

    match ic_cdk::api::management_canister::http_request::http_request(request, 1_000_000_000_000).await {
        Ok((response,)) => {
            let body = String::from_utf8(response.body).map_err(|e| format!("Failed to decode response: {}", e))?;
            let json: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
            let location_key = format!("{},{}", lat, lon);

            WEATHER_DATA.with(|store| {
                let mut data = store.borrow_mut();
                let entry = data.entry(location_key.clone()).or_insert_with(WeatherStorage::default);
                entry.openweather.push(json);
            });

            ic_cdk::println!("OpenWeather data saved.");
            Ok("Weather data saved".to_string())
        }
        Err((code, msg)) => Err(format!("HTTP request failed: {:?} - {}", code, msg)),
    }
}

#[query]
fn get_weather_data(lat: f64, lon: f64) -> Option<String> {
    let location_key = format!("{},{}", lat, lon);
    WEATHER_DATA.with(|store| {
        store.borrow().get(&location_key).map(|data| serde_json::to_string(data).unwrap_or_default())
    })
}

//**** NOA API ****//
#[update]
async fn save_noaa_data(lat: f64, lon: f64) -> Result<String, String> {
    let noaa_url = "https://www.ncei.noaa.gov/cdo-web/api/v2/data?datasetid=GHCND&datatypeid=CO2&datatypeid=TAVG&startdate=2025-02-23&enddate=2025-03-01&units=metric&limit=100&extent=-122.5%2C37.5%2C-122.0%2C38.0&includemetadata=false";//"https://www.ncei.noaa.gov/cdo-web/api/v2/data";
    let params = format!(
        "?datasetid={}&datatypeid={}&datatypeid={}&startdate={}&enddate={}&units={}&limit={}&extent={}&includemetadata=false",
        encode(DATASET_ID).to_string(), encode("CO2").to_string(), encode(DATATYPE_ID).to_string(),
        encode(START_DATE), encode(END_DATE), encode(UNITS), encode(LIMIT), encode(EXTENT)
    );

    let full_url = format!("{}{}", noaa_url, params);
    ic_cdk::println!("NOAA Request URL: {}", full_url);


    let request = CanisterHttpRequestArgument {
        url: noaa_url.to_string(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: vec![HttpHeader {
            name: "token".to_string(),
            value: NOA_KEY.to_string(),
        }],
        
    };

    match ic_cdk::api::management_canister::http_request::http_request(request, 1_000_000_000_000)
        .await
    {
        Ok((response,)) => {
            let body = String::from_utf8(response.body).map_err(|e| format!("Failed to decode NOAA response: {}", e))?;
            let json: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
            let location_key = format!("{},{}", lat, lon);
            ic_cdk::println!("Raw NOAA API Response: {}", body);


            WEATHER_DATA.with(|store| {
                let mut data = store.borrow_mut();
                let entry = data.entry(location_key.clone()).or_insert_with(WeatherStorage::default);
                entry.noaa.push(json);
            });

            ic_cdk::println!("NOAA data saved.");
            Ok("NOAA data saved".to_string())
        }
        Err((code, msg)) => Err(format!("NOAA request failed: {:?} - {}", code, msg)),
    }
}

#[query]
fn get_noaa_data(lat: f64, lon: f64) -> Option<String> {
    let location_key = format!("{},{}", lat, lon);
    WEATHER_DATA.with(|store| {
        store.borrow().get(&location_key)
            .map(|data| serde_json::to_string(&data.noaa).unwrap_or_default())
    })
}

//**** METEOMATICS API ****//
#[update]
async fn save_meteomatics_data(lat: f64, lon: f64) -> Result<String, String> {
    let valid_datetime = "2025-03-05T12:00:00Z";  // Match Python's datetime format
    let parameters = "t_2m:C,precip_1h:mm";       // Match Python's parameters
    let response_format = "json";                 // Match Python's format
    let locations = format!("{},{}", lat, lon);   // Match Python's location format

    // Construct API URL similar to Python
    let url = format!(
        "https://api.meteomatics.com/{}/{}/{}/{}",
        encode(valid_datetime),
        encode(parameters),
        encode(&locations),
        encode(response_format)
    );

    ic_cdk::println!("Meteomatics Request URL: {}", url); // Debug URL before making request

    // Encode authentication credentials
    let auth_header = format!(
        "Basic {}",
        base64::encode(format!("{}:{}", USERNAME, PASSWORD))
    );

    // Prepare HTTP request
    let request = CanisterHttpRequestArgument {
        url: url.clone(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: None,
        transform: None,
        headers: vec![HttpHeader {
            name: "Authorization".to_string(),
            value: auth_header,
        }],
    };

    // Send request
    match ic_cdk::api::management_canister::http_request::http_request(request, 1_000_000_000_000)
        .await
    {
        Ok((response,)) => {
            let body = String::from_utf8(response.body).map_err(|e| format!("Failed to decode Meteomatics response: {}", e))?;
            let json: Value = serde_json::from_str(&body).unwrap_or(Value::Null);
            let location_key = format!("{},{}", lat, lon);

            ic_cdk::println!("Raw Meteomatics API Response: {}", body);

            WEATHER_DATA.with(|store| {
                let mut data = store.borrow_mut();
                let entry = data.entry(location_key.clone()).or_insert_with(WeatherStorage::default);
                entry.meteomatics.push(json);
            });

            ic_cdk::println!("Meteomatics data saved.");
            Ok("Meteomatics data saved".to_string())
        }
        Err((code, msg)) => Err(format!("Meteomatics request failed: {:?} - {}", code, msg)),
    }
}


#[query]
fn get_meteomatics_data(lat: f64, lon: f64) -> Option<String> {
    let location_key = format!("{},{}", lat, lon);
    WEATHER_DATA.with(|store| {
        store.borrow().get(&location_key)
            .map(|data| serde_json::to_string(&data.meteomatics).unwrap_or_default())
    })
}


//****** Location Data Functions ******//
#[update]
fn add_location(user_id: String, lat_long: Vec<(f64, f64)>) {
    LOCATION_STORE.with(|store| {
        let mut data = store.borrow_mut();
        let entry = data.entry(user_id.clone()).or_insert_with(LocationData::default);
        entry.lat_long.extend(lat_long);
    });

    ic_cdk::println!("Added locations for user: {}", user_id);
}

#[query]
fn get_location(user_id: String) -> Option<LocationData> {
    LOCATION_STORE.with(|store| store.borrow().get(&user_id).cloned())
}

#[query]
fn get_all_locations() -> HashMap<String, LocationData> {
    LOCATION_STORE.with(|store| store.borrow().clone())
}




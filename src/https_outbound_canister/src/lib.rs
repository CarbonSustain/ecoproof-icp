use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs,
    TransformContext,
};
use ic_cdk_macros::{init, query, update};
use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::cell::RefCell;

// Store the API key in thread-local storage
thread_local! {
    static API_KEY: RefCell<Option<String>> = RefCell::new(None);
}

// OpenWeather API response structures
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct WeatherMain {
    pub temp: f64,
    pub feels_like: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub pressure: f64,
    pub humidity: f64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct WeatherCondition {
    pub id: u32,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct WeatherWind {
    pub speed: f64,
    pub deg: f64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct WeatherClouds {
    pub all: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct WeatherSys {
    pub country: String,
    pub sunrise: u64,
    pub sunset: u64,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct OpenWeatherResponse {
    pub coord: HashMap<String, f64>,
    pub weather: Vec<WeatherCondition>,
    pub base: String,
    pub main: WeatherMain,
    pub visibility: u32,
    pub wind: WeatherWind,
    pub clouds: WeatherClouds,
    pub dt: u64,
    pub sys: WeatherSys,
    pub timezone: i32,
    pub id: u64,
    pub name: String,
    pub cod: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct WeatherData {
    pub city: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub temperature: f64,
    pub feels_like: f64,
    pub temp_min: f64,
    pub temp_max: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub wind_speed: f64,
    pub wind_direction: f64,
    pub visibility: u32,
    pub cloudiness: u32,
    pub weather_condition: String,
    pub weather_description: String,
    pub timestamp: u64,
    pub sunrise: u64,
    pub sunset: u64,
}

#[derive(CandidType, Serialize, Deserialize)]
pub struct InitArgs {
    pub openweather_api_key: String,
}

#[init]
fn init(args: Option<InitArgs>) {
    ic_cdk::println!("HTTPS Outbound Canister initialized");
    
    // Initialize with the API key from deployment arguments
    if let Some(init_args) = args {
        API_KEY.with(|key| {
            *key.borrow_mut() = Some(init_args.openweather_api_key);
        });
        ic_cdk::println!("API key configured from initialization arguments");
    } else {
        ic_cdk::println!("Warning: No API key provided during initialization");
    }
}

// Function to set or update the API key (admin only in production)
#[update]
fn set_api_key(api_key: String) -> Result<String, String> {
    if api_key.is_empty() {
        return Err("API key cannot be empty".to_string());
    }
    
    API_KEY.with(|key| {
        *key.borrow_mut() = Some(api_key);
    });
    
    Ok("API key updated successfully".to_string())
}

fn get_stored_api_key() -> Result<String, String> {
    API_KEY.with(|key| {
        match key.borrow().as_ref() {
            Some(api_key) => Ok(api_key.clone()),
            None => Err("Weather API key not configured.".to_string()),
        }
    })
}

#[update]
async fn fetch_weather_data(lat: f64, lon: f64) -> Result<WeatherData, String> {
    let api_key = get_stored_api_key()?;
    
    // Construct the OpenWeatherMap API URL
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=metric",
        lat, lon, api_key
    );

    ic_cdk::println!("Fetching weather data from: {}", url);

    // Prepare HTTP request
    let request_headers = vec![
        HttpHeader {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "Internet Computer Weather Agent".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: url.clone(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(2048),
        transform: Some(TransformContext::from_name("transform".to_string(), vec![])),
        headers: request_headers,
    };

    // Make the HTTP request with cycles
    let cycles = 230_949_972_000u128; // Cycles for HTTP outcalls
    match http_request(request, cycles).await {
        Ok((response,)) => {
            ic_cdk::println!("HTTP Response status: {}", response.status);
            
            if response.status == 200u16 {
                // Parse the response body
                let response_str = String::from_utf8(response.body)
                    .map_err(|e| format!("Failed to parse response as UTF-8: {}", e))?;
                
                ic_cdk::println!("Response body: {}", response_str);

                // Parse JSON response
                let weather_response: OpenWeatherResponse = serde_json::from_str(&response_str)
                    .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

                // Extract weather data
                let weather_data = WeatherData {
                    city: weather_response.name,
                    country: weather_response.sys.country,
                    latitude: weather_response.coord.get("lat").copied().unwrap_or(lat),
                    longitude: weather_response.coord.get("lon").copied().unwrap_or(lon),
                    temperature: weather_response.main.temp,
                    feels_like: weather_response.main.feels_like,
                    temp_min: weather_response.main.temp_min,
                    temp_max: weather_response.main.temp_max,
                    humidity: weather_response.main.humidity,
                    pressure: weather_response.main.pressure,
                    wind_speed: weather_response.wind.speed,
                    wind_direction: weather_response.wind.deg,
                    visibility: weather_response.visibility,
                    cloudiness: weather_response.clouds.all,
                    weather_condition: weather_response.weather.first()
                        .map(|w| w.main.clone())
                        .unwrap_or_default(),
                    weather_description: weather_response.weather.first()
                        .map(|w| w.description.clone())
                        .unwrap_or_default(),
                    timestamp: weather_response.dt,
                    sunrise: weather_response.sys.sunrise,
                    sunset: weather_response.sys.sunset,
                };

                Ok(weather_data)
            } else {
                Err(format!("HTTP request failed with status: {}", response.status))
            }
        }
        Err((r, m)) => {
            let message = format!("HTTP request failed: RejectionCode: {:?}, Error: {}", r, m);
            ic_cdk::println!("{}", message);
            Err(message)
        }
    }
}

#[query]
fn transform(raw: TransformArgs) -> HttpResponse {
    let headers = vec![
        HttpHeader {
            name: "Content-Security-Policy".to_string(),
            value: "default-src 'self'".to_string(),
        },
        HttpHeader {
            name: "Referrer-Policy".to_string(),
            value: "strict-origin".to_string(),
        },
        HttpHeader {
            name: "Permissions-Policy".to_string(),
            value: "geolocation=(self)".to_string(),
        },
        HttpHeader {
            name: "Strict-Transport-Security".to_string(),
            value: "max-age=63072000".to_string(),
        },
        HttpHeader {
            name: "X-Frame-Options".to_string(),
            value: "DENY".to_string(),
        },
        HttpHeader {
            name: "X-Content-Type-Options".to_string(),
            value: "nosniff".to_string(),
        },
    ];

    let mut res = HttpResponse {
        status: raw.response.status.clone(),
        body: raw.response.body.clone(),
        headers,
    };

    if res.status == 200u16 {
        res.body = raw.response.body;
    } else {
        ic_cdk::api::print(format!("Received an error from OpenWeather API: err = {:?}", raw));
    }
    res
}

#[update]
async fn fetch_weather_by_city(city_name: String) -> Result<WeatherData, String> {
    let api_key = get_stored_api_key()?;
    
    // Construct the OpenWeatherMap API URL for city lookup
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city_name, api_key
    );

    ic_cdk::println!("Fetching weather data for city: {}", city_name);

    // Prepare HTTP request
    let request_headers = vec![
        HttpHeader {
            name: "Accept".to_string(),
            value: "application/json".to_string(),
        },
        HttpHeader {
            name: "User-Agent".to_string(),
            value: "Internet Computer Weather Agent".to_string(),
        },
    ];

    let request = CanisterHttpRequestArgument {
        url: url.clone(),
        method: HttpMethod::GET,
        body: None,
        max_response_bytes: Some(2048),
        transform: Some(TransformContext::from_name("transform".to_string(), vec![])),
        headers: request_headers,
    };

    // Make the HTTP request with cycles
    let cycles = 230_949_972_000u128; // Cycles for HTTP outcalls
    match http_request(request, cycles).await {
        Ok((response,)) => {
            if response.status == 200u16 {
                let response_str = String::from_utf8(response.body)
                    .map_err(|e| format!("Failed to parse response as UTF-8: {}", e))?;

                let weather_response: OpenWeatherResponse = serde_json::from_str(&response_str)
                    .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

                let weather_data = WeatherData {
                    city: weather_response.name,
                    country: weather_response.sys.country,
                    latitude: weather_response.coord.get("lat").copied().unwrap_or(0.0),
                    longitude: weather_response.coord.get("lon").copied().unwrap_or(0.0),
                    temperature: weather_response.main.temp,
                    feels_like: weather_response.main.feels_like,
                    temp_min: weather_response.main.temp_min,
                    temp_max: weather_response.main.temp_max,
                    humidity: weather_response.main.humidity,
                    pressure: weather_response.main.pressure,
                    wind_speed: weather_response.wind.speed,
                    wind_direction: weather_response.wind.deg,
                    visibility: weather_response.visibility,
                    cloudiness: weather_response.clouds.all,
                    weather_condition: weather_response.weather.first()
                        .map(|w| w.main.clone())
                        .unwrap_or_default(),
                    weather_description: weather_response.weather.first()
                        .map(|w| w.description.clone())
                        .unwrap_or_default(),
                    timestamp: weather_response.dt,
                    sunrise: weather_response.sys.sunrise,
                    sunset: weather_response.sys.sunset,
                };

                Ok(weather_data)
            } else {
                Err(format!("HTTP request failed with status: {}", response.status))
            }
        }
        Err((r, m)) => {
            Err(format!("HTTP request failed: RejectionCode: {:?}, Error: {}", r, m))
        }
    }
}

#[query]
fn get_api_info() -> String {
    "HTTPS Outbound Canister for OpenWeatherMap API integration. Functions: fetch_weather_data(lat, lon), fetch_weather_by_city(city_name)".to_string()
}

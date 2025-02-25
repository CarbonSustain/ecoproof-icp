use ic_cdk::api::time;
use ic_cdk::export::candid::{CandidType, Deserialize};
use ic_cdk::storage;
use std::collections::{HashMap, HashSet};

type UserId = String;
type VoteId = u64;

#[derive(CandidType, Deserialize, Clone)]
struct WeatherData {
    location: String,
    temperature: f64,
    humidity: f64,
    timestamp: u64,
}

#[derive(CandidType, Deserialize, Clone)]
struct Vote {
    user: UserId,
    data_id: u64,
    vote_value: bool, // true = valid, false = invalid
}

#[derive(CandidType, Deserialize, Clone)]
struct UserSubmission {
    user: UserId,
    data: WeatherData,
    votes: Vec<Vote>,
    rewarded: bool,
}

#[derive(CandidType, Deserialize, Clone, Default)]
struct TokenBalance {
    balance: u64,
}

#[ic_cdk::update]
fn submit_weather_data(user: String, location: String, temperature: f64, humidity: f64) -> u64 {
    let timestamp = ic_cdk::api::time();
    let data_id = storage::get_mut::<HashMap<u64, UserSubmission>>().len() as u64 + 1;

    let new_data = UserSubmission {
        user: user.clone(),
        data: WeatherData {
            location,
            temperature,
            humidity,
            timestamp,
        },
        votes: vec![],
        rewarded: false,
    };

    storage::get_mut::<HashMap<u64, UserSubmission>>().insert(data_id, new_data);
    data_id
}

#[ic_cdk::update]
fn vote_on_data(user: UserId, data_id: u64, vote_value: bool) -> Result<String, String> {
    let submissions = storage::get_mut::<HashMap<u64, UserSubmission>>();

    if let Some(submission) = submissions.get_mut(&data_id) {
        if submission.votes.iter().any(|v| v.user == user) {
            return Err("User has already voted on this submission.".to_string());
        }

        submission.votes.push(Vote {
            user: user.clone(),
            data_id,
            vote_value,
        });

        Ok("Vote registered successfully.".to_string())
    } else {
        Err("Weather data submission not found.".to_string())
    }
}

#[ic_cdk::update]
fn reward_user(data_id: u64) -> Result<String, String> {
    let submissions = storage::get_mut::<HashMap<u64, UserSubmission>>();

    if let Some(submission) = submissions.get_mut(&data_id) {
        if submission.rewarded {
            return Err("User has already been rewarded for this submission.".to_string());
        }

        let valid_votes = submission.votes.iter().filter(|v| v.vote_value).count();
        let invalid_votes = submission.votes.len() - valid_votes;

        if valid_votes > invalid_votes {
            let balances = storage::get_mut::<HashMap<UserId, TokenBalance>>();
            let entry = balances.entry(submission.user.clone()).or_default();
            entry.balance += 10; // Reward user with 10 tokens
            submission.rewarded = true;

            Ok(format!("User {} rewarded with 10 tokens.", submission.user))
        } else {
            Err("Majority voted the data as invalid, no reward given.".to_string())
        }
    } else {
        Err("Weather data submission not found.".to_string())
    }
}

#[ic_cdk::query]
fn get_balance(user: UserId) -> u64 {
    let balances = storage::get::<HashMap<UserId, TokenBalance>>();
    balances.get(&user).map_or(0, |balance| balance.balance)
}

#[ic_cdk::init]
fn init() {
    storage::stable_save((HashMap::<u64, UserSubmission>::new(), HashMap::<UserId, TokenBalance>::new()))
        .expect("Initialization failed");
}

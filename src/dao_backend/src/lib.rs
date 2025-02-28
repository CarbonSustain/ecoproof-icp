use ic_cdk::api::time;
use ic_cdk_macros::{update, query, init, pre_upgrade, post_upgrade};
use candid::{CandidType, Nat};
use candid::Principal;
use candid::candid_method;
use ic_stable_structures::{
    StableBTreeMap,
    DefaultMemoryImpl,
    storable::Bound,
    memory_manager::{MemoryManager, MemoryId, VirtualMemory},
};
use std::borrow::Cow;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

type UserId = String;
type VoteId = u64;

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct WeatherData {
    latitude: f64,       
    longitude: f64,      
    city: String,        
    temperature: f64,    
    weather: String,   
    timestamp: u64,     
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct Vote {
    user: UserId,
    data_id: u64,
    vote_value: bool,
    submission_id: u64,
}

// make another user structure
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
struct User {
    user_id: UserId,
    balance: u64,
}

impl ic_stable_structures::Storable for User {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }
}

impl ic_stable_structures::Storable for Vote {
    const BOUND: Bound = Bound::Unbounded;
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
struct VoteList(std::vec::Vec<Vote>);

impl ic_stable_structures::Storable for VoteList {
    const BOUND: Bound = Bound::Unbounded;
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(&self.0).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        match serde_cbor::from_slice::<Vec<Vote>>(&bytes) {
            Ok(votes) => VoteList(votes), 
            Err(e) => {
                ic_cdk::println!("ERROR: Failed to deserialize VoteList: {:?}", e);
                VoteList(vec![])
            }
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct UserSubmission {
    user: UserId,
    data: WeatherData,
    rewarded: bool,
}

impl ic_stable_structures::Storable for UserSubmission {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        match serde_cbor::from_slice(&bytes) {
            Ok(data) => {
                ic_cdk::println!("DEBUG: Successfully deserialized UserSubmission.");
                data
            }
            Err(e) => {
                ic_cdk::println!("ERROR: Failed to deserialize UserSubmission: {:?}", e);
                ic_cdk::trap("Deserialization of UserSubmission failed, returning default UserSubmission.");
            }
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Default)]
struct TokenBalance {
    balance: u64,
}

impl ic_stable_structures::Storable for TokenBalance {
    const BOUND: Bound = Bound::Unbounded;

    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).unwrap()
    }
}

thread_local! {
    static MEMORY_MANAGER: std::cell::RefCell<MemoryManager<DefaultMemoryImpl>> =
        std::cell::RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static SUBMISSIONS: std::cell::RefCell<StableBTreeMap<u64, UserSubmission, VirtualMemory<DefaultMemoryImpl>>> = 
        std::cell::RefCell::new({
            let memory = MEMORY_MANAGER.with(|m| {
                let memory_id = MemoryId::new(0);
                let mem = m.borrow().get(memory_id);
                ic_cdk::println!("DEBUG: Allocated memory region {:?}", memory_id);
                mem
            });

            StableBTreeMap::init(memory) 
        });

    // static SUBMISSIONS: std::cell::RefCell<StableBTreeMap<u64, UserSubmission, DefaultMemoryImpl>> =
    //     std::cell::RefCell::new(StableBTreeMap::new(DefaultMemoryImpl::default()));

    // static BALANCES: std::cell::RefCell<StableBTreeMap<UserId, TokenBalance, DefaultMemoryImpl>> =
    //     std::cell::RefCell::new(StableBTreeMap::new(DefaultMemoryImpl::default()));
    static USERS: std::cell::RefCell<StableBTreeMap<UserId, User, DefaultMemoryImpl>> =
        std::cell::RefCell::new(StableBTreeMap::new(DefaultMemoryImpl::default()));

    static VOTES: std::cell::RefCell<HashMap<u64, Vec<Vote>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[pre_upgrade]
fn pre_upgrade() {
    let submission_backup: Vec<(u64, UserSubmission)> = SUBMISSIONS.with(|s| {
        s.borrow().iter().map(|(k, v)| (k, v.clone())).collect()
    });

    ic_cdk::storage::stable_save((submission_backup,))
        .expect("Failed to save state before upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    if let Ok((submission_backup, user_backup)) =
        ic_cdk::storage::stable_restore::<(Vec<(u64, UserSubmission)>, Vec<(UserId, User)>)>()
    {
        SUBMISSIONS.with(|s| {
            let mut s = s.borrow_mut();
            for (k, v) in submission_backup {
                s.insert(k, v);
            }
        });

        USERS.with(|u| {
            let mut u = u.borrow_mut();
            for (k, v) in user_backup {
                u.insert(k, v);
            }
        });

        ic_cdk::println!("✅ INFO: Successfully restored submissions and users after upgrade.");
    } else {
        ic_cdk::println!("⚠️ WARNING: Failed to restore submissions or users after upgrade.");
    }
}

#[update]
fn submit_weather_data(user: String, latitude: f64, longitude: f64, city: String, temperature: f64, weather: String) -> u64 {
    let timestamp = time();
    
    // let data_id = SUBMISSIONS.with(|s| s.borrow().len() as u64 + 1);

    let data_id = SUBMISSIONS.with(|s| s.borrow().len() as u64 + 1);

    let new_data = UserSubmission {
        user: user.clone(),
        data: WeatherData {
            latitude,
            longitude,
            city,
            temperature,
            weather,
            timestamp,
        },
        rewarded: false,
    };

    SUBMISSIONS.with(|s| {
        s.borrow_mut().insert(data_id, new_data.clone());
        ic_cdk::println!("SUBMISSIONS len after insert: {}", s.borrow().len());
    });

    ic_cdk::println!("Inserted data #{}: {:?}", data_id, new_data);

    data_id
}

#[query]
#[candid_method(query)]
fn get_all_submissions() -> Vec<UserSubmission> {
    SUBMISSIONS.with(|submissions| {
        let submissions = submissions.borrow();
        let all_data: Vec<UserSubmission> = submissions.iter().map(|(_, v)| v.clone()).collect();
        
        ic_cdk::println!("✅ DEBUG: Returning all submissions, count: {}", all_data.len());
        all_data
    })
}

#[query]
#[candid_method(query)] 
fn get_submission(data_id: u64) -> Result<UserSubmission, String> {
    SUBMISSIONS.with(|submissions| {
        let submissions = submissions.borrow();
        let total_count = submissions.len();
        ic_cdk::println!("DEBUG: Total submissions stored: {}", total_count);

        match submissions.get(&data_id) {
            Some(sub) => {
                ic_cdk::println!("✅ DEBUG: Found submission with data_id: {}", data_id);
                Ok(sub.clone()) 
            }
            None => {
                ic_cdk::println!("❌ ERROR: Submission not found for data_id: {} | Current total: {}", data_id, total_count);
                Err(format!("Submission {} not found. Total records: {}", data_id, total_count))
            }
        }
    })
}

#[query]
fn get_balance(user_id: UserId) -> u64 {
    USERS.with(|users| {
        let users = users.borrow();

        if users.is_empty() {
            ic_cdk::println!("❌ ERROR: USERS map is empty, returning 0 for {}", user_id);
            return 0;
        }

        match users.get(&user_id) {
            Some(user) => {
                ic_cdk::println!("🔍 DEBUG: Final balance check for user {}: {}", user_id, user.balance);
                user.balance
            }
            None => {
                ic_cdk::println!("⚠️ WARNING: No balance found for user {}, returning 0", user_id);
                0
            }
        }
    })
}

#[derive(CandidType, Serialize, Deserialize)]
enum RewardResponse {
    Ok(String),
    Err(String),
}

#[update]
#[candid_method(update)]
fn reward_user(data_id: u64) -> Result<String, String> {
    SUBMISSIONS.with(|submissions| {
        let mut submissions = submissions.borrow_mut();
        let submission = match submissions.get(&data_id) {
            Some(sub) => {
                ic_cdk::println!("✅ DEBUG: Found submission: {:?}", sub);
                sub.clone()
            }
            None => {
                ic_cdk::println!("❌ ERROR: Submission not found for data_id: {}", data_id);
                return Err("Weather data submission not found.".to_string());
            }
        };

        if submission.rewarded {
            ic_cdk::println!("⚠️ WARNING: Submission already rewarded: {:?}", submission);
            return Err("User has already been rewarded for this submission.".to_string());
        }

        let vote_list = VOTES.with(|votes_map| {
            let votes_map = votes_map.borrow();
            votes_map.get(&data_id).cloned().unwrap_or_default()
        });

        if vote_list.is_empty() {
            ic_cdk::println!("⚠️ WARNING: No votes found for submission {}", data_id);
            return Err("No votes found for this submission. Cannot determine validity.".to_string());
        }

        let valid_votes = vote_list.iter().filter(|v| v.vote_value).count();
        let invalid_votes = vote_list.len() - valid_votes;
        ic_cdk::println!(
            "📝 INFO: Submission {} - valid_votes: {}, invalid_votes: {}",
            data_id, valid_votes, invalid_votes
        );

        let user_id = submission.user.clone();

        if valid_votes > invalid_votes {
            USERS.with(|users| {
                let mut users = users.borrow_mut();

                let user = users.get(&user_id).map(|u| u.clone());

                match user {
                    Some(mut existing_user) => {
                        existing_user.balance += 10;
                        users.insert(user_id.clone(), existing_user.clone());
                        ic_cdk::println!(
                            "✅ DEBUG: Updated balance for user {}: {}",
                            user_id, existing_user.balance
                        );
                    }
                    None => {
                        let new_user = User {
                            user_id: user_id.clone(),
                            balance: 10,
                        };
                        users.insert(user_id.clone(), new_user.clone());
                        ic_cdk::println!(
                            "🆕 INFO: Created new user balance for {}: {} tokens",
                            user_id, new_user.balance
                        );
                    }
                }

                match users.get(&user_id) {
                    Some(updated_user) => {
                        ic_cdk::println!(
                            "🔍 DEBUG: Final balance check for {}: {} tokens",
                            user_id, updated_user.balance
                        );
                    }
                    None => {
                        ic_cdk::println!("❌ ERROR: Balance update failed for {}", user_id);
                    }
                }
            });

            let mut updated_submission = submission.clone();
            updated_submission.rewarded = true;
            submissions.insert(data_id, updated_submission);
            ic_cdk::println!("🎉 SUCCESS: User {} rewarded with 10 tokens.", user_id);
            Ok(format!("User {} rewarded with 10 tokens.", user_id))
        } else {
            ic_cdk::println!(
                "❌ ERROR: Submission {} rejected, majority voted as invalid",
                data_id
            );
            Err("Majority voted the data as invalid, no reward given.".to_string())
        }
    })
}

#[update]
#[candid_method(update)]
fn vote_on_data(user_id: String, data_id: u64, vote_value: bool) -> String {
    ic_cdk::println!("DEBUG: vote_on_data called with user_id: {}, data_id: {}, vote_value: {}", user_id, data_id, vote_value);
    let submission_exists = SUBMISSIONS.with(|submissions| {
        let exists = submissions.borrow().get(&data_id).is_some();
        ic_cdk::println!("DEBUG: Submission existence for {}: {}", data_id, exists);
        exists
    });
    if !submission_exists {
        ic_cdk::println!("DEBUG: Submission {} not found", data_id);
        return "Weather data submission not found.".to_string();
    }

    let new_vote = Vote {
        user: user_id.clone(),
        data_id,
        vote_value,
        submission_id: data_id,
    };
    ic_cdk::println!("DEBUG: New vote created: {:?}", new_vote);

    VOTES.with(|votes_map| {
        let mut votes_map = votes_map.borrow_mut();
        
        let entry = votes_map.entry(data_id).or_insert(Vec::new());
        if entry.iter().any(|vote| vote.user == user_id) {
            ic_cdk::println!("DEBUG: User {} already voted on submission {}", user_id, data_id);
            return "User has already voted on this submission.".to_string();
        }

        entry.push(new_vote);
        ic_cdk::println!("DEBUG: Updated vote list for data_id {}: {:?}", data_id, entry);
        
        format!("User {} successfully voted on data {}.", user_id, data_id)
    })
}

#[init]
fn init() {
    ic_cdk::println!("Canister initialized with StableBTreeMap storage.");
}
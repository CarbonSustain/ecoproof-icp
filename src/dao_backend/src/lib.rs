use ic_cdk::api::time;
use ic_cdk_macros::{update, query, init, pre_upgrade, post_upgrade};
use candid::{CandidType, Nat};
use candid::Principal;
use candid::candid_method;
use ic_stable_structures::{
    StableBTreeMap,
    DefaultMemoryImpl,
    storable::{Storable, Bound},
    memory_manager::{MemoryManager, MemoryId, VirtualMemory},
};
use std::borrow::Cow;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use ic_cdk::call; 
use std::cell::RefCell;  

// -------- Type Definitions --------

type UserId = String;
const MAX_CHALLENGE_BYTES: u32 = 512;

// -------- Structs --------

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct WeatherData {
    latitude: f64,
    longitude: f64,
    city: String,
    temperature: f64,
    weather: String,
    timestamp: u64,
    submission_photo_url: String,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct UserSubmission {
    data_id: u64, 
    user: UserId,
    data: WeatherData,
    rewarded: bool,
    status: PostStatus,
    expiration_timestamp: u64,
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

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct Challenge {
    id: u64,
    title: String,
    latitude: f64,
    longitude: f64,
    radius_m: f64,
    expiration: u64,
    picture_url: String,
}

impl Storable for Challenge {
    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_CHALLENGE_BYTES,
        is_fixed_size: false,
    };
    
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(serde_cbor::to_vec(self).expect("Challenge serialization failed"))
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        serde_cbor::from_slice(&bytes).expect("Challenge deserialization failed")
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct Vote {
    user: UserId,
    data_id: u64,
    vote_value: bool,
    submission_id: u64,
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

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct VoteSummary {
    data_id: u64,
    upvotes: u32,
    downvotes: u32,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Role {
    User,
    Admin,
    Moderator,
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, Default)]
struct User {
    user_id: UserId,
    balance: u64,
    first_name: Option<String>,
    last_name: Option<String>,
    username: Option<String>,
    language_code: Option<String>,
    is_bot: bool,
    profile_picture_url: Option<String>,
    wallet_address: Option<String>,
    role: Role,
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

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
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
struct UserSubmissionSummary {
    data_id: u64,
    city: String,
    status: PostStatus,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct SubmissionInfo {
    data_id: u64,
    user_id: UserId,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    profile_picture_url: Option<String>,
    latitude: f64,
    longitude: f64,
    city: String,
    temperature: f64,
    weather: String,
    timestamp: u64,
    submission_photo_url: String,
    rewarded: bool,
    status: PostStatus,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct ExpirationInfo {
    data_id: u64,
    expiration_timestamp: u64,
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

// -------- Enums --------

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq)]
enum PostStatus {
    OPEN,
    PENDING,
    PAID,
    EXPIRED,
}

#[derive(CandidType, Serialize, Deserialize)]
enum RewardResponse {
    Ok(String),
    Err(String),
}

// -------- Ledger Transfer Types --------

#[derive(CandidType, Deserialize)]
pub struct TransferArg {
    pub to: Account,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub from_subaccount: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
}

// -------- Ledger Account / TransferResult / TransferError --------

#[derive(CandidType, Deserialize)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferResult {
    Ok(Nat),
    Err(TransferError),
}

#[derive(CandidType, Deserialize, Debug)]
pub enum TransferError {
    GenericError { message: String, error_code: Nat },
    TemporarilyUnavailable,
    Duplicate { duplicate_of: Nat },
    BadFee { expected_fee: Nat },
    CreatedInFuture { ledger_time: u64 },
    TooOld,
    InsufficientFunds { balance: Nat },
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
struct SubmissionLocationInfo {
    data_id: u64,
    latitude: f64,
    longitude: f64,
    status: PostStatus,
}

// -------- Storage (StableBTreeMap + MemoryManager) --------
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

    static USERS: std::cell::RefCell<StableBTreeMap<UserId, User, VirtualMemory<DefaultMemoryImpl>>> =
        std::cell::RefCell::new({
            let memory = MEMORY_MANAGER.with(|m| {
                let memory_id = MemoryId::new(1);
                m.borrow().get(memory_id)
            });
            StableBTreeMap::init(memory)
        });

    static VOTES: std::cell::RefCell<HashMap<u64, Vec<Vote>>> =
        std::cell::RefCell::new(HashMap::new());

    static CHALLENGES: RefCell<StableBTreeMap<u64, Challenge, VirtualMemory<DefaultMemoryImpl>>> = 
        RefCell::new({
            let memory = MEMORY_MANAGER.with(|m| {
                m.borrow().get(MemoryId::new(2))
            });
            StableBTreeMap::init(memory)
        });
}

// -------- Upgrade Hooks --------
#[pre_upgrade]
fn pre_upgrade() {
    let submission_backup: Vec<(u64, UserSubmission)> = SUBMISSIONS.with(|s| {
        s.borrow().iter().map(|(k, v)| (k, v.clone())).collect()
    });

    let user_backup: Vec<(UserId, User)> = USERS.with(|u| {
        u.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    });    

    ic_cdk::storage::stable_save((submission_backup, user_backup))
        .expect("Failed to save state before upgrade");
}

#[post_upgrade]
fn post_upgrade() {
    if let Ok((submission_backup, user_backup)) =
        ic_cdk::storage::stable_restore::<(Vec<(u64, UserSubmission)>, Vec<(UserId, User)>)>()
    {
        SUBMISSIONS.with(|s| {
            let mut s = s.borrow_mut();
            for (k, mut v) in submission_backup {
                v.data_id = k;
                s.insert(k, v);
            }
        });

        USERS.with(|u| {
            let mut u = u.borrow_mut();
            for (k, v) in user_backup {
                u.insert(k, v);
            }
        });

        ic_cdk::println!("INFO: Successfully restored submissions and users after upgrade.");
    } else {
        ic_cdk::println!("WARNING: Failed to restore submissions or users after upgrade.");
    }
}

// -------- User functions --------
#[update]
#[candid_method(update)]
fn create_tg_user(telegram_id: String, first_name: String, last_name: String, username: String, language_code: String, is_bot: bool, profile_picture_url: String) -> String {
    USERS.with(|users_map| {
        let mut users = users_map.borrow_mut();
        let user_id = telegram_id.clone();

        if let Some(user) = users.get(&user_id) {
            let mut updated_user = user;
            ic_cdk::println!("👤 Existing user found: {}. Updating info.", user_id);
            updated_user.first_name = Some(first_name);
            updated_user.last_name = Some(last_name);
            updated_user.username = Some(username);
            updated_user.language_code = Some(language_code);
            updated_user.is_bot = is_bot;
            updated_user.profile_picture_url = Some(profile_picture_url);
            users.insert(user_id.clone(), updated_user);
            format!("Updated user: {}", user_id)
        } else {
            let new_user = User {
                user_id: user_id.clone(),
                balance: 0,
                first_name: Some(first_name),
                last_name: Some(last_name),
                username: Some(username),
                language_code: Some(language_code),
                is_bot,
                profile_picture_url: Some(profile_picture_url),
                wallet_address: None,
                role: Role::User,
            };
            users.insert(user_id.clone(), new_user);
            ic_cdk::println!("Created new user: {}", user_id);
            format!("Created new user: {}", user_id)
        }
    })
}

#[query]
#[candid_method(query)]
fn get_tg_user(user_id: String) -> Result<User, String> {
    USERS.with(|users| {
        let users = users.borrow();
        match users.get(&user_id) {
            Some(user) => Ok(user.clone()),
            None => Err(format!("User {} not found.", user_id)),
        }
    })
}

#[update]
fn update_wallet_address(user_id: String, wallet_address: String) -> String {
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        match users.get(&user_id) {
            Some(mut user) => {
                user.wallet_address = Some(wallet_address);
                users.insert(user_id.clone(), user);
                format!("Wallet address updated for user: {}", user_id)
            }
            None => format!("User {} not found", user_id),
        }
    })
}

#[query]
#[candid_method(query)]
fn get_all_users() -> Vec<User> {
    USERS.with(|users_map| {
        let users = users_map.borrow();
        let all_users: Vec<User> = users.iter().map(|(_, user)| user.clone()).collect();
        ic_cdk::println!("DEBUG: Returning all users, count: {}", all_users.len());
        all_users
    })
}

#[query]
fn get_balance(user_id: UserId) -> u64 {
    USERS.with(|users| {
        let users = users.borrow();

        if users.is_empty() {
            ic_cdk::println!("ERROR: USERS map is empty, returning 0 for {}", user_id);
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

// -------- Submission functions --------
#[update]
fn submit_weather_data(telegram_id: String, latitude: f64, longitude: f64, city: String, temperature: f64, weather: String, submission_photo_url: String) -> u64 {
    const SECOND: u64 = 1_000_000_000;
    let timestamp = time();
    let expiration_timestamp = timestamp + 900 * SECOND;
    ic_cdk::println!("Received weather submission from {}", telegram_id);
    ic_cdk::println!("Submission time (timestamp): {}", timestamp);
    ic_cdk::println!("Expiration time (timestamp): {}", expiration_timestamp);
    
    let data_id = SUBMISSIONS.with(|s| s.borrow().len() as u64 + 1);

    let new_data = UserSubmission {
        data_id,
        user: telegram_id.clone(),
        data: WeatherData {
            latitude,
            longitude,
            city,
            temperature,
            weather,
            timestamp,
            submission_photo_url,
        },
        rewarded: false,
        status: PostStatus::OPEN,
        expiration_timestamp,
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
        
        ic_cdk::println!("DEBUG: Returning all submissions, count: {}", all_data.len());
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
                ic_cdk::println!("DEBUG: Found submission with data_id: {}", data_id);
                Ok(sub.clone()) 
            }
            None => {
                ic_cdk::println!("ERROR: Submission not found for data_id: {} | Current total: {}", data_id, total_count);
                Err(format!("Submission {} not found. Total records: {}", data_id, total_count))
            }
        }
    })
}

#[query]
#[candid_method(query)]
fn get_user_posts(user_id: String) -> Vec<UserSubmission> {
    SUBMISSIONS.with(|submissions| {
        let submissions = submissions.borrow();

        let user_posts: Vec<UserSubmission> = submissions
            .iter()
            .filter(|(_, sub)| sub.user == user_id)
            .map(|(id, sub)| {
                let mut sub_with_id = sub.clone();
                sub_with_id.data_id = id;
                sub_with_id
            })
            .collect();

        ic_cdk::println!(
            "DEBUG: Returning {} posts submitted by user {}",
            user_posts.len(),
            user_id
        );

        user_posts
    })
}

#[query]
#[candid_method(query)]
fn get_submissions_by_city(city: String) -> Vec<UserSubmission> {
    SUBMISSIONS.with(|submissions| {
        let submissions = submissions.borrow();
        submissions
            .iter()
            .filter(|(_, sub)| sub.data.city.to_lowercase() == city.to_lowercase())
            .map(|(_, sub)| sub.clone())
            .collect()
    })
}

#[query]
#[candid_method(query)]
fn get_rewarded_submissions(user_id: String) -> Vec<UserSubmission> {
    SUBMISSIONS.with(|submissions| {
        let submissions = submissions.borrow();
        submissions
            .iter()
            .filter(|(_, sub)| sub.user == user_id && sub.rewarded)
            .map(|(_, sub)| sub.clone())
            .collect()
    })
}

#[query]
#[candid_method(query)]
fn get_user_submission_locations(user_id: String) -> Vec<SubmissionLocationInfo> {
    SUBMISSIONS.with(|submissions| {
        let submissions = submissions.borrow();
        submissions
            .iter()
            .filter(|(_, sub)| sub.user == user_id)
            .map(|(id, sub)| SubmissionLocationInfo {
                data_id: id,
                latitude: sub.data.latitude,
                longitude: sub.data.longitude,
                status: sub.status.clone(),
            })
            .collect()
    })
}

#[query]
#[candid_method(query)]
fn get_user_submission_summary(user_id: String) -> Vec<UserSubmissionSummary> {
    SUBMISSIONS.with(|submissions| {
        let submissions = submissions.borrow();
        submissions
            .iter()
            .filter(|(_, sub)| sub.user == user_id)
            .map(|(id, sub)| UserSubmissionSummary {
                data_id: id,
                city: sub.data.city.clone(),
                status: sub.status.clone(),
            })
            .collect()
    })
}

#[query]
#[candid_method(query)]
fn get_submission_map_by_city() -> Vec<(String, Vec<SubmissionInfo>)> {
    let submissions: Vec<_> = SUBMISSIONS.with(|s| s.borrow().iter().map(|(_, v)| v.clone()).collect());
    let users: HashMap<_,_> = USERS.with(|u| u.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    let mut city_map: HashMap<String, Vec<SubmissionInfo>> = HashMap::new();

    for submission in submissions {
        let user_info = users.get(&submission.user);
        let submission_info = SubmissionInfo {
            data_id: submission.data_id,
            user_id: submission.user.clone(),
            username: user_info.and_then(|u| u.username.clone()),
            first_name: user_info.and_then(|u| u.first_name.clone()),
            last_name: user_info.and_then(|u| u.last_name.clone()),
            profile_picture_url: user_info.and_then(|u| u.profile_picture_url.clone()),
            latitude: submission.data.latitude,
            longitude: submission.data.longitude,
            city: submission.data.city.clone(),
            temperature: submission.data.temperature,
            weather: submission.data.weather.clone(),
            timestamp: submission.data.timestamp,
            submission_photo_url: submission.data.submission_photo_url.clone(),
            rewarded: submission.rewarded,
            status: submission.status.clone(),
        };
        city_map.entry(submission.data.city.clone()).or_default().push(submission_info);
    }

    city_map.into_iter().collect()
}

#[query]
#[candid_method(query)]
fn get_paid_submission_map_by_city() -> Vec<(String, Vec<SubmissionInfo>)> {
    let submissions: Vec<_> = SUBMISSIONS.with(|s| s.borrow().iter().map(|(_, v)| v.clone()).collect());
    let users: HashMap<_,_> = USERS.with(|u| u.borrow().iter().map(|(k, v)| (k.clone(), v.clone())).collect());
    let mut city_map: HashMap<String, Vec<SubmissionInfo>> = HashMap::new();

    for submission in submissions {
        if submission.rewarded || submission.status == PostStatus::PAID {
            let user_info = users.get(&submission.user);
            let submission_info = SubmissionInfo {
                data_id: submission.data_id,
                user_id: submission.user.clone(),
                username: user_info.and_then(|u| u.username.clone()),
                first_name: user_info.and_then(|u| u.first_name.clone()),
                last_name: user_info.and_then(|u| u.last_name.clone()),
                profile_picture_url: user_info.and_then(|u| u.profile_picture_url.clone()),
                latitude: submission.data.latitude,
                longitude: submission.data.longitude,
                city: submission.data.city.clone(),
                temperature: submission.data.temperature,
                weather: submission.data.weather.clone(),
                timestamp: submission.data.timestamp,
                submission_photo_url: submission.data.submission_photo_url.clone(),
                rewarded: submission.rewarded,
                status: submission.status.clone(),
            };
            city_map.entry(submission.data.city.clone()).or_default().push(submission_info);
        }
    }

    city_map.into_iter().collect()
}

// -------- Post status functions --------
#[query]
#[candid_method(query)]
fn get_post_status(data_id: u64) -> String {
    SUBMISSIONS.with(|subs| {
        let subs = subs.borrow();
        match subs.get(&data_id) {
            Some(sub) => format!("Post status: {:?}", sub.status),
            None => format!("Submission {} not found.", data_id),
        }
    })
}

#[query]
#[candid_method(query)]
fn get_expiration_time(data_id: u64) -> Result<u64, String> {
    SUBMISSIONS.with(|subs| {
        let subs = subs.borrow();
        match subs.get(&data_id) {
            Some(sub) => Ok(sub.expiration_timestamp),
            None => Err(format!("Submission {} not found.", data_id)),
        }
    })
}

#[query]
#[candid_method(query)]
fn get_all_expiration_times() -> Vec<ExpirationInfo> {
    SUBMISSIONS.with(|subs| {
        subs.borrow()
            .iter()
            .map(|(id, sub)| ExpirationInfo {
                data_id: id,
                expiration_timestamp: sub.expiration_timestamp,
            })
            .collect()
    })
}

#[update]
#[candid_method(update)]
fn finalize_post_status(data_id: u64) -> String {
    use PostStatus::*;

    let result = SUBMISSIONS.with(|subs| {
        let mut subs = subs.borrow_mut();
        match subs.get(&data_id) {
            Some(sub) => {
                let now = time();
                if now < sub.expiration_timestamp {
                    return Some(("Post is still open. Not finalized.".to_string(), sub.status.clone()));
                }

                if sub.status != OPEN {
                    return Some((format!("Post already finalized with status {:?}", sub.status), sub.status.clone()));
                }

                let mut updated = sub.clone();
                let votes = VOTES.with(|v| v.borrow().get(&data_id).cloned().unwrap_or_default());

                let valid = votes.iter().filter(|v| v.vote_value).count();
                let invalid = votes.len().saturating_sub(valid);

                if valid > invalid {
                    updated.status = PENDING;
                } else {
                    updated.status = EXPIRED;
                }

                subs.insert(data_id, updated.clone());
                Some((format!("Post finalized as {:?}", updated.status), updated.status))
            },
            None => None
        }
    });

    match result {
        Some((msg, _)) => msg,
        None => format!("Submission {} not found.", data_id),
    }
}

// -------- Vote functions --------

#[update]
#[candid_method(update)]
fn vote_on_data(user_id: String, data_id: u64, vote_value: bool) -> String {
    ic_cdk::println!("DEBUG: vote_on_data called with user_id: {}, data_id: {}, vote_value: {}", user_id, data_id, vote_value);

    let submission_exists = SUBMISSIONS.with(|subs| subs.borrow().contains_key(&data_id));

    if !submission_exists {
        return "Submission not found.".to_string();
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

#[query]
#[candid_method(query)]
fn get_vote_summary(data_id: u64) -> VoteSummary {
    VOTES.with(|votes_map| {
        let votes_map = votes_map.borrow();
        let votes = votes_map.get(&data_id).cloned().unwrap_or_default();

        let upvotes = votes.iter().filter(|v| v.vote_value).count() as u32;
        let downvotes = votes.len() as u32 - upvotes;

        VoteSummary {
            data_id,
            upvotes,
            downvotes,
        }
    })
}

#[query]
#[candid_method(query)]
fn get_votes_by_user(user_id: String) -> Vec<Vote> {
    VOTES.with(|votes_map| {
        let votes_map = votes_map.borrow();
        let mut user_votes = Vec::new();

        for votes in votes_map.values() {
            for vote in votes {
                if vote.user == user_id {
                    user_votes.push(vote.clone());
                }
            }
        }

        ic_cdk::println!(
            "DEBUG: Found {} votes for user {}",
            user_votes.len(),
            user_id
        );

        user_votes
    })
}

#[update]
#[candid_method(update)]
fn update_vote(user_id: String, data_id: u64, new_vote_value: bool) -> String {
    VOTES.with(|votes_map| {
        let mut votes_map = votes_map.borrow_mut();

        if let Some(votes) = votes_map.get_mut(&data_id) {
            for vote in votes.iter_mut() {
                if vote.user == user_id {
                    vote.vote_value = new_vote_value;
                    ic_cdk::println!(
                        "DEBUG: Updated vote for user {} on data {} to {}",
                        user_id, data_id, new_vote_value
                    );
                    return format!("Vote updated successfully for user {}", user_id);
                }
            }
            ic_cdk::println!("DEBUG: User {} has not voted yet on data {}", user_id, data_id);
            "Vote not found for user on this data.".to_string()
        } else {
            ic_cdk::println!("DEBUG: No votes found for data {}", data_id);
            "No votes found for this submission.".to_string()
        }
    })
}

#[update]
#[candid_method(update)]
fn delete_vote(user_id: String, data_id: u64) -> String {
    VOTES.with(|votes_map| {
        let mut votes_map = votes_map.borrow_mut();

        if let Some(votes) = votes_map.get_mut(&data_id) {
            let original_len = votes.len();
            votes.retain(|vote| vote.user != user_id);

            if votes.len() < original_len {
                ic_cdk::println!(
                    "DEBUG: Deleted vote of user {} for data {}",
                    user_id,
                    data_id
                );
                "Vote deleted successfully.".to_string()
            } else {
                ic_cdk::println!(
                    "DEBUG: Vote of user {} not found for data {}",
                    user_id,
                    data_id
                );
                "Vote not found for this user on this submission.".to_string()
            }
        } else {
            ic_cdk::println!(
                "DEBUG: No votes found for data {} when trying to delete vote of user {}",
                data_id,
                user_id
            );
            "No votes found for this submission.".to_string()
        }
    })
}

#[query]
#[candid_method(query)]
fn get_leaderboard_by_total_votes() -> Vec<VoteSummary> {
    VOTES.with(|votes_map| {
        let votes_map = votes_map.borrow();
        
        let mut summaries: Vec<VoteSummary> = votes_map.iter().map(|(data_id, votes)| {
            let upvotes = votes.iter().filter(|v| v.vote_value).count() as u32;
            let downvotes = votes.len() as u32 - upvotes;
            VoteSummary {
                data_id: *data_id,
                upvotes,
                downvotes,
            }
        }).collect();

        summaries.sort_by(|a, b| {
            let total_a = a.upvotes + a.downvotes;
            let total_b = b.upvotes + b.downvotes;
            total_b.cmp(&total_a)  
                .then_with(|| a.data_id.cmp(&b.data_id)) 
        });

        summaries.into_iter().take(10).collect()
    })
}

#[query]
#[candid_method(query)]
fn get_leaderboard_by_upvotes() -> Vec<VoteSummary> {
    VOTES.with(|votes_map| {
        let votes_map = votes_map.borrow();
        
        let mut summaries: Vec<VoteSummary> = votes_map.iter().map(|(data_id, votes)| {
            let upvotes = votes.iter().filter(|v| v.vote_value).count() as u32;
            let downvotes = votes.len() as u32 - upvotes;
            VoteSummary {
                data_id: *data_id,
                upvotes,
                downvotes,
            }
        }).collect();

        summaries.sort_by(|a, b| {
            b.upvotes.cmp(&a.upvotes) 
                .then_with(|| a.data_id.cmp(&b.data_id)) 
        });

        summaries.into_iter().take(10).collect()
    })
}

// -------- Challenge functions --------
#[update]
#[candid_method(update)]
fn create_challenge(title: String, latitude: f64, longitude: f64, radius_m: f64, expiration_duration: u64, picture_url: String) -> u64 {
    let now = time();
    let expiration = now + expiration_duration;
    let id = CHALLENGES.with(|c| c.borrow().len() as u64 + 1);
    let challenge = Challenge {
        id,
        title,
        latitude,
        longitude,
        radius_m,
        expiration,
        picture_url,
    };
    CHALLENGES.with(|c| {
        c.borrow_mut().insert(id, challenge.clone());
    });
    ic_cdk::println!("Created challenge: {} (id: {})", challenge.title, challenge.id);
    id
}

#[query]
#[candid_method(query)]
fn get_active_challenges(lat: f64, lon: f64) -> Vec<Challenge> {
    let now = time();
    CHALLENGES.with(|c| {
        ic_cdk::println!("Starting get_active_challenges for location: ({}, {})", lat, lon);
        let challenges: Vec<Challenge> = c.borrow()
            .iter()
            .filter(|(_, ch)| {
                ic_cdk::println!("Checking challenge: lat={}, lon={}, radius={}, expiration={}", 
                    ch.latitude, ch.longitude, ch.radius_m, ch.expiration);
                ch.expiration > now && is_within_geofence(lat, lon, ch.latitude, ch.longitude, ch.radius_m)
            })
            .map(|(_, ch)| ch.clone())
            .collect();
        ic_cdk::println!("Found {} active challenges", challenges.len());
        challenges
    })
}

fn is_submission_within_challenge(challenge_id: u64, lat: f64, lon: f64) -> Result<(), String> {
    CHALLENGES.with(|c| {
        match c.borrow().get(&challenge_id) {
            Some(ch) => {
                if ch.expiration < time() {
                    return Err("Challenge expired".to_string());
                }
                if !is_within_geofence(lat, lon, ch.latitude, ch.longitude, ch.radius_m) {
                    return Err("Location outside challenge geofence".to_string());
                }
                Ok(())
            }
            None => Err("Challenge not found".to_string()),
        }
    })
}

fn is_within_geofence(
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
    radius_m: f64,
) -> bool {
    let distance = haversine_distance(lat1, lon1, lat2, lon2);
    ic_cdk::println!("distance: {}, radius: {}", distance, radius_m);
    distance <= radius_m
}

fn haversine_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371e3_f64; // meters
    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let delta_phi = (lat2 - lat1).to_radians();
    let delta_lambda = (lon2 - lon1).to_radians();

    let a = (delta_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (delta_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    r * c
}

#[update]
#[candid_method(update)]
fn submit_weather_data_with_challenge(
    telegram_id: String,
    latitude: f64,
    longitude: f64,
    city: String,
    temperature: f64,
    weather: String,
    submission_photo_url: String,
    challenge_id: u64,
) -> Result<u64, String> {
    const SECOND: u64 = 1_000_000_000;
    let timestamp = time();
    let expiration_timestamp = timestamp + 300 * SECOND;

    is_submission_within_challenge(challenge_id, latitude, longitude)?;

    let data_id = SUBMISSIONS.with(|s| s.borrow().len() as u64 + 1);

    let new_data = UserSubmission {
        data_id,
        user: telegram_id.clone(),
        data: WeatherData {
            latitude,
            longitude,
            city,
            temperature,
            weather,
            timestamp,
            submission_photo_url,
        },
        rewarded: false,
        status: PostStatus::OPEN,
        expiration_timestamp,
    };

    SUBMISSIONS.with(|s| {
        s.borrow_mut().insert(data_id, new_data.clone());
    });

    Ok(data_id)
}

#[query]
#[candid_method(query)]
fn get_challenge(challenge_id: u64) -> Result<Challenge, String> {
    CHALLENGES.with(|c| {
        c.borrow().get(&challenge_id)
        .map(|ch| ch.clone()) 
        .ok_or("Challenge not found.".to_string())
    })
}

#[query]
#[candid_method(query)]
fn get_all_challenges() -> Vec<Challenge> {
    CHALLENGES.with(|c| {
        c.borrow().iter().map(|(_, ch)| ch.clone()).collect()
    })
}

#[query]
#[candid_method(query)]
fn get_user_submissions_by_challenge(user_id: String, challenge_id: u64) -> Vec<UserSubmission> {
    SUBMISSIONS.with(|subs| {
        subs.borrow().iter()
            .filter(|(_, sub)| sub.user == user_id 
                && is_submission_in_challenge(sub, challenge_id))
            .map(|(_, sub)| sub.clone())
            .collect()
    })
}

fn is_submission_in_challenge(sub: &UserSubmission, challenge_id: u64) -> bool {
    CHALLENGES.with(|c| {
        c.borrow().get(&challenge_id).map_or(false, |ch| {
            is_within_geofence(
                sub.data.latitude,
                sub.data.longitude,
                ch.latitude,
                ch.longitude,
                ch.radius_m,
            )
        })
    })
}

#[query]
#[candid_method(query)]
fn get_challenges_expiring_soon(cutoff_seconds: u64) -> Vec<Challenge> {
    let now = time();
    CHALLENGES.with(|c| {
        c.borrow().iter()
            .filter(|(_, ch)| {
                let remaining = ch.expiration.saturating_sub(now);
                remaining <= cutoff_seconds * 1_000_000_000
            })
            .map(|(_, ch)| ch.clone())
            .collect()
    })
}

#[query]
#[candid_method(query)]
fn get_challenges_by_radius(lat: f64, lon: f64, radius_m: f64) -> Vec<Challenge> {
    let now = time();
    CHALLENGES.with(|c| {
        c.borrow().iter()
            .filter(|(_, ch)| {
                ch.expiration > now &&
                haversine_distance(lat, lon, ch.latitude, ch.longitude) <= radius_m
            })
            .map(|(_, ch)| ch.clone())
            .collect()
    })
}

#[query]
#[candid_method(query)]
fn get_submissions_by_challenge(challenge_id: u64) -> Vec<UserSubmission> {
    CHALLENGES.with(|c| {
        let challenge = match c.borrow().get(&challenge_id) {
            Some(ch) => ch.clone(),
            None => return vec![],
        };

        SUBMISSIONS.with(|subs| {
            subs.borrow().iter()
                .filter(|(_, sub)| {
                    is_within_geofence(
                        sub.data.latitude,
                        sub.data.longitude,
                        challenge.latitude,
                        challenge.longitude,
                        challenge.radius_m,
                    )
                })
                .map(|(_, sub)| sub.clone())
                .collect()
        })
    })
}

// -------- Reward functions --------

#[update]
#[candid_method(update)]
async fn reward_user(data_id: u64) -> Result<String, String> {
    let submission = SUBMISSIONS.with(|subs| subs.borrow().get(&data_id))
        .ok_or_else(|| format!("Submission {} not found.", data_id))?
        .clone();

    if submission.rewarded {
        return Err("Already rewarded.".to_string());
    }

    let vote_list = VOTES.with(|vm| vm.borrow().get(&data_id).cloned().unwrap_or_default());
    let valid_votes = vote_list.iter().filter(|v| v.vote_value).count();
    let invalid_votes = vote_list.len() - valid_votes;

    if valid_votes <= invalid_votes {
        return Err("Majority voted invalid, no reward.".to_string());
    }

    let user_id = submission.user.clone();
    let recipient_address = USERS.with(|u| {
        u.borrow().get(&user_id).and_then(|u| u.wallet_address.clone())
    }).ok_or("User has no wallet address. Please connect your wallet first.")?;

    let recipient_principal = Principal::from_text(&recipient_address)
        .map_err(|_| "Recipient address is not a valid principal".to_string())?;

    let amount_to_send = Nat::from(10_000u64);

    let transfer_arg = TransferArg {
        to: Account {
            owner: recipient_principal,
            subaccount: None,
        },
        fee: None,
        memo: None,
        from_subaccount: None,
        created_at_time: None,
        amount: amount_to_send,
    };

    let ledger_canister_id = Principal::from_text("br5f7-7uaaa-aaaaa-qaaca-cai")
        .map_err(|_| "Invalid ledger canister ID".to_string())?;

    let (transfer_result,): (TransferResult,) = call(
        ledger_canister_id,
        "icrc1_transfer",
        (transfer_arg,),
    )
    .await
    .map_err(|e| format!("Ledger call failed: {:?}", e))?;

    match transfer_result {
        TransferResult::Ok(block_idx) => {
            SUBMISSIONS.with(|subs| {
                let mut subs = subs.borrow_mut();
                if let Some(existing_sub) = subs.get(&data_id) {
                    let mut sub = existing_sub.clone();
                    sub.rewarded = true;
                    subs.insert(data_id, sub);
                }
            });
            Ok(format!("Successfully rewarded user {} at block {}", user_id, block_idx))
        },
        TransferResult::Err(err) => {
            Err(format!("Transfer failed: {:?}", err))
        }
    }
}

// -------- Canister init --------

#[init]
fn init() {
    ic_cdk::println!("Canister initialized with StableBTreeMap storage.");
}

// Role management functions
// fn is_admin(user_id: &UserId) -> bool {
//     USERS.with(|users| {
//         users.borrow()
//             .get(user_id)
//             .map(|user| user.role == Role::ADMIN)
//             .unwrap_or(false)
//     })
// }

// fn is_moderator(user_id: &UserId) -> bool {
//     USERS.with(|users| {
//         users.borrow()
//             .get(user_id)
//             .map(|user| user.role == Role::MODERATOR || user.role == Role::ADMIN)
//             .unwrap_or(false)
//     })
// }

#[update]
#[candid_method(update)]
async fn update_user_role(
    caller_id: String,
    target_user_id: String,
    new_role: Role,
) -> Result<String, String> {
    // Check if caller is admin and get target user in a single scope
    let target_user = USERS.with(|users| {
        let users = users.borrow();
        let caller_user = users.get(&caller_id).ok_or("Caller not found")?;
        let target_user = users.get(&target_user_id).ok_or("Target user not found")?;
        
        // Allow creating the first admin
        let is_first_admin = !users.iter().any(|(_, u)| u.role == Role::Admin);
        
        if !is_first_admin && caller_user.role != Role::Admin {
            return Err("Only admins can update roles".to_string());
        }
        
        Ok(target_user.clone())
    })?;
    
    // Update the user in storage
    USERS.with(|users| {
        let mut users = users.borrow_mut();
        let mut updated_user = target_user;
        updated_user.role = new_role;
        users.insert(target_user_id, updated_user);
    });

    Ok("Role updated successfully".to_string())
}

#[query]
fn get_user_role(user_id: String) -> Result<Role, String> {
    USERS.with(|users| {
        users.borrow()
            .get(&user_id)
            .map(|user| user.role.clone())
            .ok_or_else(|| "User not found".to_string())
    })
}

#[update]
#[candid_method(update)]
fn mark_submission_rewarded(data_id: u64) -> Result<String, String> {
    // First check if the submission exists
    let submission = SUBMISSIONS.with(|subs| {
        subs.borrow().get(&data_id).clone()
    }).ok_or_else(|| format!("Submission {} not found", data_id))?;

    // Check if already rewarded
    if submission.rewarded {
        return Err("Submission already rewarded".to_string());
    }

    // Update the submission status
    SUBMISSIONS.with(|subs| {
        let mut subs = subs.borrow_mut();
        if let Some(mut sub) = subs.get(&data_id) {
            sub.rewarded = true;
            sub.status = PostStatus::PAID;
            subs.insert(data_id, sub.clone());
            Ok(format!("Successfully marked submission {} as rewarded", data_id))
        } else {
            Err(format!("Failed to update submission {}", data_id))
        }
    })
}
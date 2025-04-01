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
use ic_cdk::call; 

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
    first_name: Option<String>,
    last_name: Option<String>,
    username: Option<String>,
    language_code: Option<String>,
    is_bot: bool,
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

#[derive(CandidType, Deserialize)]
pub struct TransferArg {
    pub to: Account,
    pub fee: Option<Nat>,
    pub memo: Option<Vec<u8>>,
    pub from_subaccount: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
    pub amount: Nat,
}

// ------------------ ICRC-1 LEDGER CALL DEFINITIONS ------------------
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
    // etc.
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

// TODO:
// make create_tg_user function inside of icp/dao-backend
// if user does not exitst, create; if user exists updates field (when they change their name or language_code)
#[update]
#[candid_method(update)]
fn create_tg_user(telegram_id: String, first_name: String, last_name: String, username: String, language_code: String, is_bot: bool) -> String {
    USERS.with(|users_map| {
        let mut users = users_map.borrow_mut();
        let user_id = telegram_id.clone();

        match users.get(&user_id) {
            Some(mut user) => {
                ic_cdk::println!("👤 Existing user found: {}. Updating info.", user_id);
                user.first_name = Some(first_name);
                user.last_name = Some(last_name);
                user.username = Some(username);
                user.language_code = Some(language_code);
                user.is_bot = is_bot;
                users.insert(user_id.clone(), user);
                format!("Updated user: {}", user_id)
            }
            None => {
                let new_user = User {
                    user_id: user_id.clone(),
                    balance: 0,
                    first_name: Some(first_name),
                    last_name: Some(last_name),
                    username: Some(username),
                    language_code: Some(language_code),
                    is_bot,
                };
                users.insert(user_id.clone(), new_user);
                ic_cdk::println!("Created new user: {}", user_id);
                format!("Created new user: {}", user_id)
            }
        }
    })
}

#[query]
#[candid_method(query)]
fn get_all_users() -> Vec<User> {
    USERS.with(|users_map| {
        let users = users_map.borrow();
        let all_users: Vec<User> = users.iter().map(|(_, user)| user.clone()).collect();
        ic_cdk::println!("✅ DEBUG: Returning all users, count: {}", all_users.len());
        all_users
    })
}

#[update]
fn submit_weather_data(telegram_id: String, recipient_address: String, latitude: f64, longitude: f64, city: String, temperature: f64, weather: String) -> u64 {
    let timestamp = time();
    ic_cdk::println!("🚀 Received weather submission from {}", telegram_id);
    
    // let data_id = SUBMISSIONS.with(|s| s.borrow().len() as u64 + 1);

    let data_id = SUBMISSIONS.with(|s| s.borrow().len() as u64 + 1);

    let new_data = UserSubmission {
        user: telegram_id.clone(),
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

// #[update]
// #[candid_method(update)]
// fn reward_user(data_id: u64) -> Result<String, String> {
//     SUBMISSIONS.with(|submissions| {
//         let mut submissions = submissions.borrow_mut();
//         let submission = match submissions.get(&data_id) {
//             Some(sub) => {
//                 ic_cdk::println!("✅ DEBUG: Found submission: {:?}", sub);
//                 sub.clone()
//             }
//             None => {
//                 ic_cdk::println!("❌ ERROR: Submission not found for data_id: {}", data_id);
//                 return Err("Weather data submission not found.".to_string());
//             }
//         };

//         if submission.rewarded {
//             ic_cdk::println!("⚠️ WARNING: Submission already rewarded: {:?}", submission);
//             return Err("User has already been rewarded for this submission.".to_string());
//         }

//         let vote_list = VOTES.with(|votes_map| {
//             let votes_map = votes_map.borrow();
//             votes_map.get(&data_id).cloned().unwrap_or_default()
//         });

//         if vote_list.is_empty() {
//             ic_cdk::println!("⚠️ WARNING: No votes found for submission {}", data_id);
//             return Err("No votes found for this submission. Cannot determine validity.".to_string());
//         }

//         let valid_votes = vote_list.iter().filter(|v| v.vote_value).count();
//         let invalid_votes = vote_list.len() - valid_votes;
//         ic_cdk::println!(
//             "📝 INFO: Submission {} - valid_votes: {}, invalid_votes: {}",
//             data_id, valid_votes, invalid_votes
//         );

//         let user_id = submission.user.clone();

//         if valid_votes > invalid_votes {
//             USERS.with(|users| {
//                 let mut users = users.borrow_mut();

//                 let user = users.get(&user_id).map(|u| u.clone());

//                 match user {
//                     Some(mut existing_user) => {
//                         existing_user.balance += 10;
//                         users.insert(user_id.clone(), existing_user.clone());
//                         ic_cdk::println!(
//                             "✅ DEBUG: Updated balance for user {}: {}",
//                             user_id, existing_user.balance
//                         );
//                     }
//                     None => {
//                         let new_user = User {
//                             user_id: user_id.clone(),
//                             balance: 10,
//                         };
//                         users.insert(user_id.clone(), new_user.clone());
//                         ic_cdk::println!(
//                             "🆕 INFO: Created new user balance for {}: {} tokens",
//                             user_id, new_user.balance
//                         );
//                     }
//                 }

//                 match users.get(&user_id) {
//                     Some(updated_user) => {
//                         ic_cdk::println!(
//                             "🔍 DEBUG: Final balance check for {}: {} tokens",
//                             user_id, updated_user.balance
//                         );
//                     }
//                     None => {
//                         ic_cdk::println!("❌ ERROR: Balance update failed for {}", user_id);
//                     }
//                 }
//             });

//             let mut updated_submission = submission.clone();
//             updated_submission.rewarded = true;
//             submissions.insert(data_id, updated_submission);
//             ic_cdk::println!("🎉 SUCCESS: User {} rewarded with 10 tokens.", user_id);
//             Ok(format!("User {} rewarded with 10 tokens.", user_id))
//         } else {
//             ic_cdk::println!(
//                 "❌ ERROR: Submission {} rejected, majority voted as invalid",
//                 data_id
//             );
//             Err("Majority voted the data as invalid, no reward given.".to_string())
//         }
//     })
// }

// ------------------ NEW ASYNC reward_user with ICRC-1 TRANSFER ------------------
#[update]
#[candid_method(update)]
async fn reward_user(data_id: u64, recipient_address: String) -> Result<String, String> {
    // 0) Check for submission and vote results.
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

    // 1) Convert recipient_address (provided as a string) to a Principal.
    let recipient_principal = Principal::from_text(&recipient_address)
        .map_err(|_| "Recipient address is not a valid principal".to_string())?;

    // 2) Set the amount to send to 10,000 tokens (raw value as per your command).
    let amount_to_send = Nat::from(10_000u64);

    // 3) Build the transfer arguments.
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

    // 4) Use the correct ledger canister ID (ensure this is your actual ledger canister ID).
    let ledger_canister_id = Principal::from_text("br5f7-7uaaa-aaaaa-qaaca-cai")
        .map_err(|_| "Invalid ledger canister ID".to_string())?;

    // 5) Call the ledger's "icrc1_transfer" method.
    let (transfer_result,): (TransferResult,) = call(
        ledger_canister_id,
        "icrc1_transfer",
        (transfer_arg,),
    )
    .await
    .map_err(|e| format!("Ledger call failed: {:?}", e))?;

    // 6) Process the transfer result.
    match transfer_result {
        TransferResult::Ok(block_idx) => {
            // Mark the submission as rewarded.
            SUBMISSIONS.with(|subs| {
                let mut subs = subs.borrow_mut();
                if let Some(existing_sub) = subs.get(&data_id) {
                    let mut sub = existing_sub.clone();
                    sub.rewarded = true;
                    subs.insert(data_id, sub);
                }
            });
            Ok(format!("Successfully rewarded recipient: {}, block index = {}", recipient_address, block_idx))
        },
        TransferResult::Err(err) => {
            Err(format!("Transfer failed: {:?}", err))
        }
    }
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
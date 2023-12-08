use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::{api::call::ManualReply, query, update};
use ic_stable_structures::StableVec;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{
    storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::collections::BTreeMap;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 100;
const MAX_KEY_SIZE: u32 = 100;

struct CanisterState {
    controllers: BTreeMap<String, bool>,
    max_call_per_user: u16,
    timer_limit: u64,
}

#[derive(CandidType, Deserialize)]
struct Param {
    last_use: u64,
    count: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
struct ParamKey(String);

impl Storable for Param {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

impl Storable for ParamKey {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        // String already implements `Storable`.
        self.0.to_bytes()
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self(String::from_bytes(bytes))
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_KEY_SIZE,
        is_fixed_size: false,
    };
}

thread_local! {
    // The memory manager is used for simulating multiple memories. Given a `MemoryId` it can
    // return a memory that can be used by stable structures.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static PARAMS_WHITELIST: RefCell<StableBTreeMap<ParamKey, Param, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );
}

// Retrieves the value associated with the given key if it exists.
#[query(name = "get")]
fn get(key: String) -> Option<Param> {
    PARAMS_WHITELIST.with(|p| p.borrow().get(&ParamKey(key)))
}

// Inserts an entry into the map and returns the previous value of the key if it exists.
#[update(name = "insert")]
fn whitelist_param(key: String, value: Param) -> Option<Param> {
    PARAMS_WHITELIST.with(|p| p.borrow_mut().insert(ParamKey(key), value))
}

#[query(name = "getSelf", manual_reply = true)]
fn get_self() -> ManualReply<Principal> {
    let id = ic_cdk::api::caller();
    return ManualReply::one(id);
}

// ic_cdk::export_candid!();
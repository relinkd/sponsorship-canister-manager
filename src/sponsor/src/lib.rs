use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use ic_cdk::{api::call::ManualReply, query, update};
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{
    storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable,
};
use std::collections::BTreeMap;
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 100;
const MAX_KEY_SIZE: u32 = 100;

#[derive(CandidType, Deserialize, Clone, Debug, Default)]
struct CanisterState {
    controllers: BTreeMap<String, bool>,
    max_call_per_user: u16,
    timer_limit: u64,
}

#[derive(CandidType, Deserialize, Clone)]
struct Param {
    is_whitelisted: bool,
    is_principal: bool,
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
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));

    static PARAMS_WHITELIST: RefCell<StableBTreeMap<ParamKey, Param, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        )
    );

    static CANISTER_STATE: RefCell<CanisterState> = RefCell::new(CanisterState::default());
}

#[query(name = "getParam")]
fn get_param(key: String) -> Option<Param> {
    PARAMS_WHITELIST.with(|p| p.borrow().get(&ParamKey(key)))
}

#[update(name = "whitelistParam")]
fn whitelist_param(key: String, value: Param) -> Option<Param> {
    let id = ic_cdk::api::caller();
    let is_controller = ic_cdk::api::is_controller(&id);

    if !is_controller {
        ic_cdk::api::trap("Access denied")
    } else {
        PARAMS_WHITELIST.with(|p| p.borrow_mut().insert(ParamKey(key), value))
    }
}

#[query(name = "isManagerCanister")]
fn is_manager_canister(principal: String) -> bool {
    CANISTER_STATE.with(|cs| {
        let canister_state = cs.borrow();

        if let Some(canister) = canister_state.controllers.get(&principal.clone()) {
            *canister
        } else {
            false
        }
    })
}

#[update(name = "logParamUsage")]
fn log_param_usage(key: String) -> Option<Param> {
    let id = ic_cdk::api::caller();
    // let is_controller = ic_cdk::api::is_controller(&id);

    let is_manager = is_manager_canister(id.to_string());

    ic_cdk::println!("{:?}", id.to_string());
    ic_cdk::println!("{:?}", is_manager);

    PARAMS_WHITELIST.with(|pl| {
        let mut params_mut = pl.borrow_mut();

        if is_manager {
            if let Some(mut param) = params_mut.get(&ParamKey(key.clone())) {
                param.last_use = ic_cdk::api::time();
                param.count += 1;
                params_mut.insert(ParamKey(key.clone()), param.clone());
    
                Some(param)
            } else {
                ic_cdk::api::trap("Param is not defined")
            }
        } else {
            ic_cdk::api::trap("Access denied")
        }
    })
}

#[query(name = "isController")]
fn is_controller() -> bool {
    let id = ic_cdk::api::caller();
    let is_controller = ic_cdk::api::is_controller(&id);

    return is_controller;
}

#[update(name = "setTimerLimit")]
fn set_timer_limit(limit: u64) -> ()  {
    let id = ic_cdk::api::caller();
    let is_controller = ic_cdk::api::is_controller(&id);

    if !is_controller {
        ic_cdk::api::trap("Access denied")
    } else {
        CANISTER_STATE.with(|cs| {
            let mut canister_state = cs.borrow_mut();
    
            canister_state.timer_limit = limit;
        });
    }
}

#[update(name = "editManagerCanister")]
fn edit_manager_canister(controller: String, state: bool) -> ()  {
    let id = ic_cdk::api::caller();
    let is_controller = ic_cdk::api::is_controller(&id);

    if !is_controller {
        ic_cdk::api::trap("Access denied")
    } else {
        CANISTER_STATE.with(|cs| {
            let mut canister_state = cs.borrow_mut();
    
            canister_state.controllers.insert(controller.clone(), state);
        });
    }
}

#[query(name = "isParamWhitelisted", manual_reply = true)]
fn is_param_whitelisted(key: String) -> ManualReply<bool> {
    PARAMS_WHITELIST.with(|p| {
        if let Some(param) = p.borrow().get(&ParamKey(key.clone())) {
            ManualReply::one(param.is_whitelisted)
        } else {
            ManualReply::one(false)
        }
    })
}

#[query(name = "isParamTimeAvailable", manual_reply = true)]
fn is_param_time_available(key: String) -> ManualReply<bool> {
    PARAMS_WHITELIST.with(|p| {
        if let Some(param) = p.borrow().get(&ParamKey(key.clone())) {
            CANISTER_STATE.with(|cs| {
                let canister_state = cs.borrow();
        
                ManualReply::one((ic_cdk::api::time() - param.last_use) > canister_state.timer_limit)
            })
        } else {
            ManualReply::one(false)
        }
    })
}

// ic_cdk::export_candid!();
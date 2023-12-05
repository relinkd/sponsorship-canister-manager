use candid::{CandidType, Deserialize, Principal};
use ic_cdk::{api::call::ManualReply, query, update};

#[query(name = "getSelf", manual_reply = true)]
fn get_self() -> ManualReply<Principal> {
    let id = ic_cdk::api::caller();
    return ManualReply::one(id);
}
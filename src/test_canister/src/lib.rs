use candid::Principal;
use ic_cdk::api::call::CallResult;

#[ic_cdk::update(name = "testInterCanister")]
async fn test_inter_canister(principal: Principal) -> bool {
    let result: CallResult<(bool,)> =
        ic_cdk::call(principal, "isController", ()).await;

    ic_cdk::println!("{:?}", result);

    return result.unwrap().0;
}

#[ic_cdk::query]
fn get_self() -> String {
    let id = ic_cdk::api::caller().to_string();

    return id;
}

#[ic_cdk::update(name = "logParamTest")]
async fn log_param_test(principal: Principal, param: String) -> () {

    let result: CallResult<()> =
        ic_cdk::call(principal, "logParamUsage", (param,)).await;

    ic_cdk::println!("{:?}", result);
}
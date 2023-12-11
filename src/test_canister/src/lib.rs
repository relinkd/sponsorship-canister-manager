#[ic_cdk::query]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[ic_cdk::query]
fn get_self() -> String {
    let id = ic_cdk::api::caller().to_string();

    return id;
}
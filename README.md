# Sponsorhip canister manager 

---

The Sponsorship Canister Manager is deployed for one or multiple canisters within the protocol and is controlled by the owner/owners.

The canister allows sponsoring user transactions based on Reversed Gas using a whitelist of string parameters (in the context of {r}elinkd, EVM wallets are considered).

## admin functions

Only the canister controller can add a parameter to the whitelist.

`dfx canister call sponsor whitelistParam '("your evm wallet", record { is_whitelisted = true; last_use = 0; count =  0; is_principal = false })'`

The controller must also add canisters that will have access to interact with the whitelist. If you need to remove a canister from the whitelist, replace true with false.

`dfx canister call sponsor editManagerCanister '("your canister principal id", true)'`

To add a time limit, pass the time duration in nanoseconds, for example, half an hour.

`dfx canister call sponsor setTimerLimit '(1800000000000)'`

## inter-canister interaction 

Canisters that are in the whitelist can fully utilize all functionality with a check on whether the parameter is in the whitelist and how long it has been since it was last called. After the canister sponsors the call of the operation, it must log the call of a specific parameter (using the `log_param` method, updating the last usage date and the total number of calls).

For more details, you can refer to `src/test_canister`.

## the project locally

If you want to test your project locally, you can use the following commands:

```shell
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```shell
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

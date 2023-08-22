#[cfg(test)]
mod tests {
    use aurora_sdk_integration_tests::{aurora_engine_sdk::types::near_account_to_evm_address, tokio, workspaces, {utils::process}, aurora_engine, wnear, ethabi};
    use std::path::Path;
    use aurora_sdk_integration_tests::aurora_engine::AuroraEngine;
    use aurora_sdk_integration_tests::aurora_engine_types::parameters::engine::{CallArgs, FunctionCallArgsV1};
    use aurora_sdk_integration_tests::aurora_engine_types::types::{Address, Wei};
    use aurora_sdk_integration_tests::aurora_engine_types::U256;
    use aurora_sdk_integration_tests::utils::forge;
    use aurora_sdk_integration_tests::utils::ethabi::DeployedContract;
    use aurora_sdk_integration_tests::workspaces::Contract;

    #[tokio::test]
    async fn counter_test() {
        let worker = workspaces::sandbox().await.unwrap();
        let near_counter = deploy_near_counter(&worker).await;

        let engine = aurora_engine::deploy_latest(&worker).await.unwrap();
        let wnear = wnear::Wnear::deploy(&worker, &engine).await.unwrap();

        let user_account = worker.dev_create_account().await.unwrap();
        let aurora_counter = deploy_aurora_counter(&engine, &user_account, wnear.aurora_token.address, &near_counter).await;

        let user_address = near_account_to_evm_address(user_account.id().as_bytes());
        const NEAR_DEPOSIT: u128 =  2 * near_sdk::ONE_NEAR;

        engine.mint_wnear(&wnear, user_address, NEAR_DEPOSIT).await.unwrap();

        let evm_call_args = wnear.aurora_token.create_approve_call_bytes(aurora_counter.address, U256::MAX);
        let result = engine
            .call_evm_contract_with(
                &user_account,
                wnear.aurora_token.address,
                evm_call_args,
                Wei::zero(),
            ).await.unwrap();
        aurora_engine::unwrap_success(result.status).unwrap();

        increment(&engine, &user_account, aurora_counter).await;

        let counter_val: u64 = near_counter.view("get_num").await.unwrap().json().unwrap();
        assert_eq!(counter_val, 1);
    }

    async fn deploy_near_counter(worker: &workspaces::Worker<workspaces::network::Sandbox>) -> Contract {
        let contract_path = Path::new("../../near/contracts");
        let output = tokio::process::Command::new("bash")
            .current_dir(contract_path)
            .args(["build.sh"])
            .output()
            .await
            .unwrap();

        process::require_success(&output).unwrap();

        let artifact_path =
            contract_path.join("target/wasm32-unknown-unknown/release/counter.wasm");
        let wasm_bytes = tokio::fs::read(artifact_path).await.unwrap();
        let near_counter = worker.dev_deploy(&wasm_bytes).await.unwrap();

        near_counter.call("new")
            .transact().await.unwrap().into_result().unwrap();

        near_counter
    }

    async fn deploy_aurora_counter(engine: &AuroraEngine,
                                   user_account: &workspaces::Account,
                                   wnear_address: Address,
                                   near_counter: &Contract) -> DeployedContract {
        let contract_path = "../contracts";

        let aurora_sdk_path = Path::new("../contracts/lib/aurora-contracts-sdk/aurora-solidity-sdk");
        let codec_lib = forge::deploy_codec_lib(&aurora_sdk_path, engine)
            .await
            .unwrap();
        let utils_lib = forge::deploy_utils_lib(&aurora_sdk_path, engine)
            .await
            .unwrap();
        let aurora_sdk_lib =
            forge::deploy_aurora_sdk_lib(&aurora_sdk_path, engine, codec_lib, utils_lib)
                .await
                .unwrap();

        let constructor = forge::forge_build(
            contract_path,
            &[
                format!(
                    "@auroraisnear/aurora-sdk/aurora-sdk/AuroraSdk.sol:AuroraSdk:0x{}",
                    aurora_sdk_lib.encode()
                )
            ],
            &[
                "out",
                "Counter.sol",
                "Counter.json",
            ],
        ).await.unwrap();

        let deploy_bytes = constructor.create_deploy_bytes_with_args(&[
            ethabi::Token::Address(wnear_address.raw()),
            ethabi::Token::String(near_counter.id().to_string()),
        ]);

        let address = engine
            .deploy_evm_contract_with(user_account, deploy_bytes)
            .await
            .unwrap();

        constructor.deployed_at(address)
    }

    async fn increment(
        engine: &AuroraEngine,
        user_account: &workspaces::Account,
        aurora_counter: DeployedContract) {
        let contract_args = aurora_counter.create_call_method_bytes_without_args(
            "incrementXCC"
        );

        let call_args = CallArgs::V1(FunctionCallArgsV1 {
            contract: aurora_counter.address,
            input: contract_args,
        });

        let outcome = user_account
            .call(engine.inner.id(), "call")
            .args_borsh(call_args)
            .max_gas()
            .transact()
            .await
            .unwrap();

        assert!(
                outcome.failures().is_empty(),
                "Call to set failed: {:?}",
                outcome.failures()
            );
    }
}

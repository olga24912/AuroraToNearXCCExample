#[cfg(test)]
mod tests {
    use aurora_sdk_integration_tests::{aurora_engine_sdk::types::near_account_to_evm_address, tokio, workspaces, {utils::process}, aurora_engine, wnear};
    use std::path::Path;
    use aurora_sdk_integration_tests::aurora_engine::AuroraEngine;
    use aurora_sdk_integration_tests::aurora_engine_types::types::Address;
    use aurora_sdk_integration_tests::utils::forge;
    use aurora_sdk_integration_tests::workspaces::Contract;

    #[tokio::test]
    async fn counter_test() {
        let worker = workspaces::sandbox().await.unwrap();
        let near_counter = deploy_near_counter(&worker).await;

        let engine = aurora_engine::deploy_latest(&worker).await.unwrap();
        let wnear = wnear::Wnear::deploy(&worker, &engine).await.unwrap();

        let user_account = worker.dev_create_account().await.unwrap();
        let aurora_counter = deploy_aurora_counter(&engine, &user_account, wnear.aurora_token.address, &near_counter);

        //let user_address = near_account_to_evm_address(user_account.id().as_bytes());
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

        near_counter
    }

    async fn deploy_aurora_counter(engine: &AuroraEngine,
                                   user_account: &workspaces::Account,
                                   wnear_address: Address,
                                   near_fast_bridge: &Contract) {
        let contract_path = "../contracts";

        let aurora_sdk_path = Path::new("./aurora-contracts-sdk/aurora-solidity-sdk");
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

        /*let deploy_bytes = constructor.create_deploy_bytes_without_constructor();

        let address = engine
            .deploy_evm_contract_with(user_account, deploy_bytes)
            .await
            .unwrap();

        let aurora_fast_bridge_impl = constructor.deployed_at(address);

        let contract_args = aurora_fast_bridge_impl.create_call_method_bytes_with_args(
            "initialize",
            &[
                ethabi::Token::Address(wnear_address.raw()),
                ethabi::Token::String(near_fast_bridge.id().to_string()),
                ethabi::Token::String(engine.inner.id().to_string()),
                ethabi::Token::Bool(false),
            ],
        );

        call_aurora_contract(
            aurora_fast_bridge_impl.address,
            contract_args,
            &user_account,
            engine.inner.id(),
            true,
        ).await.unwrap();

        return aurora_fast_bridge_impl;*/
    }
}
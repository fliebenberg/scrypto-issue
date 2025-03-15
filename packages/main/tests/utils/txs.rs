use scrypto_test::prelude::*;

use super::{AccInfo, TestRunnerType};

pub fn token_buy(
    xrd_to_send: Decimal,
    from_account: &AccInfo,
    token_curve_address: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> TransactionReceiptV1 {
    let token_buy_manifest = ManifestBuilder::new()
        // .lock_fee(from_account.address.clone(), dec!("10"))
        .lock_fee_from_faucet()
        .call_method(
            from_account.address.clone(),
            "withdraw",
            manifest_args![XRD, xrd_to_send.clone()],
        )
        .take_all_from_worktop(XRD, "tx_bucket")
        .call_method_with_name_lookup(token_curve_address.clone(), "buy", |lookup| {
            (lookup.bucket("tx_bucket"),)
        })
        .try_deposit_entire_worktop_or_abort(from_account.address, None)
        .build();
    let receipt = test_runner.execute_manifest(
        token_buy_manifest,
        vec![NonFungibleGlobalId::from_public_key(&from_account.pubkey)],
    );

    if receipt.is_commit_failure() {
        panic!("Problem with token buy tx! {:?}", receipt);
    }
    // let result = receipt.expect_commit_success();
    receipt
}

pub fn token_buy_amount(
    amount_to_buy: Decimal,
    xrd_to_send: Decimal,
    from_account: &AccInfo,
    token_curve_address: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> TransactionReceiptV1 {
    let token_buy_manifest = ManifestBuilder::new()
        // .lock_fee(from_account.address.clone(), dec!("10"))
        .lock_fee_from_faucet()
        .call_method(
            from_account.address.clone(),
            "withdraw",
            manifest_args![XRD, xrd_to_send.clone()],
        )
        .take_all_from_worktop(XRD, "tx_bucket")
        .call_method_with_name_lookup(token_curve_address.clone(), "buy_amount", |lookup| {
            (amount_to_buy, lookup.bucket("tx_bucket"))
        })
        .try_deposit_entire_worktop_or_abort(from_account.address, None)
        .build();
    let receipt = test_runner.execute_manifest(
        token_buy_manifest,
        vec![NonFungibleGlobalId::from_public_key(&from_account.pubkey)],
    );

    if receipt.is_commit_failure() {
        panic!("Problem with token buy amount tx! {:?}", receipt);
    }
    // let result = receipt.expect_commit_success();
    receipt
}

pub fn token_sell(
    tokens_to_send: Decimal,
    token_address: &ResourceAddress,
    from_account: &AccInfo,
    token_curve_address: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> TransactionReceiptV1 {
    let token_sell_manifest = ManifestBuilder::new()
        // .lock_fee(from_account.address.clone(), dec!("10"))
        .lock_fee_from_faucet()
        .call_method(
            from_account.address.clone(),
            "withdraw",
            manifest_args![token_address.clone(), tokens_to_send.clone()],
        )
        .take_all_from_worktop(token_address.clone(), "tx_bucket")
        .call_method_with_name_lookup(token_curve_address.clone(), "sell", |lookup| {
            (lookup.bucket("tx_bucket"),)
        })
        .try_deposit_entire_worktop_or_abort(from_account.address, None)
        .build();
    let receipt = test_runner.execute_manifest(
        token_sell_manifest,
        vec![NonFungibleGlobalId::from_public_key(&from_account.pubkey)],
    );

    if receipt.is_commit_failure() {
        panic!("Problem with token sell tx! {:?}", receipt);
    }
    // let result = receipt.expect_commit_success();
    receipt
}

pub fn token_sell_for_xrd_amount(
    xrd_to_receive: Decimal,
    tokens_to_send: Decimal,
    token_address: &ResourceAddress,
    from_account: &AccInfo,
    token_curve_address: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> TransactionReceiptV1 {
    let token_sell_for_xrd_manifest = ManifestBuilder::new()
        // .lock_fee(from_account.address.clone(), dec!("10"))
        .lock_fee_from_faucet()
        .call_method(
            from_account.address.clone(),
            "withdraw",
            manifest_args![token_address.clone(), tokens_to_send.clone()],
        )
        .take_all_from_worktop(token_address.clone(), "tx_bucket")
        .call_method_with_name_lookup(
            token_curve_address.clone(),
            "sell_for_xrd_amount",
            |lookup| (xrd_to_receive, lookup.bucket("tx_bucket")),
        )
        .try_deposit_entire_worktop_or_abort(from_account.address, None)
        .build();
    let receipt = test_runner.execute_manifest(
        token_sell_for_xrd_manifest,
        vec![NonFungibleGlobalId::from_public_key(&from_account.pubkey)],
    );

    if receipt.is_commit_failure() {
        panic!("Problem with token sell for xrd tx! {:?}", receipt);
    }
    // let result = receipt.expect_commit_success();
    receipt
}

pub fn claim_first_buy_tokens(
    token_curve_address: &ComponentAddress,
    owner_badge_address: &ResourceAddress,
    from_account: &AccInfo,
    test_runner: &mut TestRunnerType,
) -> TransactionReceiptV1 {
    let token_sell_manifest = ManifestBuilder::new()
        // .lock_fee(from_account.address.clone(), dec!("10"))
        .lock_fee_from_faucet()
        .call_method(
            from_account.address.clone(),
            "create_proof_of_amount",
            manifest_args![owner_badge_address.clone(), dec!("1")],
        )
        .call_method(
            token_curve_address.clone(),
            "claim_first_buy_tokens",
            manifest_args![],
        )
        .try_deposit_entire_worktop_or_abort(from_account.address, None)
        .build();
    let receipt = test_runner.execute_manifest(
        token_sell_manifest,
        vec![NonFungibleGlobalId::from_public_key(&from_account.pubkey)],
    );

    if receipt.is_commit_failure() {
        panic!("Problem with claim_first_buy_tokens tx! {:?}", receipt);
    }
    // let result = receipt.expect_commit_success();
    receipt
}

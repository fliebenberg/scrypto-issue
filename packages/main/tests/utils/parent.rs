use main::rakoonfun_main::rakoonfun_main::RakoonFunMain;
use scrypto::prelude::Url;
use scrypto_test::prelude::*;

use super::*;

pub fn create_parent_component(
    owner_badge_address: &ResourceAddress,
    max_token_supply_to_trade: Decimal,
    max_xrd_market_cap: Decimal,
    virtual_supply: Decimal,
    fair_launch_period_mins: u32,
    tx_fee_perc: Decimal,
    listing_fee_perc: Decimal,
    creator_fee_perc: Decimal,
    token_creation_fee: Decimal,
    first_buy_max_perc: Decimal,
    first_buy_lock_mins: i64,
    account: &AccInfo,
    test_runner: &mut TestRunnerType,
) -> (ComponentAddress, ComponentAddress) {
    let package_address = test_runner.compile_and_publish(this_package!());
    let new_component_manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RakoonFunMain",
            "new",
            manifest_args![
                "Rakoon.fun Main Component",
                "The main component for the Rakoon.fun Token Creator",
                "https://rakoon.fun",
                "https://rakoon.fun/rakoonfun-OG-image.png",
                max_token_supply_to_trade,
                max_xrd_market_cap,
                virtual_supply,
                fair_launch_period_mins,
                tx_fee_perc,
                listing_fee_perc,
                creator_fee_perc,
                token_creation_fee,
                first_buy_max_perc,
                first_buy_lock_mins,
                owner_badge_address,
            ],
        )
        .try_deposit_entire_worktop_or_abort(account.address, None)
        .build();
    let receipt = test_runner.execute_manifest(
        new_component_manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.pubkey)],
    );

    // println!(
    //     "Create Main Component Receipt: {:?}\n",
    //     receipt
    // );
    if receipt.is_commit_failure() {
        panic!("Problem with creating Main component! {:?}", receipt);
    }
    let result = receipt.expect_commit_success();
    // println!(
    //     "New Main components: {:?}",
    //     result.new_component_addresses()
    // );
    let component_address = result.new_component_addresses()[0];
    // println!("Main component address: {:?}", component_address);
    let dapp_def = result.new_component_addresses()[1];
    // println!("Main component dapp definition address: {:?}", dapp_def);
    // show_parent_state(&component_address, test_runner);
    (component_address, dapp_def)
}

pub fn get_parent_state(
    parent_address: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> RakoonFunMain {
    let state = get_component_state::<RakoonFunMain, NoExtension, InMemorySubstateDatabase>(
        parent_address.clone(),
        test_runner,
    );
    state
}

pub fn get_parent_state_list(
    parent_component: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> Vec<(String, String)> {
    let parent_state = get_parent_state(parent_component, test_runner);
    let mut result: Vec<(String, String)> = vec![];
    result.push((
        String::from("max_token_supply_to_trade"),
        format!("{:?}", parent_state.max_token_supply_to_trade),
    ));
    result.push((
        String::from("max_xrd_market_cap"),
        format!("{:?}", parent_state.max_xrd_market_cap),
    ));
    result.push((
        String::from("virtual_supply"),
        format!("{:?}", parent_state.virtual_supply),
    ));
    result.push((
        String::from("listing_fee_perc"),
        format!("{:?}", parent_state.listing_fee_perc),
    ));
    result.push((
        String::from("creator_fee_perc"),
        format!("{:?}", parent_state.creator_fee_perc),
    ));
    result
}

pub fn show_parent_state(parent_component: &ComponentAddress, test_runner: &mut TestRunnerType) {
    for (field, value) in get_parent_state_list(parent_component, test_runner) {
        println!("{}: {}", field, value);
    }
}

pub fn change_main_dapp_metadata(env: &mut TestEnv) {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            env.owner_account.address.clone(),
            "create_proof_of_amount",
            manifest_args!(env.owner_badge_address, dec!("1")),
        )
        .set_metadata(
            env.parent_dapp_def,
            "icon_url",
            Url::of("https://rakoon.fun/favicon-32x32.png"),
        )
        .build();
    let receipt = env.test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(
            &env.owner_account.pubkey,
        )],
    );

    println!("Change main dapp def metadata Receipt: {:?}\n", receipt);
    if receipt.is_commit_failure() {
        panic!("Problem with changing main dapp metadata! {:?}", receipt);
    }
    receipt.expect_commit_success();
}

use main::rakoonfun_token_curve::rakoonfun_token_curve::RakoonFunTokenCurve;
use scrypto_test::prelude::*;

use super::*;

pub fn create_token_curve_component(
    name: String,
    symbol: String,
    description: String,
    icon_url: String,
    telegram: String,
    x: String,
    website: String,
    curve: String,
    virtual_supply: Decimal,
    token_creation_fee: Decimal,
    first_buy_amount: Decimal,
    first_buy_lock_mins: i64,
    component_address: &ComponentAddress,
    account: &AccInfo,
    test_runner: &mut TestRunnerType,
) -> (
    ComponentAddress,
    ComponentAddress,
    ResourceAddress,
    ResourceAddress,
) {
    let mut new_component_manifest_builder = ManifestBuilder::new()
        .call_method(
            account.address.clone(),
            "withdraw",
            manifest_args![XRD, token_creation_fee.clone()],
        )
        .take_all_from_worktop(XRD, "fee_bucket");
    if first_buy_amount > Decimal::ZERO {
        new_component_manifest_builder = new_component_manifest_builder
            .call_method(
                account.address.clone(),
                "withdraw",
                manifest_args![XRD, dec!("400")],
            )
            .take_all_from_worktop(XRD, "first_buy_bucket");
    }
    new_component_manifest_builder = new_component_manifest_builder
        .lock_fee_from_faucet()
        .call_method_with_name_lookup(
            component_address.clone(),
            "new_token_curve_advanced",
            |lookup| {
                (
                    name,
                    symbol,
                    description,
                    icon_url,
                    telegram,
                    x,
                    website,
                    curve,
                    String::from(""),
                    dec!("1000000"),
                    dec!("1000000"),
                    virtual_supply,
                    dec!("0.05"),
                    0u32,
                    first_buy_amount,
                    if first_buy_amount > Decimal::ZERO {
                        Some(lookup.bucket("first_buy_bucket"))
                    } else {
                        None
                    },
                    first_buy_lock_mins,
                    lookup.bucket("fee_bucket"),
                )
            },
        )
        .try_deposit_entire_worktop_or_abort(account.address, None);
    let new_component_manifest = new_component_manifest_builder.build();
    let receipt = test_runner.execute_manifest(
        new_component_manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.pubkey)],
    );

    // println!("Create Token Component Receipt: {:?}\n", receipt);
    if receipt.is_commit_failure() {
        panic!("Problem with creating Token component! {:?}", receipt);
    }
    let result = receipt.expect_commit_success();
    // println!("New Token resources: {:?}", result.new_resource_addresses());
    let component_address = result.new_component_addresses()[0];
    // println!("TokenCurves component address: {:?}", component_address);
    let dapp_def = result.new_component_addresses()[1];
    // println!("TokenCurvese dapp definition address: {:?}", dapp_def);
    let token_address = result.new_resource_addresses()[1];
    let token_owner_badge_address = result.new_resource_addresses()[0];
    (
        component_address,
        dapp_def,
        token_address,
        token_owner_badge_address,
    )
}

pub fn get_token_data(token_address: ResourceAddress, test_runner: &mut TestRunnerType) {
    let token_name = test_runner
        .get_metadata(token_address.into(), "name")
        .expect("Could not find metadata field 'name'");
    println!("Token name: {:?}", token_name);
}

pub fn get_token_state(
    token_component: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> RakoonFunTokenCurve {
    let token_state = get_component_state::<
        RakoonFunTokenCurve,
        NoExtension,
        InMemorySubstateDatabase,
    >(token_component.clone(), test_runner);
    token_state
}

pub fn get_token_state_list(
    token_component: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> Vec<(String, String)> {
    let token_state = get_token_state(token_component, test_runner);
    let mut result: Vec<(String, String)> = vec![];
    result.push((
        String::from("max_token_supply"),
        format!("{:?}", token_state.max_token_supply),
    ));
    result.push((
        String::from("max_token_supply_to_trade"),
        format!("{:?}", token_state.max_token_supply_to_trade),
    ));
    result.push((
        String::from("max_xrd_market_cap"),
        format!("{:?}", token_state.max_xrd_market_cap),
    ));
    result.push((
        String::from("max_xrd"),
        format!("{:?}", token_state.max_xrd),
    ));
    result.push((
        String::from("virtual_supply"),
        format!("{:?}", token_state.virtual_supply),
    ));
    result.push((
        String::from("last_price"),
        format!("{:?}", token_state.last_price),
    ));
    result.push((
        String::from("current_supply"),
        format!("{:?}", token_state.current_supply),
    ));
    result.push((
        String::from("listing_fee_perc"),
        format!("{:?}", token_state.listing_fee_perc),
    ));
    result.push((
        String::from("creator_fee_perc"),
        format!("{:?}", token_state.creator_fee_perc),
    ));
    result
}

pub fn get_token_state_map(
    token_component: &ComponentAddress,
    test_runner: &mut TestRunnerType,
) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    for (field, value) in get_token_state_list(token_component, test_runner) {
        result.insert(field, value);
    }
    result
}

pub fn show_token_state(token_component: &ComponentAddress, test_runner: &mut TestRunnerType) {
    for (field, value) in get_token_state_list(token_component, test_runner) {
        println!("{}: {}", field, value);
    }
}

use scrypto_test::prelude::*;

pub mod utils;

#[test]
fn setup_env_test() {
    let mut env = utils::setup_test_env(0, false, dec!("1000000"), dec!("100000"), false);
    println!(
        "Test env owner account address: {:?}",
        env.owner_account.address
    );
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
}

#[test]
fn simple_buy_sell_tests() {
    let mut env = utils::setup_test_env(0, false, dec!("1000000"), dec!("100000"), false);
    println!("\n## Token state before buy:");
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let _first_buy_receipt = utils::txs::token_buy(
        dec!(100),
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    // println!("First buy receipt: {:?}", _first_buy_receipt);
    println!("\n## Token state after buy:");
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.011911384251964326"),
        "Incorrect price after buy. {:?}",
        token_state.last_price
    );
    assert!(
        token_state.current_supply == dec!("109139.288306110584511913"),
        "Incorrect supply after buy. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("100"),
        "Incorrect XRD in component after first buy. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("9139.288306110584511913"),
        "Incorrect token Balance in account after first buy. {:?}",
        token_balance.clone()
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("9900"),
        "Incorrect XRD Balance in account after first buy. {:?}",
        xrd_balance.clone()
    );

    let _first_sell_receipt = utils::txs::token_sell(
        dec!("9139.288306110584511913"),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    // println!("Receipt after first_sell: {:?}", _first_sell_receipt);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.01"),
        "Incorrect current price after sell. {:?}",
        token_state.last_price
    );
    assert!(
        token_state.current_supply == dec!("100000"),
        "Incorrect supply after sell. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("0"),
        "Incorrect XRD in component after first sell. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("0"),
        "Incorrect token Balance in account after first sell. {:?}",
        token_balance
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("10000"),
        "Incorrect XRD Balance in account after first sell. {:?}",
        xrd_balance.clone()
    );

    println!("\n## Token state before 2nd buy:");
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let _second_buy_receipt = utils::txs::token_buy_amount(
        dec!("9139.288306110584511913"),
        dec!("100"),
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    println!("Receipt after second_buy: {:?}", _second_buy_receipt);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.011911384251964326"),
        "Incorrect current price after 2nd buy. {:?}",
        token_state.last_price
    );
    assert!(
        token_state.current_supply == dec!("109139.288306110584511913"),
        "Incorrect supply after 2nd buy. {:?}",
        token_state.current_supply
    );
    // utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("100"),
        "Incorrect XRD in component after 2nd buy. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("9139.288306110584511913"),
        "Incorrect token Balance in account after 2nd buy. {:?}",
        token_balance
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("9900"),
        "Incorrect XRD Balance in account after 2nd buy. {:?}",
        xrd_balance.clone()
    );

    let _second_sell_receipt = utils::txs::token_sell_for_xrd_amount(
        dec!("50"),
        dec!("4370.332988945930"),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.010976533998250059"),
        "Incorrect current price after sell for xrd amount. {:?}",
        token_state.last_price
    );
    assert!(
        token_state.current_supply == dec!("104768.955317164729069559"),
        "Incorrect supply after sell for xrd amount. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("50"),
        "Incorrect XRD in component after sell for xrd amount. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("4768.955317164729069559"),
        "Incorrect Token Balance in wallet after sell for XRD amount. {:?}",
        token_balance
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("9950"),
        "Incorrect XRD Balance in account after sell for XRD amount. {:?}",
        xrd_balance.clone()
    );

    let _last_sell_receipt = utils::txs::token_sell(
        dec!("4768.955317164729069559"),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    println!("After last sell: ");
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.01"),
        "Incorrect current price after last sell: {:?}",
        token_state.last_price
    );
    assert!(
        token_state.current_supply == dec!("100000"),
        "Incorrect supply after last sell"
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("0"),
        "Incorrect XRD in component after last sell. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("0"),
        "Incorrect Token Balance in wallet after last sell. {:?}",
        token_balance
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("10000"),
        "Incorrect XRD Balance in account after last sell. {:?}",
        xrd_balance.clone()
    );
}

#[test]
fn buy_sell_all_tokens_tests() {
    let mut env = utils::setup_test_env(0, false, dec!("1000000"), dec!("100000"), false);
    utils::load_account_with_xrd(&env.owner_account, dec!("500000"), &mut env.test_runner);
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    println!("XRD Balance in account: {:?}", xrd_balance);

    let _first_buy_receipt = utils::txs::token_buy(
        dec!(500000),
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("1"),
        "Incorrect price after buy"
    );
    assert!(
        token_state.current_supply == dec!("1000000"),
        "Incorrect supply after buy. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("333000"),
        "Incorrect XRD in component after first buy. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("900000"),
        "Incorrect token Balance in account after first buy. {:?}",
        token_balance.clone()
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("177000"),
        "Incorrect XRD Balance in account after first buy. {:?}",
        xrd_balance.clone()
    );

    let _first_sell_receipt = utils::txs::token_sell_for_xrd_amount(
        dec!("333000"),
        dec!(900000),
        &env.token1_address,
        &env.owner_account,
        &env.token1_component,
        &mut env.test_runner,
    );
    println!("Sell receipt: {:?}", _first_sell_receipt);
    utils::token::show_token_state(&env.token1_component, &mut env.test_runner);
    let token_state = utils::token::get_token_state(&env.token1_component, &mut env.test_runner);
    assert!(
        token_state.last_price == dec!("0.01"), // result not zero because of limited accuracy of scrypto calcs
        "Incorrect price after sell. {:?}",
        token_state.last_price
    );
    assert!(
        token_state.current_supply == dec!("100000"),
        "Incorrect supply after sell. {:?}",
        token_state.current_supply
    );
    let component_xrd_balance = env
        .test_runner
        .get_component_balance(env.token1_component, XRD);
    assert!(
        component_xrd_balance == dec!("0"),
        "Incorrect XRD in component after first sell. {:?}",
        component_xrd_balance
    );
    let token_balance = env.test_runner.get_component_balance(
        env.owner_account.address.clone(),
        env.token1_address.clone(),
    );
    assert!(
        token_balance == dec!("0"),
        "Incorrect token Balance in account after first sell. {:?}",
        token_balance.clone()
    );
    let xrd_balance = env
        .test_runner
        .get_component_balance(env.owner_account.address.clone(), XRD);
    assert!(
        xrd_balance == dec!("510000"),
        "Incorrect XRD Balance in account after first buy. {:?}",
        xrd_balance.clone()
    );
}

use scrypto_test::prelude::*;

pub mod parent;
pub mod token;
pub mod txs;

pub type TestRunnerType = LedgerSimulator<NoExtension, InMemorySubstateDatabase>;
pub struct AccInfo {
    pub address: ComponentAddress,
    pub pubkey: Secp256k1PublicKey,
}
pub struct TestEnv {
    pub test_runner: TestRunnerType,
    pub owner_account: AccInfo,
    pub owner_badge_address: ResourceAddress,
    pub parent_component_address: ComponentAddress,
    pub parent_dapp_def: ComponentAddress,
    pub token1_component: ComponentAddress,
    pub token1_address: ResourceAddress,
    pub token1_owner_address: ResourceAddress,
    pub tx_fee_perc: Decimal,
    pub listing_fee_perc: Decimal,
    pub creator_fee_perc: Decimal,
    pub token_creation_fee: Decimal,
}

pub fn setup_test_env(
    fair_launch_period: u32,
    with_fees: bool,
    max_supply: Decimal,
    virtual_supply: Decimal,
    with_first_buy: bool,
) -> TestEnv {
    let mut test_runner = LedgerSimulatorBuilder::new().build();
    let owner_account = create_new_account(&mut test_runner);
    let owner_badge_address =
        test_runner.create_fungible_resource(dec!(1), DIVISIBILITY_MAXIMUM, owner_account.address);
    let mut tx_fee_perc = dec!("0");
    let mut listing_fee_perc = dec!("0");
    let mut creator_fee_perc = dec!("0");
    let mut token_creation_fee = dec!("0");
    let mut first_buy_max_perc = dec!("0");
    let mut first_buy_lock_mins = 0i64;
    let mut first_buy_amount = dec!("0");
    if with_fees {
        tx_fee_perc = dec!("0.01");
        listing_fee_perc = dec!("0.05");
        creator_fee_perc = dec!("0.05");
        token_creation_fee = dec!("100");
    }
    if with_first_buy {
        first_buy_max_perc = dec!("0.1");
        first_buy_lock_mins = 10i64;
        first_buy_amount = dec!("100000");
    }

    let (parent_component, parent_dapp_def) = parent::create_parent_component(
        &owner_badge_address,
        max_supply.clone(),
        max_supply.clone(),
        virtual_supply.clone(),
        fair_launch_period,
        tx_fee_perc.clone(),
        listing_fee_perc.clone(),
        creator_fee_perc.clone(),
        token_creation_fee.clone(),
        first_buy_max_perc.clone(),
        first_buy_lock_mins.clone(),
        &owner_account,
        &mut test_runner,
    );
    let (token1_component, _token1_dapp_def, token1_address, token1_owner_address) =
        token::create_token_curve_component(
            String::from("First Token"),
            String::from("FIRST"),
            String::from("The first token on Rakoon.fun"),
            String::from("https://rakoon.fun/rakoonfun-OG-image.png"),
            String::from(""),
            String::from(""),
            String::from("https://rakoon.fun"),
            String::from("cs2"),
            virtual_supply.clone(),
            token_creation_fee.clone(),
            first_buy_amount.clone(),
            first_buy_lock_mins.clone(),
            &parent_component,
            &owner_account,
            &mut test_runner,
        );
    TestEnv {
        test_runner,
        owner_account,
        owner_badge_address,
        parent_component_address: parent_component,
        parent_dapp_def,
        token1_component,
        token1_address,
        token1_owner_address,
        tx_fee_perc,
        listing_fee_perc,
        creator_fee_perc,
        token_creation_fee,
    }
}

pub fn create_new_account(test_runner: &mut TestRunnerType) -> AccInfo {
    let (pubkey, _, address) = test_runner.new_allocated_account();
    AccInfo { address, pubkey }
}

pub fn load_account_with_xrd(account: &AccInfo, amount: Decimal, test_runner: &mut TestRunnerType) {
    const FREE_AMOUNT: Decimal = dec!("10000");
    let mut allocated_amount = Decimal::ZERO;
    while allocated_amount < amount {
        let txmanifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .deposit_batch(account.address, ManifestExpression::EntireWorktop)
            .build();
        let _receipt = test_runner.execute_manifest(
            txmanifest,
            vec![NonFungibleGlobalId::from_public_key(&account.pubkey)],
        );
        allocated_amount += FREE_AMOUNT;
    }
}

pub fn get_component_state<T: ScryptoDecode, E: NativeVmExtension, D: TestDatabase>(
    component_address: ComponentAddress,
    test_runner: &mut LedgerSimulator<E, D>,
) -> T {
    let node_id: &NodeId = component_address.as_node_id();
    let partition_number = MAIN_BASE_PARTITION;
    let substate_key: &SubstateKey = &ComponentField::State0.into();

    let substate: Option<T> =
        test_runner
            .substate_db()
            .get_substate(node_id, partition_number, substate_key);
    substate.unwrap()
}

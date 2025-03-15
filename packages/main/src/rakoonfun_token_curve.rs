use crate::rakoonfun_main::rakoonfun_main::RakoonFunMain;
use curve_calcs_interface::curve_calcs_interface::*;
use dex_interface::dex_interface::*;
use scrypto::prelude::*;

#[derive(ScryptoSbor, Debug)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub icon_url: String,
    pub telegram_url: String,
    pub x_url: String,
    pub website_url: String,
}

#[derive(ScryptoSbor, NonFungibleData)]
struct OwnerBadgeData {
    #[mutable]
    pub name: String,
}

#[derive(ScryptoSbor, NonFungibleData)]
struct FairLaunchReceiptData {
    // TODO add user id
    xrd_amount: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunTokenCreateEvent {
    // TODO add user id
    token_address: ResourceAddress,
    component_address: ComponentAddress,
    main_component: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunTokenTradeEvent {
    // TODO add user id
    token_address: ResourceAddress,
    side: String,
    fair_launch_period: bool,
    token_amount: Decimal,
    xrd_amount: Decimal,
    end_price: Decimal,
    total_xrd_traded: Decimal,
    total_txs: u32,
    main_component: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunClaimTokensEvent {
    // TODO add user id
    tokens_claimed: Decimal,
    xrd_amount: Decimal,
    token_address: ResourceAddress,
    main_component: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunDexEvent {
    event_type: String,
    pool_address: ComponentAddress,
    token_address: ResourceAddress,
    main_component: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunClaimCreatorFeeEvent {
    // TODO add user id
    fee_claimed: Decimal,
    token_address: ResourceAddress,
    main_component: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunClaimTokenFeeEvent {
    fee_claimed: Decimal,
    token_address: ResourceAddress,
    main_component: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunChangeTokenCurveCalcsEvent {
    old_curve_calc_component: ComponentAddress,
    new_curve_calc_component: ComponentAddress,
    token_address: ResourceAddress,
    main_component: ComponentAddress,
}

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunChangeTokenDexAdaptorEvent {
    old_dex_adaptor_component: Option<ComponentAddress>,
    new_dex_adaptor_component: ComponentAddress,
    token_address: ResourceAddress,
    main_component: ComponentAddress,
}

type CurveCalcsAdaptor = CurveCalcsAdaptorInterfaceScryptoStub;
type DexAdaptor = DexAdaptorInterfaceScryptoStub;

#[blueprint]
#[events(
    RakoonFunTokenCreateEvent,
    RakoonFunTokenTradeEvent,
    RakoonFunClaimTokensEvent,
    RakoonFunDexEvent,
    RakoonFunClaimCreatorFeeEvent,
    RakoonFunClaimTokenFeeEvent,
    RakoonFunChangeTokenCurveCalcsEvent,
    RakoonFunChangeTokenDexAdaptorEvent
)]
mod rakoonfun_token_curve {
    enable_function_auth! {
        new => AccessRule::AllowAll;
    }

    enable_method_auth! {
        roles {
            creator => updatable_by: [OWNER];
            rakoonfun_admin => updatable_by: [rakoonfun_admin];
        },
        methods {
            buy => PUBLIC;
            buy_amount => PUBLIC;
            first_buy => PUBLIC;
            sell => PUBLIC;
            sell_for_xrd_amount => PUBLIC;
            claim_fair_launch_tokens => PUBLIC;
            claim_first_buy_tokens => restrict_to: [creator];
            claim_creator_fees => restrict_to: [creator];
            claim_fees => restrict_to: [rakoonfun_admin];
            change_curve_calcs => restrict_to: [rakoonfun_admin];
            set_dex_adaptor => restrict_to: [creator, rakoonfun_admin];
            list_token => PUBLIC;
            remove_listing => restrict_to: [rakoonfun_admin];
        }
    }
    struct RakoonFunTokenCurve {
        pub parent_address: ComponentAddress, // address of the parent component that this bonding curve component is part of
        pub address: ComponentAddress,        // the address of this bonding curve component
        pub owner_badge_address: ResourceAddress, // the address of the owner badge for this token and component
        pub dapp_def_address: GlobalAddress,      // the dapp def account address for this component
        pub token_manager: FungibleResourceManager, // the resource manager for the token created as part of this component
        pub max_token_supply: Decimal, // the maximum supply of the token that will be available after it is listed on a dex
        pub max_token_supply_to_trade: Decimal, // the maximum supply of the token that can be traded on the bonding curve
        pub max_xrd_market_cap: Decimal, // the maximum market cap in XRD that will be reached when the max tokens have been traded on bonding curve
        pub max_xrd: Decimal, // the maximum XRD that will be received into this component
        pub virtual_supply: Decimal, // the virtual supply that will be created to avoid the flat part of the curve for a token
        pub virtual_supply_xrd: Decimal, // the xrd value of the virtual supply
        pub tx_fee_perc: Decimal,    // fee % taken on every tx, specified in decimals 1% = 0.01,
        pub listing_fee_perc: Decimal, // fee % taken when a token is listed on external dex, specified in decimals 1% = 0.01
        pub creator_fee_perc: Decimal, // fee % paid to the token creator when the token is listed on a dex, specified in decimals 1% = 0.01
        pub xrd_vault: Vault,          // the vault that holds all the XRD recived by the component
        pub token_vault: Vault, // the vault that holds the tokens received by the component (this vault is only used once the token is listed on an external dex)
        pub fee_vault: Vault,   // vault that holds all the fees earned by the component
        pub creator_fee_vault: Vault, // vault that holds fees earned by the creator of the token
        pub last_price: Decimal, // the price reached with the last trade on the component
        pub current_supply: Decimal, // the current supply of the token associated with this component
        pub fair_launch_period_mins: u32, // the number of mins allocated for a fair launch period
        pub in_fair_launch_period: bool, // indicates whether the token is still in its fair_launch_period
        pub fair_launch_receipt_manager: NonFungibleResourceManager, // teh resource manager for fair launch receipts
        pub fair_launch_tokens: Vault, // vault containing tokens that are bought during fair launch period
        pub fair_launch_xrd: Decimal, // amount of xrd corresponding to tokens in fair launch tokens vault - used to determine tokens that cna be claimed
        pub time_created: i64, // the date the token curve was created in seconds since unix epoch - included for easy lookup
        pub target_reached: i64, // the date the token reached its target market cap in seconds since unix epoch
        pub curve_calcs: CurveCalcsAdaptor, // the curve calcs to be used in this component
        pub dex_adaptor: Option<DexAdaptor>, // the currently selected Dex adaptor that will be used for listing the token
        pub dex_listing_address: Option<ComponentAddress>, // the pool/pair address of the token listing
        pub listing_token_address: Option<ResourceAddress>, // the resource address of the current listing token
        pub other_tokens: KeyValueStore<ResourceAddress, Vault>, // vaults that hold other tokens returned from the dex
        pub xrd_traded: Decimal, // the total value of XRD traded across all trades (buys and sells)
        pub no_txs: u32,         // the total number of buy sell txs
        pub first_buy_vault: Vault,
        pub first_buy_claim: i64,
    }
    // TODO Add Token Minter badge, that is initially held by the component and then can be passed to the token creator (with an optional lockup period) after launch on a dex.
    // TODO Add ability to pass tokens + info to another component to allow for upgrading of token components
    impl RakoonFunTokenCurve {
        // a function that creates a new bonding curve component
        // the function takes in several values that are used to launch the new token and set up the bonding curve component
        // the function returns a global instance of the component, a bucket with the owner badge for the new token, the address of the newly created component, the address of the token and an optional Bucket with remaining first_buy XRD
        pub fn new(
            token_info: TokenInfo,
            max_token_supply_to_trade: Decimal,
            max_xrd_market_cap: Decimal,
            virtual_supply: Decimal,
            tx_fee_perc: Decimal,
            listing_fee_perc: Decimal,
            creator_fee_perc: Decimal,
            fair_launch_period_mins: u32,
            parent_address: ComponentAddress,
            parent_owner_rule: AccessRule,
            curve_calcs: CurveCalcsAdaptor,
            dex_adaptor: Option<DexAdaptor>,
            first_buy_amount: Decimal,
            first_buy_bucket: Option<Bucket>,
            first_buy_lock_mins: i64, // mins for locking first buy tokens
        ) -> (
            Global<RakoonFunTokenCurve>,
            Bucket,
            ComponentAddress,
            ResourceAddress,
            Bucket, // first buy tokens (bucket will be empty if no first buy or lock period > 0)
            Bucket, // remaining XRD from first buy bucket
        ) {
            info!("New child virtual supply: {:?}", virtual_supply);
            assert!(tx_fee_perc < Decimal::ONE, "tx_fee_perc cannot be >= 1. tx_fee_perc is specified in decimals, e.g. 1% = 0.01. ");
            assert!(listing_fee_perc < Decimal::ONE, "listing_fee_perc cannot be >= 1. listing_fee_perc is specified in decimals, e.g. 1% = 0.01. ");
            assert!(
                virtual_supply >= Decimal::ZERO && virtual_supply < max_token_supply_to_trade,
                "Virtual supply must be less than the maximum token supply."
            );
            let _parent_instance = Global::<RakoonFunMain>::from(parent_address.clone()); // checks that the function was called from a TokenCurves component
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(<RakoonFunTokenCurve>::blueprint_id());
            let require_component_rule = rule!(require(global_caller(component_address.clone())));

            let owner_badge = ResourceBuilder::new_integer_non_fungible::<OwnerBadgeData>(
                OwnerRole::Updatable(AccessRule::AllowAll),
            ) // this will be reset to any who owns the token after the token has been created
            .mint_roles(mint_roles! {
                minter => rule!(allow_all);
                minter_updater => rule!(allow_all);
            })
            .burn_roles(burn_roles! {
                burner => rule!(deny_all);
                burner_updater => rule!(deny_all);
            })
            .metadata(metadata!(
                init {
                    "name" => format!("{} owner badge.", token_info.symbol.clone()), updatable;
                    "symbol" => format!("{} OWNER", token_info.symbol.clone()), updatable;
                    "icon_url" => Url::of(token_info.icon_url.clone()), updatable;
                    "rakoonfun_component" => component_address.clone(), locked;
                    "tags" => vec!["RakoonFun"], updatable;
                }
            ))
            .mint_initial_supply([(
                1.into(),
                OwnerBadgeData {
                    name: format!("{} original owner", token_info.symbol.clone()),
                },
            )]);
            let owner_badge_manager = owner_badge.resource_manager();
            owner_badge_manager.set_mintable(rule!(require(owner_badge.resource_address()))); // any owner badge holder can mint more owner badges
            owner_badge_manager.lock_mintable();
            owner_badge_manager.set_owner_role(rule!(require(owner_badge.resource_address()))); // set owner role to be anyone with an owner badge

            let mut social_urls_vec: Vec<Url> = vec![];
            if token_info.telegram_url.len() > 0 {
                social_urls_vec.push(Url::of(token_info.telegram_url.clone()));
            }
            if token_info.x_url.len() > 0 {
                social_urls_vec.push(Url::of(token_info.x_url));
            }
            if token_info.website_url.len() > 0 {
                social_urls_vec.push(Url::of(token_info.website_url));
            }

            let virtual_supply_bucket = ResourceBuilder::new_fungible(OwnerRole::Updatable(rule!(
                require(owner_badge.resource_address())
            )))
            .divisibility(DIVISIBILITY_MAXIMUM)
            .mint_roles(mint_roles! {
                minter => require_component_rule.clone();
                minter_updater => require_component_rule.clone();
            })
            .burn_roles(burn_roles! {
                burner => require_component_rule.clone();
                burner_updater => require_component_rule.clone();
            })
            .metadata(metadata!(
                init {
                    "name" => token_info.name.clone(), updatable;
                    "symbol" => token_info.symbol.clone(), updatable;
                    "description" => format!("{} ## Token created on Rakoon.fun.", token_info.description), updatable;
                    "icon_url" => Url::of(token_info.icon_url.clone()), updatable;
                    "social_urls" => social_urls_vec.clone(), updatable;
                    "tags" => vec!["RakoonFunToken"], updatable;
                    "rakoonfun_component" => component_address.clone(), updatable;
                }
            ))
            .mint_initial_supply(virtual_supply.clone());

            let token_manager = virtual_supply_bucket.resource_manager();
            // .create_with_no_initial_supply();

            let token_address = token_manager.address();

            let fair_launch_receipt_manager = ResourceBuilder::new_ruid_non_fungible::<FairLaunchReceiptData>(OwnerRole::Fixed(require_component_rule.clone()))
            .mint_roles(mint_roles! {
                minter => require_component_rule.clone();
                minter_updater => AccessRule::DenyAll;
            })
            .burn_roles(burn_roles! {
                burner => require_component_rule.clone();
                burner_updater => AccessRule::DenyAll;
            })
            .metadata(metadata!(
                init {
                    "name" => format!("{} Fair Launch Receipt", token_info.symbol.clone()), updatable;
                    "symbol" => format!("{}-FAIR", token_info.symbol.clone()), locked;
                    "description" => format!("Rakoon.fun Fair launch receipt for token {}. This receipt can be redeemed for tokens after the fair launch period has expired.", token_info.symbol.clone()), locked;
                }
            ))
            .create_with_no_initial_supply();

            // each component creates its own dapp definition account with permission granted to the token owner to change the metadata in future
            let dapp_def_account =
                Blueprint::<Account>::create_advanced(OwnerRole::Updatable(rule!(allow_all)), None); // will reset owner role after dapp def metadata has been set
            dapp_def_account.set_metadata("account_type", String::from("dapp definition"));
            dapp_def_account
                .set_metadata("name", format!("Rakoon.fun Token: {}", token_info.symbol));
            dapp_def_account.set_metadata(
                "description",
                format!("Rakoon.fun Token: {}", token_info.name),
            );
            dapp_def_account.set_metadata("icon_url", Url::of(token_info.icon_url.clone()));
            dapp_def_account.set_metadata(
                "claimed_entities",
                vec![GlobalAddress::from(component_address.clone())],
            );
            dapp_def_account.set_owner_role(rule!(require(owner_badge.resource_address())));
            let dapp_def_address = GlobalAddress::from(dapp_def_account.address());

            // calculate the amount of xrd that would have been received for the virtual supply
            let virtual_supply_xrd = curve_calcs.calculate_buy_price(
                virtual_supply.clone(),
                Decimal::ZERO,
                max_xrd_market_cap.clone(),
                max_token_supply_to_trade.clone(),
            );
            let max_xrd = curve_calcs.calculate_max_xrd(
                max_xrd_market_cap.clone(),
                max_token_supply_to_trade.clone(),
            ) - virtual_supply_xrd;
            let dex_xrd_supply = (max_xrd + virtual_supply_xrd)
                * (Decimal::ONE - listing_fee_perc - creator_fee_perc);

            let dex_token_supply = dex_xrd_supply * max_token_supply_to_trade / max_xrd_market_cap;
            let max_token_supply = max_token_supply_to_trade + dex_token_supply;
            let dex_tokens_ratio = dex_token_supply / (dex_token_supply + dex_xrd_supply);
            assert!(
                dex_tokens_ratio <= dec!("0.95") && dex_tokens_ratio >= dec!("0.05"),
                "Dex tokens ratio (dex tokens supply / (dex_xrd_supply + dex_tokens_supply)) must be between 0.05 and 0.95."
            );
            let last_price = curve_calcs.calculate_price(
                virtual_supply,
                max_xrd_market_cap,
                max_token_supply_to_trade,
            );

            let new_token_curve = RakoonFunTokenCurve {
                parent_address: parent_address.clone(),
                address: component_address.clone(),
                owner_badge_address: owner_badge.resource_address(),
                dapp_def_address,
                token_manager,
                current_supply: virtual_supply.clone(),
                max_token_supply,
                max_token_supply_to_trade,
                max_xrd_market_cap,
                max_xrd,
                virtual_supply,
                virtual_supply_xrd,
                tx_fee_perc,
                listing_fee_perc,
                creator_fee_perc,
                xrd_vault: Vault::new(XRD),
                token_vault: Vault::with_bucket(Bucket::from(virtual_supply_bucket)),
                fee_vault: Vault::new(XRD),
                creator_fee_vault: Vault::new(XRD),
                last_price,
                fair_launch_period_mins,
                in_fair_launch_period: if fair_launch_period_mins > 0 {true} else {false},
                fair_launch_receipt_manager,
                fair_launch_tokens: Vault::new(token_address.clone()),
                fair_launch_xrd: Decimal::ZERO,
                time_created: Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch,
                target_reached: 0,
                curve_calcs,
                dex_adaptor,
                dex_listing_address: None,
                listing_token_address: None,
                other_tokens: KeyValueStore::new(),
                xrd_traded: Decimal::ZERO,
                no_txs: 0,
                first_buy_vault: Vault::new(token_manager.address()),
                first_buy_claim: Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch + (first_buy_lock_mins * 60),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(
                owner_badge.resource_address()
            ))))
            .with_address(address_reservation)
            .roles(roles! {
                creator => rule!(require(owner_badge.resource_address()));
                rakoonfun_admin => parent_owner_rule.clone();
            })
            .metadata(metadata! {
                init {
                    "name" => format!("Rakoon.fun: {}", token_info.symbol.clone()), updatable;
                    "description" => format!("Rakoon.fun Token Bonding Curve component for token {} ({})", token_info.name.clone(), token_info.symbol.clone()), updatable;
                    "info_url" => Url::of(String::from("https://rakoon.fun")), updatable;
                    "social_urls" => social_urls_vec.clone(), updatable;
                    "tags" => vec!["RakoonFun"], updatable;
                    "dapp_definition" => dapp_def_address.clone(), updatable;
                }
            })
            .globalize();
            Runtime::emit_event(RakoonFunTokenCreateEvent {
                token_address: token_address.clone(),
                component_address: component_address.clone(),
                main_component: parent_address.clone(),
            });

            let (first_buy_tokens, first_buy_remaining) =
                new_token_curve.first_buy(first_buy_amount, first_buy_bucket, first_buy_lock_mins);

            (
                new_token_curve,
                Bucket::from(owner_badge),
                component_address,
                token_address,
                first_buy_tokens,
                first_buy_remaining,
            )
        }

        // function to buy tokens from the bonding curve using the sent XRD
        // function takes a bucket with XRD to use to buy new tokens
        // function returns a bucket with the bought tokens as well as a bucket with any remaining XRD (if any)
        pub fn buy(&mut self, mut in_bucket: Bucket) -> (Bucket, Bucket) {
            assert!(
                Runtime::get_tip_percentage() == 0,
                "Rakoon.fun does not allow adding tips to transactions."
            );
            assert!(
                in_bucket.resource_address() == XRD,
                "Can only buy tokens with XRD"
            );
            self.check_in_fair_launch_period();
            let mut xrd_amount = in_bucket.amount();
            // info!("XRD amount before fees: {}", xrd_amount);
            // info!("Max xrd: {}", self.max_xrd);
            let available_xrd = self.max_xrd - self.xrd_vault.amount();
            let mut fee_amount = xrd_amount * self.tx_fee_perc;
            if xrd_amount > available_xrd {
                // calculate fee based on available xrd only
                fee_amount = available_xrd * self.tx_fee_perc;
                xrd_amount = xrd_amount - fee_amount;
                if xrd_amount >= available_xrd {
                    xrd_amount = available_xrd;
                    self.target_reached =
                        Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
                }
            } else {
                xrd_amount = xrd_amount - fee_amount;
            };
            self.fee_vault.put(in_bucket.take(fee_amount));
            // info!("XRD Amount after fees: {}", xrd_amount);
            let mut out_bucket = if self.in_fair_launch_period {
                // create bucket with fair launch receipt address
                Bucket::new(self.fair_launch_receipt_manager.address())
            } else {
                Bucket::new(self.token_manager.address())
            };
            if xrd_amount > Decimal::ZERO {
                let mut receive_tokens = self.curve_calcs.calculate_tokens_received(
                    xrd_amount.clone(),
                    self.current_supply.clone(),
                    self.max_xrd_market_cap.clone(),
                    self.max_token_supply_to_trade.clone(),
                );
                if receive_tokens + self.current_supply - self.max_token_supply_to_trade
                    > Decimal::ZERO
                {
                    if receive_tokens + self.current_supply - self.max_token_supply_to_trade
                        > dec!("0.00000000000000001")
                    {
                        panic!("Unexpected error! Receive tokens calc error.");
                    }
                    receive_tokens = self.max_token_supply_to_trade - self.current_supply;
                    // TODO check the curve.calcs.calculate_tokens_received fn to make sure rounding works as expected
                    // although this should never happen, it might be happening due to a rounding error in the calculate tokens received fn.
                    // While I investigate this, I am just setting the receive tokens to the exact allowed amount.
                }
                let new_tokens = self.token_manager.mint(receive_tokens.clone());
                let new_tokens_amount = new_tokens.amount();
                if self.in_fair_launch_period {
                    // in fair launch period buyer receives a receipt that can be used to claim tokens after the fair launch period.
                    self.fair_launch_tokens.put(new_tokens.into());
                    self.fair_launch_xrd = self.fair_launch_xrd + xrd_amount;
                    out_bucket.put(
                        self.fair_launch_receipt_manager
                            .mint_ruid_non_fungible(FairLaunchReceiptData {
                                xrd_amount: xrd_amount.clone(),
                            })
                            .into(),
                    )
                } else {
                    out_bucket.put(new_tokens.into());
                }
                self.current_supply = self.current_supply + receive_tokens.clone();
                self.xrd_vault.put(in_bucket.take(xrd_amount));
                self.last_price = self.curve_calcs.calculate_price(
                    self.current_supply.clone(),
                    self.max_xrd_market_cap.clone(),
                    self.max_token_supply_to_trade.clone(),
                );
                self.xrd_traded = self.xrd_traded + xrd_amount;
                self.no_txs = self.no_txs + 1;
                Runtime::emit_event(RakoonFunTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("buy"),
                    fair_launch_period: self.in_fair_launch_period.clone(),
                    token_amount: new_tokens_amount.clone(),
                    xrd_amount: xrd_amount.clone(),
                    end_price: self.last_price.clone(),
                    total_xrd_traded: self.xrd_traded.clone(),
                    total_txs: self.no_txs.clone(),
                    main_component: self.parent_address.clone(),
                });
            }
            if self.target_reached > 0 {
                self.list_token();
            }
            (out_bucket, in_bucket)
        }

        // function to buy a specificly specified amount of tokens
        // the function takes in the specified value of tokens that must be bought as well as a bucket of XRD to pay for the tx
        // the function returns a bucket with the bought tokens as well as a bucket with any remaining XRD (if any)
        pub fn buy_amount(&mut self, amount: Decimal, mut in_bucket: Bucket) -> (Bucket, Bucket) {
            assert!(
                Runtime::get_tip_percentage() == 0,
                "Rakoon.fun does not allow adding tips to transactions."
            );
            assert!(
                in_bucket.resource_address() == XRD,
                "Can only buy tokens with XRD"
            );
            assert!(
                amount + self.current_supply <= self.max_token_supply_to_trade,
                "Cannot buy requested amount of tokens. Not enough supply left"
            );
            self.check_in_fair_launch_period();
            let mut out_bucket = if self.in_fair_launch_period {
                Bucket::new(self.fair_launch_receipt_manager.address())
            } else {
                Bucket::new(self.token_manager.address())
            };
            if amount > Decimal::ZERO {
                let mut xrd_required = self.curve_calcs.calculate_buy_price(
                    amount.clone(),
                    self.current_supply.clone(),
                    self.max_xrd_market_cap.clone(),
                    self.max_token_supply_to_trade.clone(),
                );
                // info!("Xrd required for buy_amount: {:?}", xrd_required);
                let fee_amount = xrd_required * self.tx_fee_perc;
                if xrd_required + fee_amount > in_bucket.amount() {
                    panic!("Not enough XRD sent for tx.");
                }
                if xrd_required + self.xrd_vault.amount() - self.max_xrd > Decimal::ZERO {
                    if xrd_required + self.xrd_vault.amount() - self.max_xrd
                        > dec!("0.00000000000000001")
                    {
                        panic!("Unexpected error! Max XRD will be exceeded in tx.")
                    }
                    xrd_required = self.max_xrd - self.xrd_vault.amount();
                    // TODO check curve_calcs.calculate_buy_price fn for rounding errors
                    // this condition should never be true
                }
                self.fee_vault.put(in_bucket.take(fee_amount));
                let new_tokens = self.token_manager.mint(amount.clone());
                let new_tokens_amount = new_tokens.amount();
                if self.in_fair_launch_period {
                    self.fair_launch_tokens.put(new_tokens.into());
                    self.fair_launch_xrd = self.fair_launch_xrd + xrd_required;
                    out_bucket.put(
                        self.fair_launch_receipt_manager
                            .mint_ruid_non_fungible(FairLaunchReceiptData {
                                xrd_amount: xrd_required.clone(),
                            })
                            .into(),
                    )
                } else {
                    out_bucket.put(new_tokens.into());
                }
                self.current_supply = self.current_supply + amount;
                self.xrd_vault.put(in_bucket.take(xrd_required));
                self.last_price = self.curve_calcs.calculate_price(
                    self.current_supply.clone(),
                    self.max_xrd_market_cap.clone(),
                    self.max_token_supply_to_trade.clone(),
                );
                if self.xrd_vault.amount() >= self.max_xrd {
                    self.target_reached =
                        Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch;
                    self.list_token();
                }
                self.xrd_traded = self.xrd_traded + xrd_required;
                self.no_txs = self.no_txs + 1;
                Runtime::emit_event(RakoonFunTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("buy"),
                    fair_launch_period: self.in_fair_launch_period.clone(),
                    token_amount: new_tokens_amount.clone(),
                    xrd_amount: xrd_required.clone(),
                    end_price: self.last_price.clone(),
                    total_xrd_traded: self.xrd_traded.clone(),
                    total_txs: self.no_txs,
                    main_component: self.parent_address.clone(),
                });
            }
            (out_bucket, in_bucket)
        }

        pub fn first_buy(
            &mut self,
            amount: Decimal,
            in_bucket: Option<Bucket>,
            lock_mins: i64,
        ) -> (Bucket, Bucket) {
            let mut remaining_bucket = Bucket::new(XRD);
            let mut first_buy_bucket = Bucket::new(self.token_manager.address());
            if in_bucket.is_some() {
                let (mut first_buy_tokens, first_buy_remaining) = self.buy_amount(
                    amount,
                    in_bucket
                        .expect("Unexpected error! First buy bucket should be some, but is none."),
                );
                if lock_mins > 0 {
                    self.first_buy_vault
                        .put(first_buy_tokens.take(first_buy_tokens.amount()));
                }
                first_buy_bucket.put(first_buy_tokens);
                remaining_bucket.put(first_buy_remaining);
            }
            (first_buy_bucket, remaining_bucket)
        }

        // function to sell the tokens provided
        // function takes in a bucket of tokens to sell
        // function returns a bucket of XRD from the sale as well as a bucket with any remaining tokens (if any)
        pub fn sell(&mut self, mut in_bucket: Bucket) -> (Bucket, Bucket) {
            assert!(
                Runtime::get_tip_percentage() == 0,
                "Rakoon.fun does not allow adding tips to transactions."
            );
            assert!(
                in_bucket.resource_address() == self.token_manager.address(),
                "Wrong tokens sent in bucket"
            );
            self.check_in_fair_launch_period();
            if self.in_fair_launch_period {
                panic!("Cannot sell tokens during fair launch period.")
            }
            let token_amount = in_bucket.amount();
            if token_amount > self.current_supply {
                panic!("Unexpected error! Sending more tokens to sell than current supply.");
            }
            let mut out_bucket = Bucket::new(XRD);
            if token_amount > Decimal::ZERO {
                let mut receive_xrd = self.curve_calcs.calculate_sell_price(
                    token_amount.clone(),
                    self.current_supply.clone(),
                    self.max_xrd_market_cap.clone(),
                    self.max_token_supply_to_trade.clone(),
                );
                if receive_xrd - self.xrd_vault.amount() > Decimal::ZERO {
                    if receive_xrd - self.xrd_vault.amount() > dec!("0.00000000000000001") {
                        panic!("Unexpected error! Not enough XRD in component for sell tx.")
                    }
                    receive_xrd = self.xrd_vault.amount();
                    // TODO curve_calcs.calculate_sell_price for rounding errors
                    // This should never happen, but currently might due to rounding errors in curve_calcs.calculate_sell_price
                }
                let fee_amount = receive_xrd * self.tx_fee_perc;
                self.fee_vault.put(self.xrd_vault.take(fee_amount));
                receive_xrd = receive_xrd - fee_amount;
                let burn_bucket = in_bucket.take(token_amount);
                burn_bucket.burn();
                self.current_supply = self.current_supply - token_amount.clone();
                out_bucket.put(self.xrd_vault.take(receive_xrd.clone()));
                self.last_price = self.curve_calcs.calculate_price(
                    self.current_supply.clone(),
                    self.max_xrd_market_cap.clone(),
                    self.max_token_supply_to_trade.clone(),
                );
                self.xrd_traded = self.xrd_traded + receive_xrd.clone() + fee_amount.clone();
                self.no_txs = self.no_txs + 1;
                Runtime::emit_event(RakoonFunTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("sell"),
                    fair_launch_period: self.in_fair_launch_period.clone(),
                    token_amount: token_amount.clone(),
                    xrd_amount: out_bucket.amount(),
                    end_price: self.last_price.clone(),
                    total_xrd_traded: self.xrd_traded.clone(),
                    total_txs: self.no_txs,
                    main_component: self.parent_address.clone(),
                });
            }
            (out_bucket, in_bucket)
        }

        // function to sell tokens to the value of the specified XRD amount
        // the function takes in the amount of XRD to receive as well as a bucket of tokens to sell
        // the function returns a bucket with XRD and a bucket with any remaining tokens (if any)
        pub fn sell_for_xrd_amount(
            &mut self,
            amount: Decimal,
            mut in_bucket: Bucket,
        ) -> (Bucket, Bucket) {
            assert!(
                Runtime::get_tip_percentage() == 0,
                "Rakoon.fun does not allow adding tips to transactions."
            );
            assert!(
                in_bucket.resource_address() == self.token_manager.address(),
                "Wrong tokens sent in bucket"
            );
            self.check_in_fair_launch_period();
            if self.in_fair_launch_period {
                panic!("Cannot sell tokens during fair launch period.")
            }
            let fee_amount = amount * self.tx_fee_perc;
            if amount + fee_amount > self.xrd_vault.amount() {
                panic!("Not enough XRD in component vault for requested amount.");
            }
            self.fee_vault.put(self.xrd_vault.take(fee_amount));
            let mut out_bucket = Bucket::new(XRD);
            info!("In bucket amount: {:?}", in_bucket.amount());
            if amount > Decimal::ZERO {
                let mut tokens_to_sell = self.curve_calcs.calculate_tokens_to_sell(
                    amount.clone() + fee_amount.clone(),
                    self.current_supply.clone(),
                    self.max_xrd_market_cap.clone(),
                    self.max_token_supply_to_trade.clone(),
                );
                info!("Tokens required: {:?}", tokens_to_sell);
                if tokens_to_sell > in_bucket.amount() {
                    panic!("Not enough tokens supplied for required amount of XRD");
                }
                if tokens_to_sell - self.current_supply > Decimal::ZERO {
                    if tokens_to_sell - self.current_supply > dec!("0.00000000000000001") {
                        panic!("Unexpected error! Not enough token supply in component to sell.");
                    }
                    tokens_to_sell = self.current_supply;
                    // TODO check curve_calcs.calculate_tokens_to_sell for rounding errors
                }
                let burn_bucket = in_bucket.take(tokens_to_sell.clone());
                burn_bucket.burn();
                self.current_supply = self.current_supply - tokens_to_sell;
                out_bucket.put(self.xrd_vault.take(amount.clone()));
                self.last_price = self.curve_calcs.calculate_price(
                    self.current_supply.clone(),
                    self.max_xrd_market_cap.clone(),
                    self.max_token_supply_to_trade.clone(),
                );
                self.xrd_traded = self.xrd_traded + amount.clone() + fee_amount.clone();
                self.no_txs = self.no_txs + 1;
                Runtime::emit_event(RakoonFunTokenTradeEvent {
                    token_address: self.token_manager.address(),
                    side: String::from("sell"),
                    fair_launch_period: self.in_fair_launch_period.clone(),
                    token_amount: tokens_to_sell.clone(),
                    xrd_amount: out_bucket.amount(),
                    end_price: self.last_price.clone(),
                    total_xrd_traded: self.xrd_traded.clone(),
                    total_txs: self.no_txs,
                    main_component: self.parent_address.clone(),
                });
            }
            (out_bucket, in_bucket)
        }

        // function to claim tokens allocated during fair launch period
        pub fn claim_fair_launch_tokens(&mut self, receipts_bucket: Bucket) -> Bucket {
            let mut out_bucket = Bucket::new(self.token_manager.address());
            let mut total_xrd = Decimal::ZERO;
            self.check_in_fair_launch_period();
            assert!(!self.in_fair_launch_period, "Fair launch period not finished. Fair launch tokens can only be claimed once fair launch period has finished.");
            assert!(
                receipts_bucket.resource_address() == self.fair_launch_receipt_manager.address(),
                "Incorrect tokens submitted for claim."
            );
            for receipt in receipts_bucket
                .as_non_fungible()
                .non_fungibles::<FairLaunchReceiptData>()
            {
                let receipt_data = receipt.data();
                total_xrd = total_xrd + receipt_data.xrd_amount;
                let mut claim_tokens = self.fair_launch_tokens.amount() * receipt_data.xrd_amount
                    / self.fair_launch_xrd;
                if claim_tokens > self.fair_launch_tokens.amount() {
                    claim_tokens = self.fair_launch_tokens.amount();
                }
                out_bucket.put(self.fair_launch_tokens.take(claim_tokens));
                self.fair_launch_xrd = self.fair_launch_xrd - receipt_data.xrd_amount;
            }
            receipts_bucket.burn();
            Runtime::emit_event(RakoonFunClaimTokensEvent {
                tokens_claimed: out_bucket.amount(),
                xrd_amount: total_xrd.clone(),
                token_address: self.token_manager.address(),
                main_component: self.parent_address.clone(),
            });
            out_bucket
        }

        pub fn claim_first_buy_tokens(&mut self) -> Bucket {
            assert!(
                Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch
                    > self.first_buy_claim,
                "Owner bought tokens cannot be claimed yet."
            );
            self.first_buy_vault.take_all()
        }

        pub fn claim_creator_fees(&mut self) -> Bucket {
            Runtime::emit_event(RakoonFunClaimCreatorFeeEvent {
                fee_claimed: self.fee_vault.amount(),
                token_address: self.token_manager.address(),
                main_component: self.parent_address.clone(),
            });
            self.creator_fee_vault.take_all()
        }

        pub fn claim_fees(&mut self) -> Bucket {
            Runtime::emit_event(RakoonFunClaimTokenFeeEvent {
                fee_claimed: self.fee_vault.amount(),
                token_address: self.token_manager.address(),
                main_component: self.parent_address.clone(),
            });
            self.fee_vault.take_all()
        }

        pub fn change_curve_calcs(&mut self, curve_calcs_component_address: ComponentAddress) {
            let new_curve_calcs_component = CurveCalcsAdaptor::from(curve_calcs_component_address);
            Runtime::emit_event(RakoonFunChangeTokenCurveCalcsEvent {
                old_curve_calc_component: self.curve_calcs.address(),
                new_curve_calc_component: curve_calcs_component_address.clone(),
                token_address: self.token_manager.address(),
                main_component: self.parent_address.clone(),
            });
            self.curve_calcs = new_curve_calcs_component;
        }

        pub fn set_dex_adaptor(&mut self, dex_adaptor_component_address: ComponentAddress) {
            assert!(self.dex_listing_address.is_none(), "Cannot change dex adaptor for token that is already listed. Need to remove listing before changing dex adaptor.");
            let new_dex_adaptor = DexAdaptor::from(dex_adaptor_component_address);
            Runtime::emit_event(RakoonFunChangeTokenDexAdaptorEvent {
                old_dex_adaptor_component: if let Some(adaptor) = self.dex_adaptor {
                    Some(adaptor.address())
                } else {
                    None
                },
                new_dex_adaptor_component: dex_adaptor_component_address.clone(),
                token_address: self.token_manager.address(),
                main_component: self.parent_address.clone(),
            });
            self.dex_adaptor = Some(new_dex_adaptor);
        }

        // method to launch the token on a DEX
        pub fn list_token(&mut self) {
            info!("Token will be listed!");
            assert!(
                self.target_reached > 0,
                "Cannot list token before target is reached."
            );
            assert!(
                self.dex_listing_address.is_none(),
                "Token is already listed."
            );
            assert!(
                self.dex_adaptor.is_some(),
                "Token must have a dex adaptor set in order to list the token."
            );
            let mut dex_adaptor = self.dex_adaptor.unwrap();
            let xrd_for_listing = (self.max_xrd + self.virtual_supply_xrd)
                * (Decimal::ONE - self.listing_fee_perc - self.creator_fee_perc);
            let available_for_fees = self.xrd_vault.amount() - xrd_for_listing.clone();
            let creator_fee = available_for_fees * self.creator_fee_perc
                / (self.listing_fee_perc + self.creator_fee_perc);
            let listing_fee = available_for_fees - creator_fee;
            self.creator_fee_vault.put(self.xrd_vault.take(creator_fee));
            self.fee_vault.put(self.xrd_vault.take(listing_fee));
            let xrd_bucket = self.xrd_vault.take_all();
            let mut tokens_bucket = Bucket::new(self.token_manager.address());
            let tokens_to_mint =
                self.max_token_supply - self.current_supply - self.token_vault.amount();
            if tokens_to_mint > Decimal::ZERO {
                tokens_bucket.put(self.token_manager.mint(tokens_to_mint).into());
            }
            tokens_bucket.put(self.token_vault.take_all());
            self.current_supply = self
                .token_manager
                .total_supply()
                .expect("Could not get token supply before listing");
            let (listing_component_address, listing_tokens, other_tokens) =
                dex_adaptor.create_listing(tokens_bucket, xrd_bucket);
            self.dex_listing_address = Some(listing_component_address);
            self.listing_token_address = Some(listing_tokens.resource_address());
            self.insert_other_token(listing_tokens);
            for other_token_bucket in other_tokens {
                if other_token_bucket.resource_address() == XRD {
                    self.xrd_vault.put(other_token_bucket);
                } else if other_token_bucket.resource_address() == self.token_manager.address() {
                    self.token_vault.put(other_token_bucket);
                } else {
                    self.insert_other_token(other_token_bucket);
                }
            }
            Runtime::emit_event(RakoonFunDexEvent {
                event_type: String::from("List on DEX"),
                pool_address: self.dex_listing_address.clone().unwrap(),
                token_address: self.token_manager.address(),
                main_component: self.parent_address.clone(),
            });
        }

        // method to remove all liquidity and allow the token to be listed somewhere else
        pub fn remove_listing(&mut self) {
            assert!(
                self.dex_listing_address.is_some(),
                "Token is not listed yet."
            );
            assert!(
                self.dex_adaptor.is_some(),
                "Token must have a dex adaptor set."
            );
            let old_pool_address = self.dex_listing_address.clone().unwrap();
            self.remove_liquidity();
            self.dex_listing_address = None;
            Runtime::emit_event(RakoonFunDexEvent {
                event_type: String::from("Delist from DEX"),
                pool_address: old_pool_address,
                token_address: self.token_manager.address(),
                main_component: self.parent_address.clone(),
            });
        }

        // method to remove all liquidity form the token's dex pool.
        fn remove_liquidity(&mut self) {
            assert!(
                self.dex_listing_address.is_some(),
                "Token is not listed yet."
            );
            assert!(
                self.dex_adaptor.is_some(),
                "Token must have a dex adaptor set."
            );

            let mut dex_adaptor = self.dex_adaptor.unwrap();
            let listing_tokens = self
                .other_tokens
                .get_mut(
                    &self
                        .listing_token_address
                        .expect("Listed token should have a valid listing token."),
                )
                .expect("Could not find vault for listing tokens.")
                .take_all();
            let (token_bucket, xrd_bucket, other_tokens) =
                dex_adaptor.remove_liquidity(self.dex_listing_address.unwrap(), listing_tokens);
            self.token_vault.put(token_bucket);
            self.xrd_vault.put(xrd_bucket);
            for other_token_bucket in other_tokens {
                if other_token_bucket.resource_address() == XRD {
                    self.xrd_vault.put(other_token_bucket);
                } else if other_token_bucket.resource_address() == self.token_manager.address() {
                    self.token_vault.put(other_token_bucket);
                } else {
                    self.insert_other_token(other_token_bucket);
                }
            }
        }

        fn check_in_fair_launch_period(&mut self) {
            if self.in_fair_launch_period
                && Clock::current_time_rounded_to_seconds().seconds_since_unix_epoch
                    > self.time_created
                        + (self
                            .fair_launch_period_mins
                            .to_i64()
                            .expect("Could not convert fair_launch_period_mins to i64")
                            * 60)
            {
                self.end_fair_launch_period();
            }
        }

        fn end_fair_launch_period(&mut self) {
            self.in_fair_launch_period = false;
        }

        fn insert_other_token(&mut self, other_token_bucket: Bucket) {
            let other_token_address = other_token_bucket.resource_address();
            if self.other_tokens.get(&other_token_address).is_some() {
                let mut existing_vault = self.other_tokens.get_mut(&other_token_address).unwrap();
                existing_vault.put(other_token_bucket);
            } else {
                self.other_tokens.insert(
                    other_token_address.clone(),
                    Vault::with_bucket(other_token_bucket),
                )
            }
        }
    }
}

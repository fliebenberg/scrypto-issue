use crate::rakoonfun_token_curve::rakoonfun_token_curve::{
    RakoonFunTokenCurve, RakoonFunTokenCurveFunctions,
};
use crate::rakoonfun_token_curve::TokenInfo;
use curve_calcs_interface::curve_calcs_interface::*;
use curve_cs2_v1::curve_cs2_v1::*;
use dex_interface::dex_interface::*;

use scrypto::prelude::*;

#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunChangeDefaultEvent {
    field_name: String, // the name of the field that had a change in value
    old_value: String,  // the value of the field before the change
    new_value: String,  // the value of the field after the change
    main_component: ComponentAddress, // the main component address - used to distinguish between events from different versions of Rakoon.fun
}
#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
struct RakoonFunClaimFeeEvent {
    fee_claimed: Decimal,             // the amount of fee claimed
    main_component: ComponentAddress, // the main component address - used to distinguish between events from different versions of Rakoon.fun
}

type CurveCalcsAdaptor = CurveCalcsAdaptorInterfaceScryptoStub;
type DexAdaptor = DexAdaptorInterfaceScryptoStub;

#[blueprint]
#[events(RakoonFunClaimFeeEvent, RakoonFunChangeDefaultEvent)]
// the rakoonfun_main component is the overall "parent" component for all Rakoon.fun components.
// It is used to launch new token components and keeps a kvs with the component and resource addresses of all token components.
// It also allows for the management of default values used to set up new tokens and the collection of fees for token creation.
mod rakoonfun_main {
    enable_function_auth! {
        new => AccessRule::AllowAll;
    }

    enable_method_auth! {
        roles {
            admin => updatable_by: [OWNER]; // an admin can manage settings on the component
            owner => updatable_by: [OWNER]; // an owner can claim fees on the component
        },
        methods {
            new_token_curve_advanced => PUBLIC;
            new_token_curve_simple => PUBLIC;
            change_default_parameters => restrict_to: [admin];
            change_default_parameter => restrict_to: [admin];
            claim_fee_amount => restrict_to: [owner];
            claim_all_fees => restrict_to:[owner];
            transfer_fees => PUBLIC;
            update_curve_calc => restrict_to: [admin];
            delete_curve_calc => restrict_to: [admin];
            update_dexs => restrict_to: [admin];
            delete_dex => restrict_to: [admin];
            get_dex_adaptor => PUBLIC;
        }
    }

    struct RakoonFunMain {
        pub address: ComponentAddress, // the address of this component - included for easy reference
        pub owner_badge_manager: ResourceManager, // the resource manager for the owner badge - created as part of the component instantiation.
        pub max_token_supply_to_trade: Decimal, // the default maximum token supply available for trading on the bonding curve
        pub max_xrd_market_cap: Decimal, // the default maximum market cap in XRD that will be reached when the max tokens have been traded on the bonding curve
        pub virtual_supply: Decimal, // the default virtual supply that will be created to avoid the flat part of the curve for a token
        pub tokens: KeyValueStore<ComponentAddress, ResourceAddress>, // a simple list of the tokens launched - (token component address, token resource address)
        pub tx_fee_perc: Decimal, // fee % taken on every tx, specified in decimals 1% = 0.01
        pub listing_fee_perc: Decimal, // fee % paid to Rakoon.fun when a token is listed on a dex, specified in decimals 1% = 0.01
        pub creator_fee_perc: Decimal, // fee % paid to the token creator when a token is listed on a dex, specified in decimals 1% = 0.01
        pub token_creation_fee: Decimal, // XRD fee for creating a token - might be needed for spam protection
        pub fees_vault: Vault,           // vault to hold fees
        pub fair_launch_period_mins: u32, // the number of minutes for a fair launch period
        pub curve_calcs: KeyValueStore<String, CurveCalcsAdaptor>, // the list of available curve calcs adaptors - (name, component address)
        pub dexs: KeyValueStore<String, DexAdaptor>, // a list of the available DEX adaptors - (name, component address)
        pub default_curve: String, // the name (key in curve_calcs) of the default curve to use when setting up token components.
        pub first_buy_max_perc: Decimal, // the maximum percentage of trade curve tokens a creator can buy when creating a token
        pub first_buy_lock_mins: i64, // the number of mins that first_buy tokens are locked before the creator can claim them
    }

    impl RakoonFunMain {
        // function to create a new RakoonFunMain (parent) instance. This instance is used to create and track individual RRakoon.fun token curves.
        pub fn new(
            name: String, // the name of the component - used to set up the dapp definition account and will be displayed in wallet with txs. Can be changed later by admin.
            description: String, // a description for the component - used to set up the dapp definition account.
            info_url: String,    // a url to the Rakoon.fun website.
            icon_url: String,    // a url for the icon of the component
            max_token_supply_to_trade: Decimal, // the default token supply available to trade when setting up a new token curve component - does not include the supply that will be minted when the token gets listed on a dex.
            max_xrd_market_cap: Decimal, // the default maximum XRD market cap when setting up a new token curve component - it is the XRD market cap that will be reached when the token reaches max_token_supply_to_trade.
            virtual_supply: Decimal, // the default virtual supply that will be created to avoid the flat part of the curve for a token
            fair_launch_period_mins: u32, // the default number of minutes that the fair launch period will last when setting up a new token curve component.
            tx_fee_perc: Decimal, // the default percentage transaction fee when setting up a new token curve component - tx fee percentage will be charged on every token curve buy/sell tx.
            listing_fee_perc: Decimal, // the default percentage listing fee when setting up a new token curve component - listing fee percentage is a Rakoon.fun fee that will be charged when the token is listed on a dex.
            creator_fee_perc: Decimal, // the default percentage creator fee when setting up a new token curve component - creator fee percentage is a fee for the token creator what will be charged when the token is listed on a dex.
            token_creation_fee: Decimal, // the default creation fee amount when setting up a new token curve component - token creation fee is a Rakoon.fun fee that is paid by the creator when a new token curve component is created.
            first_buy_max_perc: Decimal, // the maximum percentage of trade curve tokens a creator can buy when creating a token
            first_buy_lock_mins: i64, // the minimum number of mins that first_buy tokens are locked before the creator can claim them
            owner_badge_address: ResourceAddress, // resource address of the owner badge
        ) -> Global<RakoonFunMain> {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(<RakoonFunMain>::blueprint_id());
            let dapp_def_account =
                Blueprint::<Account>::create_advanced(OwnerRole::Updatable(rule!(allow_all)), None); // will reset owner role after dapp def metadata has been set
            dapp_def_account.set_metadata("account_type", String::from("dapp definition"));
            dapp_def_account.set_metadata("name", name.clone());
            dapp_def_account.set_metadata("description", description.clone());
            dapp_def_account.set_metadata("info_url", Url::of(info_url.clone()));
            dapp_def_account.set_metadata("icon_url", Url::of(icon_url.clone()));
            dapp_def_account.set_metadata(
                "claimed_entities",
                vec![GlobalAddress::from(component_address.clone())],
            );
            dapp_def_account.set_owner_role(rule!(require(owner_badge_address)));
            let dapp_def_address = GlobalAddress::from(dapp_def_account.address());

            let curve_calcs = KeyValueStore::<String, CurveCalcsAdaptor>::new();
            let new_curve_calcs = CurveCs2Calcs::new(
                OwnerRole::Updatable(rule!(require(owner_badge_address.clone()))),
                None,
            );
            let default_curve = String::from("cs2");
            curve_calcs.insert(default_curve.clone(), new_curve_calcs.address().into());
            info!(
                "Virtual supply set for main component {:?}",
                virtual_supply.clone()
            );
            RakoonFunMain {
                address: component_address,
                owner_badge_manager: ResourceManager::from_address(owner_badge_address.clone()),
                max_token_supply_to_trade,
                max_xrd_market_cap,
                virtual_supply,
                tx_fee_perc,
                listing_fee_perc,
                creator_fee_perc,
                token_creation_fee,
                tokens: KeyValueStore::new(),
                fees_vault: Vault::new(XRD),
                fair_launch_period_mins,
                curve_calcs,
                dexs: KeyValueStore::new(),
                default_curve,
                first_buy_max_perc,
                first_buy_lock_mins,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(rule!(require(
                owner_badge_address.clone()
            ))))
            .with_address(address_reservation)
            .roles(roles! {
                admin => rule!(require(owner_badge_address.clone()));
                owner => rule!(require(owner_badge_address.clone()));
            })
            .metadata(metadata! {
                init {
                "name" => name, updatable;
                "description" => description, updatable;
                "info_url" => Url::of(info_url), updatable;
                "tags" => vec!["Rakoon.fun", "Token", "Meme", "Launcher"], updatable;
                "dapp_definition" => dapp_def_address.clone(), updatable;
                }
            })
            .globalize()
        }

        // function to create an individual token bonding curve component
        // takes in values used to set up the new token and its bonding curve component
        // returns a global instance of the new component, an owner badge for the token, any remaing XRD from paying the fee and remaining XRD from creator buying tokens.
        pub fn new_token_curve_advanced(
            &mut self,
            name: String,
            symbol: String,
            description: String,
            icon_url: String,
            telegram_url: String,
            x_url: String,
            website_url: String,
            curve: String,
            dex: String,
            max_token_supply_to_trade: Decimal,
            max_xrd_market_cap_to_trade: Decimal,
            virtual_supply: Decimal,
            creator_fee_perc: Decimal,
            fair_launch_period_mins: u32,
            first_buy_amount: Decimal,
            first_buy_bucket: Option<Bucket>,
            first_buy_lock_mins: i64,
            mut fee_bucket: Bucket,
        ) -> (Global<RakoonFunTokenCurve>, Bucket, Bucket, Bucket) {
            info!(
                "main component virtual supply for advanced new token: {:?}",
                virtual_supply
            );
            if self.token_creation_fee > Decimal::ZERO {
                assert!(
                    fee_bucket.resource_address() == XRD,
                    "Only XRD can be sent for fees."
                );
                assert!(
                    fee_bucket.amount() >= self.token_creation_fee,
                    "Not enough XRD sent for token creation fee."
                );
                self.fees_vault
                    .put(fee_bucket.take(self.token_creation_fee));
            }
            let token_curve_calcs = self
                .curve_calcs
                .get(&curve)
                .expect("Could not find curve in RakoonFunMain curve_calcs.")
                .clone();
            let dex_adaptor = if dex.len() == 0 {
                None
            } else {
                Some(
                    self.dexs
                        .get(&dex)
                        .expect("Could not find dex in RakoonFunMain dex adaptors.")
                        .clone(),
                )
            };

            assert!(
                name.len() > 0 && symbol.len() > 0,
                "Both name and symbol must be specified."
            );
            assert!(
                first_buy_amount == Decimal::ZERO
                    || first_buy_amount <= self.first_buy_max_perc * max_token_supply_to_trade,
                "Creator buy amount too high."
            );
            assert!(
                first_buy_lock_mins >= self.first_buy_lock_mins,
                "Creator token lock period too short."
            );

            let (
                new_instance,
                owner_badge,
                component_address,
                token_address,
                first_buy_tokens_bucket,
                first_buy_remaining_bucket,
            ) = Blueprint::<RakoonFunTokenCurve>::new(
                TokenInfo {
                    name,
                    symbol,
                    description,
                    icon_url,
                    telegram_url,
                    x_url,
                    website_url,
                },
                max_token_supply_to_trade,
                max_xrd_market_cap_to_trade,
                virtual_supply,
                self.tx_fee_perc.clone(),
                self.listing_fee_perc.clone(),
                creator_fee_perc,
                fair_launch_period_mins,
                self.address.clone(),
                rule!(require(self.owner_badge_manager.address())),
                token_curve_calcs,
                dex_adaptor,
                first_buy_amount,
                first_buy_bucket,
                first_buy_lock_mins,
            );
            let mut remaining_xrd_bucket = Bucket::new(XRD);
            remaining_xrd_bucket.put(fee_bucket);
            remaining_xrd_bucket.put(first_buy_remaining_bucket);
            self.tokens.insert(component_address.clone(), token_address);
            (
                new_instance,
                owner_badge,
                remaining_xrd_bucket,
                first_buy_tokens_bucket,
            )
        }

        // function to create an individual token bonding curve component
        // takes in values used to set up the new token and its bonding curve component
        // returns a global instance of the new component as well as an owner badge for the token.
        pub fn new_token_curve_simple(
            &mut self,
            name: String,
            symbol: String,
            description: String,
            icon_url: String,
            telegram: String,
            x: String,
            website: String,
            curve: String,
            dex: String,
            first_buy_amount: Decimal,
            first_buy_bucket: Option<Bucket>,
            fee_bucket: Bucket,
        ) -> (Global<RakoonFunTokenCurve>, Bucket, Bucket, Bucket) {
            info!("simple curve virtual supply: {:?}", self.virtual_supply);
            self.new_token_curve_advanced(
                name,
                symbol,
                description,
                icon_url,
                telegram,
                x,
                website,
                curve,
                dex,
                self.max_token_supply_to_trade,
                self.max_xrd_market_cap,
                self.virtual_supply,
                self.creator_fee_perc,
                self.fair_launch_period_mins,
                first_buy_amount,
                first_buy_bucket,
                self.first_buy_lock_mins,
                fee_bucket,
            )
        }

        pub fn change_default_parameters(&mut self, param_values: Vec<(String, String)>) {
            for (param_name, param_value) in param_values {
                self.change_default_parameter(param_name, param_value);
            }
        }

        pub fn change_default_parameter(&mut self, param_name: String, param_value: String) {
            let old_value: String;
            let new_value = param_value.clone();
            match param_name.as_str() {
                "max_token_supply_to_trade" => {
                    old_value = self.max_token_supply_to_trade.to_string();
                    self.max_token_supply_to_trade = Decimal::try_from(
                        param_value,
                    ).expect(
                        "Could not convert parameter value for max_token_supply_to_trade to Decimal",
                    );
                }
                "max_xrd_market_cap" => {
                    old_value = self.max_xrd_market_cap.to_string();
                    self.max_xrd_market_cap = Decimal::try_from(param_value).expect(
                        "Could not convert parameter value for max_xrd_market_cap to Decimal",
                    )
                }
                "virtual_supply" => {
                    old_value = self.virtual_supply.to_string();
                    self.virtual_supply = Decimal::try_from(param_value)
                        .expect("Could not convert parameter value for virtual_supply to Decimal")
                }
                "tx_fee_perc" => {
                    old_value = self.tx_fee_perc.to_string();
                    self.tx_fee_perc = Decimal::try_from(param_value)
                        .expect("Could not convert parameter value for tx_fee_perc to Decimal")
                }
                "listing_fee_perc" => {
                    old_value = self.listing_fee_perc.to_string();
                    self.listing_fee_perc = Decimal::try_from(param_value)
                        .expect("Could not convert parameter value for listing_fee_perc to Decimal")
                }
                "creator_fee_perc" => {
                    old_value = self.creator_fee_perc.to_string();
                    self.creator_fee_perc = Decimal::try_from(param_value)
                        .expect("Could not convert parameter value for creator_fee_perc to Decimal")
                }
                "token_creation_fee" => {
                    old_value = self.token_creation_fee.to_string();
                    self.token_creation_fee = Decimal::try_from(param_value).expect(
                        "Could not convert parameter value for token_creation_fee to Decimal",
                    )
                }
                "fair_launch_period_mins" => {
                    old_value = self.fair_launch_period_mins.to_string();
                    self.fair_launch_period_mins = param_value.parse().expect(
                        "Could not convert parameter value for fair_launch_period_mins to u32",
                    );
                }
                "default_curve" => {
                    old_value = self.default_curve.clone();
                    // check if curve exists in curve_calcs
                    self.curve_calcs
                        .get(&param_value)
                        .expect("Could not find curve in existing curve_calcs");
                    self.default_curve = param_value;
                }
                "first_buy_max_perc" => {
                    old_value = self.first_buy_max_perc.to_string();
                    self.first_buy_max_perc = Decimal::try_from(param_value).expect(
                        "Could not convert parameter value for first_buy_max_perc to Decimal",
                    );
                }
                "first_buy_lock_mins" => {
                    old_value = self.first_buy_lock_mins.to_string();
                    self.first_buy_lock_mins = param_value
                        .parse()
                        .expect("Could not convert parameter value for first_buy_lock_mins to i64");
                }
                _ => panic!("Could not match parameter name"),
            };
            Runtime::emit_event(RakoonFunChangeDefaultEvent {
                field_name: param_name.clone(),
                old_value,
                new_value,
                main_component: self.address.clone(),
            });
        }

        pub fn claim_fee_amount(&mut self, amount: Decimal) -> Bucket {
            assert!(
                amount <= self.fees_vault.amount(),
                "Not enough fees in vault."
            );
            let out_bucket = self.fees_vault.take(amount);
            Runtime::emit_event(RakoonFunClaimFeeEvent {
                fee_claimed: out_bucket.amount(),
                main_component: self.address.clone(),
            });
            out_bucket
        }

        pub fn claim_all_fees(&mut self) -> Bucket {
            Runtime::emit_event(RakoonFunClaimFeeEvent {
                fee_claimed: self.fees_vault.amount(),
                main_component: self.address.clone(),
            });
            self.fees_vault.take_all()
        }

        pub fn transfer_fees(&mut self, in_bucket: Bucket) {
            assert!(
                in_bucket.resource_address() == XRD,
                "Can only transfer XRD fees to the RakoonFunMain component."
            );
            self.fees_vault.put(in_bucket);
        }

        pub fn update_curve_calc(
            &mut self,
            curve_calcs_name: String,
            curve_calcs_component_address: ComponentAddress,
        ) {
            let curve_calcs_component = CurveCalcsAdaptor::from(curve_calcs_component_address);
            self.curve_calcs
                .insert(curve_calcs_name, curve_calcs_component);
        }

        pub fn delete_curve_calc(&mut self, curve_calcs_name: String) {
            self.curve_calcs.remove(&curve_calcs_name);
        }

        pub fn update_dexs(&mut self, dex_name: String, dex_adaptor_address: ComponentAddress) {
            assert!(
                dex_name.len() > 0,
                "Must specify a name for the dex adaptor"
            );
            let dex_adaptor = DexAdaptor::from(dex_adaptor_address);
            self.dexs.insert(dex_name, dex_adaptor);
        }

        pub fn delete_dex(&mut self, dex_name: String) {
            self.dexs.remove(&dex_name);
        }

        pub fn get_dex_adaptor(&self, dex_name: String) -> DexAdaptor {
            self.dexs
                .get(&dex_name)
                .expect("Could not find Dex")
                .clone()
        }
    }
}

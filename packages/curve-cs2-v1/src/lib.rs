use curve_calcs_interface::curve_calcs_interface::*;
use scrypto::prelude::*;
use scrypto_interface::*;

fn calculate_multiplier(
    max_xrd_market_cap: Decimal,
    max_token_supply_to_trade: Decimal,
) -> PreciseDecimal {
    let divisor = PreciseDecimal::from(max_token_supply_to_trade)
        .checked_powi(3)
        .expect("Problem in calculating multiplier. powi(3)");
    let multiplier = PreciseDecimal::from(max_xrd_market_cap)
        .checked_div(divisor)
        .expect("Problem in calculating multiplier. First div");
    multiplier
}

#[blueprint_with_traits]
mod curve_cs2_v1 {
    struct CurveCs2Calcs {
        name: String,
        address: ComponentAddress,
    }

    impl CurveCs2Calcs {
        pub fn new(
            owner_role: OwnerRole,
            address_reservation_option: Option<GlobalAddressReservation>,
        ) -> Global<CurveCs2Calcs> {
            // let (address_reservation, component_address) =
            //     address_reservation.unwrap_or_else(|| {
            //         Runtime::allocate_component_address(<CurveCs2Calcs>::blueprint_id())
            //     });
            let (address_reservation, component_address) = if let Some(
                provided_address_reservation,
            ) = address_reservation_option
            {
                (
                        provided_address_reservation.clone(),
                        ComponentAddress::try_from(Runtime::get_reservation_address(&provided_address_reservation)).expect("Could not convert CurveCs2Calcs address reservation into valid component address"),
                    )
            } else {
                Runtime::allocate_component_address(<CurveCs2Calcs>::blueprint_id())
            };
            Self{
                name: String::from("cs2"),
                address: component_address
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .metadata(metadata! {
                    init {
                    "name" => String::from("Rakoon.fun Curve Calcs: cs2"), updatable;
                    "description" => String::from("Rakoon.fun Curve calcs for curve: price = constant * (supply ^ 2)"), updatable;
                    "tags" => vec!["Rakoon.fun", "Token", "Meme", "Launcher"], updatable;
                    }
                })
            .with_address(address_reservation)
            .globalize()
        }
    }

    impl CurveCalcsAdaptorInterfaceTrait for CurveCs2Calcs {
        fn address(&self) -> ComponentAddress {
            self.address
        }

        fn calculate_price(
            &self,
            supply: Decimal,
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal,
        ) -> Decimal {
            let multiplier = calculate_multiplier(max_xrd_market_cap, max_token_supply_to_trade);
            Decimal::try_from(
                multiplier.clone()
                    * PreciseDecimal::from(supply.clone())
                        .checked_powi(2)
                        .expect("calculate_price problem. powi(2)"),
            )
            .expect("calculate_price problem. Cant convert precise decimal to decimal.")
        }

        // pure function to calculate the buy price (XRD required) in order to receive a specified amount of new tokens
        fn calculate_buy_price(
            &self,
            new_tokens: Decimal, // the amount of tokens to buy
            supply: Decimal,     // the supply of tokens before the buy transaction
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal,
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            // info!("Tokens to buy: {:?}", new_tokens);
            let multiplier = calculate_multiplier(max_xrd_market_cap, max_token_supply_to_trade);
            // info!("calc_buy_price multiplier: {:?}", multiplier);
            if new_tokens > Decimal::ZERO {
                let precise_supply = PreciseDecimal::from(supply.clone());
                // info!("Supply: {:?}", supply);
                let first_value: PreciseDecimal = multiplier
                    .checked_div(3)
                    .expect("calculate_buy_price problem. Div 3");
                // info!("First value: {:?}", first_value);
                let second_value = (precise_supply + new_tokens.clone())
                    .checked_powi(3)
                    .expect("calculate_buy_price problem. First Powi(3).");
                // info!("Second value: {:?}", second_value);
                let third_value = precise_supply
                    .checked_powi(3)
                    .expect("calculate_buy_price problem. Second Powi(3).");
                // info!("Third value: {:?}", third_value);
                let fourth_value = second_value - third_value;
                // info!("fourth value: {:?}", fourth_value);
                let precise_price = first_value
                    .checked_mul(fourth_value)
                    .expect("calculate_buy_price problem. Final Multiply.");
                // info!("Final value: {:?}", precise_price);
                result = Decimal::try_from(
                    precise_price
                        .checked_round(18, RoundingMode::ToNearestMidpointAwayFromZero)
                        .expect("calculate_buy_price problem. Cant round precise decimal."),
                )
                .expect("calculate_buy_price problem. Cant convert precise decimal to decimal.")
            }
            result
        }

        // pure function to calculate how many tokens can be bought with the specified amount of XRD
        fn calculate_tokens_received(
            &self,
            xrd_received: Decimal, // the amount of XRD to spend to buy tokens
            supply: Decimal,       // the supply of tokens before the buy transaction
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal,
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            let multiplier = calculate_multiplier(max_xrd_market_cap, max_token_supply_to_trade);
            if xrd_received > Decimal::ZERO {
                // info!("Miltiplier: {}", multiplier);
                let precise_xrd_received = PreciseDecimal::from(xrd_received.clone());
                // info!("XRD Received: {}", precise_xrd_received);
                let precise_supply = PreciseDecimal::from(supply.clone());
                // info!("Supply: {}", precise_supply);
                let mut first_value = precise_xrd_received
                    .checked_div(multiplier.clone())
                    .expect("calculate_tokens_received problem. First div");
                first_value = first_value
                    .checked_mul(3)
                    .expect("calculate_tokens_received problem. First mul");
                // info!("First value: {}", first_value);
                let second_value = precise_supply
                    .checked_powi(3)
                    .expect("calculate_tokens_received problem. First powi");
                // info!("Second value: {}", second_value);
                let third_value = (first_value + second_value)
                    .checked_nth_root(3)
                    .expect("calculate_tokens_received problem. First root");
                // info!("Third value: {}", third_value);
                let precise_result = third_value - precise_supply;
                // info!("Result: {}", precise_result);
                result = Decimal::try_from(
                    precise_result
                        .checked_round(18, RoundingMode::ToNearestMidpointAwayFromZero)
                        .expect("calculate_tokens_received problem. Cant round precise decimal."),
                )
                .expect(
                    "calculate_tokens_received problem. Cant convert precise decimal to decimal.",
                );
            }
            result
        }

        // function to calculate the sell price (XRD received) from selling the speficied number of tokens
        fn calculate_sell_price(
            &self,
            sell_tokens: Decimal, // the amount of tokens to sell
            supply: Decimal,      // the supply of tokens before the buy transaction
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal,
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            let multiplier = calculate_multiplier(max_xrd_market_cap, max_token_supply_to_trade);
            if sell_tokens > Decimal::ZERO {
                let precise_supply = PreciseDecimal::from(supply.clone());
                let precise_new_supply = precise_supply.clone() - sell_tokens.clone();

                let first_value: PreciseDecimal = multiplier
                    .clone()
                    .checked_div(3)
                    .expect("calculate_buy_price problem. Div 3");
                let second_value = (precise_supply.clone())
                    .checked_powi(3)
                    .expect("calculate_buy_price problem. First Powi(3).");
                let third_value = (precise_new_supply.clone())
                    .checked_powi(3)
                    .expect("calculate_buy_price problem. Second Powi(3).");
                let fourth_value = second_value - third_value;

                let precise_price = first_value
                    .checked_mul(fourth_value)
                    .expect("calculate_sell_price problem. Multiplication problem.");
                result = Decimal::try_from(
                    precise_price
                        .checked_round(18, RoundingMode::ToNearestMidpointAwayFromZero)
                        .expect("calculate_sell_price problem. Cant round precise decimal."),
                )
                .expect("calculate_sell_price problem. Cant convert precise decimal to decimal.");
            }
            // info!("Sell price: {:?}", result);
            result
        }

        // function to calculate the amount of tokens to sell to receiv the specified amount of XRD
        fn calculate_tokens_to_sell(
            &self,
            xrd_required: Decimal, // the amount of XRD to receive from selling tokens
            supply: Decimal,       // the supply of tokens before the buy transaction
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal,
        ) -> Decimal {
            let mut result = Decimal::ZERO;
            if xrd_required > Decimal::ZERO {
                let multiplier =
                    calculate_multiplier(max_xrd_market_cap, max_token_supply_to_trade);
                let precise_xrd_required = PreciseDecimal::from(xrd_required.clone());
                // info!("Precise XRD required: {:?}", precise_xrd_required);
                let precise_supply = PreciseDecimal::from(supply.clone());
                // info!("Precise supply: {:?}", precise_supply);
                let mut first_value: PreciseDecimal = precise_xrd_required
                    .checked_mul(3)
                    .expect("calculate_tokens_to_sell problem. First mul");
                // info!("First value: {}", first_value);
                first_value = first_value
                    .checked_div(multiplier.clone())
                    .expect("calculate_tokens_to_sell problem. First div");
                // info!("First value: {}", first_value);
                let second_value = precise_supply
                    .checked_powi(3)
                    .expect("calculate_tokens_to_sell problem. First powi");
                // info!("Second value: {:?}", second_value);
                let third_value = second_value - first_value;
                // info!("Third value: {:?}", third_value);
                let fourth_value = third_value
                    .checked_nth_root(3)
                    .expect("calculate_tokens_to_sell problem. First root");
                // info!("Fourth value: {:?}", fourth_value);
                let precise_result = precise_supply - fourth_value;
                // info!("Precise Result: {:?}", precise_result);
                result = Decimal::try_from(
                    precise_result
                        .checked_round(18, RoundingMode::ToNearestMidpointAwayFromZero)
                        .expect("calculate_tokens_to_sell problem. Cant round precise decimal."),
                )
                .expect(
                    "calculate_tokens_to_sell problem. Cant convert precise decimal to decimal.",
                );
            }
            result
        }

        fn calculate_max_xrd(
            &self,
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal,
        ) -> Decimal {
            let multiplier = calculate_multiplier(max_xrd_market_cap, max_token_supply_to_trade);
            let first_value: PreciseDecimal = multiplier
                .checked_div(3)
                .expect("Problem in calculating max_xrd. First div");
            let precise_max_supply = PreciseDecimal::from(max_token_supply_to_trade);
            let second_value: PreciseDecimal = precise_max_supply
                .checked_powi(3)
                .expect("Problem in calculating max_xrd. First powi");
            let precise_max_xrd: PreciseDecimal = first_value
                .checked_mul(second_value)
                .expect("Problem calculating max_xrd. First mul");
            Decimal::try_from(precise_max_xrd)
                .expect("Problem calculating max_xrd. Could not convert precise_max_xrd to decimal")
        }
    }
}

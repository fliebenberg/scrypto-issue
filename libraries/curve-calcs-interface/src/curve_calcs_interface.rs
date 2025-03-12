use scrypto::prelude::*;
use scrypto_interface::define_interface;

define_interface! {
    CurveCalcsAdaptor impl [
        #[cfg(feature = "trait")]
        Trait,
        #[cfg(feature = "scrypto-stubs")]
        ScryptoStub,
        #[cfg(feature = "scrypto-test-stubs")]
        ScryptoTestStub,
    ] {
        fn address(&self) -> ComponentAddress;

        fn calculate_price(&self, supply: Decimal, max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal) -> Decimal;

        fn calculate_buy_price(
            &self,
            new_tokens: Decimal,        // the amount of tokens to buy
            supply: Decimal,            // the supply of tokens before the buy transaction
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal
        ) -> Decimal;

        fn calculate_tokens_received(
            &self,
            xrd_received: Decimal,      // the amount of XRD to spend to buy tokens
            supply: Decimal,            // the supply of tokens before the buy transaction
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal
        ) -> Decimal;

        fn calculate_sell_price(
            &self,
            sell_tokens: Decimal,       // the amount of tokens to sell
            supply: Decimal,            // the supply of tokens before the buy transaction
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal
        ) -> Decimal;

        fn calculate_tokens_to_sell(
            &self,
            xrd_required: Decimal,      // the amount of XRD to receive from selling tokens
            supply: Decimal,            // the supply of tokens before the buy transaction
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal
        ) -> Decimal;

        fn calculate_max_xrd(
            &self,
            max_xrd_market_cap: Decimal,
            max_token_supply_to_trade: Decimal
        ) -> Decimal;

    }
}

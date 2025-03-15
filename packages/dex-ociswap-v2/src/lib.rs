use dex_interface::dex_interface::*;
use scrypto::prelude::*;
use scrypto_interface::*;

#[blueprint_with_traits]
mod dex_ociswap_v2 {
    extern_blueprint! {
        // "package_rdx1phyk2tszvfhz2ukht6pkg3f3q3ww3fsuwku4uyyzctslpfp6dqksqq" // mainnet
        "package_tdx_2_1phg6mrvsxm5k9zfrtuchj3ra8lp8lkv8fcndj9uj463tmr48zn2uf5", // stokenet
        Pool {
            fn instantiate_with_liquidity(
                a_bucket: Bucket,
                b_bucket: Bucket,
                input_fee_rate: Decimal,
                flash_loan_fee_rate: Decimal,
                a_share: Decimal,
                hook_badges: Vec<(ComponentAddress, Bucket)>,
            ) -> (Global<Pool>, Bucket);

            fn add_liquidity(
                &mut self,
                a_bucket: Bucket,
                b_bucket: Bucket,
            ) -> (Bucket, Option<Bucket>);

            fn remove_liquidity(&mut self, lp_token: Bucket) -> (Bucket, Bucket);
        }
    }
    struct DexOciswapV2Adaptor {
        name: String,
        address: ComponentAddress,
    }

    impl DexOciswapV2Adaptor {
        pub fn new(
            name: String,
            owner_role: OwnerRole,
            address_reservation_option: Option<GlobalAddressReservation>,
        ) -> Global<DexOciswapV2Adaptor> {
            let (address_reservation, component_address) = if let Some(
                provided_address_reservation,
            ) = address_reservation_option
            {
                (
                        provided_address_reservation.clone(),
                        ComponentAddress::try_from(Runtime::get_reservation_address(&provided_address_reservation)).expect("Could not convert DexOciSwapAdaptor address reservation into valid component address"),
                    )
            } else {
                Runtime::allocate_component_address(<DexOciswapV2Adaptor>::blueprint_id())
            };

            Self{
                name: name.clone(),
                address: component_address,
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .metadata(metadata! {
                    init {
                    "name" => format!("Rakoon.fun Dex Adaptor: {}", name.clone()), updatable;
                    "description" => format!("Rakoon.fun Dex Adaptor component for {}", name.clone()), updatable;
                    "tags" => vec!["Rakoon.fun", "Token", "Meme", "Launcher"], updatable;
                    }
                })
            .with_address(address_reservation)
            .globalize()
        }
    }

    impl DexAdaptorInterfaceTrait for DexOciswapV2Adaptor {
        fn name(&self) -> String {
            self.name.clone()
        }

        fn address(&self) -> ComponentAddress {
            self.address.clone()
        }

        // function to create a new listing/pool/pair on the dex.
        // Returns: The ComponentAddress of the listing/pool/pair, a bucket with the listing tokens, any other tokens that need to be returned.
        fn create_listing(
            &mut self,
            token1: Bucket,
            token2: Bucket,
        ) -> (ComponentAddress, Bucket, Vec<Bucket>) {
            let (pool_component, pool_tokens) = Blueprint::<Pool>::instantiate_with_liquidity(
                token1,
                token2,
                dec!("0.003"),
                dec!("0.0009"),
                dec!("0.5"), // this tells the dex that the xrd value of the tokens and xrd sent is the same i.e. it is a 50/50 pool in terms of the xrd value of the sides.
                vec![],
            );
            let pool_component_address = pool_component.address();
            let other_buckets: Vec<Bucket> = vec![];
            (pool_component_address, pool_tokens, other_buckets)
        }

        fn add_liquidity(
            &mut self,
            listing_address: ComponentAddress,
            token1: Bucket,
            token2: Bucket,
        ) -> (Bucket, Vec<Bucket>) {
            let mut pool: Global<Pool> = listing_address.into();
            let (pool_tokens, other_tokens) = pool.add_liquidity(token1, token2);
            let mut extra_tokens: Vec<Bucket> = vec![];
            if let Some(other_returned_tokens) = other_tokens {
                extra_tokens.push(other_returned_tokens);
            }
            (pool_tokens, extra_tokens)
        }

        fn remove_liquidity(
            &mut self,
            listing_address: ComponentAddress,
            listing_tokens: Bucket,
        ) -> (Bucket, Bucket, Vec<Bucket>) {
            let mut pool = Global::<Pool>::from(listing_address);
            let (token1, token2) = pool.remove_liquidity(listing_tokens);
            let extra_tokens: Vec<Bucket> = vec![];
            (token1, token2, extra_tokens)
        }
    }
}

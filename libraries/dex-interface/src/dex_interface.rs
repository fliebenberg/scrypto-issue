use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    DexAdaptor impl [
        #[cfg(feature = "trait")]
        Trait,
        #[cfg(feature = "scrypto-stubs")]
        ScryptoStub,
        #[cfg(feature = "scrypto-test-stubs")]
        ScryptoTestStub,
    ] {
        fn name(&self) -> String;

        fn address(&self) -> ComponentAddress;

        // creates a new listing on a dex
        // inputs: buckets with the 2 tokens to be used to create the listing
        // returns:
        //  - the component address of the newly created listing
        //  - a bucket with the listing/pool/pair tokens
        //  - a vector containing any other buckets returned by the function
        fn create_listing(
          &mut self,
          #[manifest_type = "ManifestBucket"]
          token1: Bucket,
          #[manifest_type = "ManifestBucket"]
          token2: Bucket
        ) -> (ComponentAddress, Bucket, Vec<Bucket>);


        // adds liquidity to an existing listing
        // inputs:
        //   - address of the listing/pool
        //   - buckets with the 2 tokens to add to liquidity
        // returns:
        //  - a bucket with the listing/pool/pair tokens
        //  - a vector containing any other buckets returned by the function
        fn add_liquidity(
            &mut self,
            listing_address: ComponentAddress,
            #[manifest_type = "ManifestBucket"]
            token1: Bucket,
            #[manifest_type = "ManifestBucket"]
            token2: Bucket
        ) -> (Bucket, Vec<Bucket>);


        // adds liquidity to an existing listing
        // inputs:
        //   - address of the listing/pool
        //   - bucket with the listing/pool/pair tokens to be redeemed
        // returns:
        //  - 2 buckets with the returned tokens
        //  - a vector containing any other buckets returned by the function
        fn remove_liquidity(
            &mut self,
            listing_address: ComponentAddress,
            #[manifest_type = "ManifestBucket"]
            listing_tokens: Bucket,
        ) -> (Bucket, Bucket, Vec<Bucket>);
    }
}

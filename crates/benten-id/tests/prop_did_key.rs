//! G14-A1 wave-4a — `did:key` round-trip byte-identity proptest (un-ignored).

#![allow(clippy::unwrap_used)]

use benten_crypto_suite::primitives::ed25519_dalek::SigningKey;
use benten_id::did::Did;
use benten_id::keypair::PublicKey;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    #[test]
    fn prop_did_key_round_trip_byte_identity(seed in any::<[u8; 32]>()) {
        // Build a SigningKey from the seed (any 32 bytes are accepted
        // by ed25519-dalek as a SigningKey seed; the corresponding
        // verifying key is always a valid Edwards point), then run
        // the encode → decode round-trip on its 32-byte pubkey.
        let signing = SigningKey::from_bytes(&seed);
        let pk_bytes = signing.verifying_key().to_bytes();
        let pk = PublicKey::from_bytes(&pk_bytes).unwrap();
        let did = Did::from_public_key(&pk);
        let pk_decoded = did.resolve().unwrap();
        prop_assert_eq!(
            pk_decoded.to_bytes(),
            pk_bytes,
            "did:key round-trip must preserve every bit of the public key"
        );
    }
}

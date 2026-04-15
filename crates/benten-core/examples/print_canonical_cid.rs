//! Print the canonical test Node's CIDv1 (base32, multibase-`b` prefixed) to
//! stdout. Used by the spike RESULTS doc and by humans who want a sanity
//! check of cross-run determinism.
//!
//! Run with: `cargo run --example print_canonical_cid -p benten-core`.

#![allow(
    clippy::print_stdout,
    clippy::expect_used,
    reason = "this is an example binary whose entire job is to print the CID"
)]

use benten_core::testing::canonical_test_node;

fn main() {
    let cid = canonical_test_node().cid().expect("hash");
    println!("{cid}");
}

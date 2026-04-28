//! Phase 2a G5-B-ii / phil-r1-1: pinned empty-extensions `AttributionFrame`
//! fixture CID.
//!
//! The public assertion lives in the integration test
//! `crates/benten-eval/tests/invariant_14_fixture_cid.rs` — it constructs an
//! [`AttributionFrame`](crate::AttributionFrame) with three zero-digest CIDs
//! and asserts that `frame.cid().to_string()` equals `FIXTURE_CID`. A shift
//! in the computed CID means the `AttributionFrame` schema changed
//! non-additively; Phase-6 additions MUST re-render the pinned constant.
//!
//! Keeping the constant in a dedicated sibling file (rather than inline in
//! the test) means Phase-3 / Phase-6 reviewers have a single grep-friendly
//! landing pad (`FIXTURE_CID`) to audit the schema pin.

/// Phase-2a empty-extensions `AttributionFrame` fixture CID. Mirrors the
/// constant in `crates/benten-eval/tests/invariant_14_fixture_cid.rs`. Any
/// change here without a matching test-side update fails CI.
pub const FIXTURE_CID: &str = "bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a";

#[cfg(test)]
mod tests {
    use super::FIXTURE_CID;
    use crate::AttributionFrame;
    use benten_core::Cid;

    fn zero_cid() -> Cid {
        Cid::from_blake3_digest([0u8; 32])
    }

    /// SHAPE-PIN: duplicate of the integration-test assertion so the
    /// fixture CID survives even if the external test file is gated off.
    #[test]
    fn fixture_cid_matches_frame_encoding() {
        let frame = AttributionFrame {
            actor_cid: zero_cid(),
            handler_cid: zero_cid(),
            capability_grant_cid: zero_cid(),
            // Phase-2b G7-B (D20 additive) — default sandbox_depth = 0
            // keeps the Phase-2a fixture CID stable.
            sandbox_depth: 0,
        };
        let cid = frame.cid().expect("attribution frame cid");
        assert_eq!(cid.to_string(), FIXTURE_CID);
    }
}

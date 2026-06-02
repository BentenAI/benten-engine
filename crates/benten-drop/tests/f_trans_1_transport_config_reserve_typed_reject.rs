//! **F-TRANS-1** — TransportConfig reserve typed-reject (GAP-2c, #53).
//!
//! ADDL Phase-4-Meta-Core, F-full **R3-W7 (doc-wave)** partition.
//! Self-contained dispatch-stub + doc-coupling family.
//!
//! Pin source: `.addl/phase-4-meta/f-full-r2-test-landscape.md` §1
//! Group-12 **F-TRANS-1**:
//!   "Willow/iroh-roq/iroh-live `TransportConfig` variants typed-reject at
//!    v1-beta while `GossipPlusBlobs` ships.  §5.2 #53, §3.9.  FG.
//!    ~3 tests.  Red-phase intent: reserve transports → typed-reject;
//!    gossip ships."
//!
//! **RED-PHASE (pim-12 §3.6e):** the real `TransportConfig` enum +
//! `benten-sync` transport selection do NOT carry these reserved variants
//! at baseline. The end-state arms `#[ignore = "RED-PHASE: F-TRANS-1 ..."]`;
//! R5 wires the real `TransportConfig`. The NON-ignored baseline arms
//! drive a self-contained transport-dispatch stub (a real selection +
//! typed-reject, not a CONST) + the Compromise #53 disclosure doc-coupling,
//! and pass green now. NEVER `assert_eq!(CONST, CONST_VAL)`.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// ===========================================================================
// SELF-CONTAINED TRANSPORT-DISPATCH STUB-SHIM (no cross-wave / no sync dep).
// Models transport selection: GossipPlusBlobs ships; Willow/iroh-roq/
// iroh-live are RESERVED → typed-reject at v1-beta. R5 replaces this with
// the real `benten_sync::TransportConfig`.
// ===========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StubTransportConfig {
    /// The ONE transport that ships at v1-beta.
    GossipPlusBlobs,
    /// Reserved-and-refused at v1-beta (#53).
    Willow,
    IrohRoq,
    IrohLive,
}

#[derive(Debug, PartialEq, Eq)]
enum StubTransportError {
    /// A reserved transport selected at v1-beta.
    TransportReservedAtV1Beta(&'static str),
}

/// Select a transport. Shipping transport → Ok; reserved → typed-reject.
/// (A real dispatch, not a CONST compare.)
fn stub_select_transport(cfg: StubTransportConfig) -> Result<&'static str, StubTransportError> {
    match cfg {
        StubTransportConfig::GossipPlusBlobs => Ok("gossip+blobs"),
        StubTransportConfig::Willow => Err(StubTransportError::TransportReservedAtV1Beta("Willow")),
        StubTransportConfig::IrohRoq => {
            Err(StubTransportError::TransportReservedAtV1Beta("iroh-roq"))
        }
        StubTransportConfig::IrohLive => {
            Err(StubTransportError::TransportReservedAtV1Beta("iroh-live"))
        }
    }
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .canonicalize()
        .expect("repo root must resolve")
}

fn security_posture_md() -> String {
    std::fs::read_to_string(repo_root().join("docs/SECURITY-POSTURE.md"))
        .expect("SECURITY-POSTURE.md must be present")
}

// ===========================================================================
// BASELINE ARMS (NOT ignored) — real transport dispatch round-trip.
// Pass green now; would-FAIL-if-no-op'd.
// ===========================================================================

/// PIN 0a (baseline) — `GossipPlusBlobs` SHIPS (selectable, returns Ok).
/// A no-op selector that rejects everything would fail this; a selector
/// that accepts everything would fail PIN 0b. The pair pins both halves.
#[test]
fn f_trans_1_gossip_plus_blobs_ships_baseline() {
    let selected = stub_select_transport(StubTransportConfig::GossipPlusBlobs)
        .expect("GossipPlusBlobs MUST be selectable at v1-beta (it ships)");
    assert_eq!(
        selected, "gossip+blobs",
        "the shipping transport MUST be gossip+blobs (the v1-beta default)."
    );
}

/// PIN 0b (baseline) — each reserved transport (Willow / iroh-roq /
/// iroh-live) typed-rejects with `TransportReservedAtV1Beta`. A selector
/// that silently accepts a reserved transport (or falls back to gossip)
/// fails here. This is the would-FAIL-if-no-op'd guard.
#[test]
fn f_trans_1_reserved_transports_typed_reject_baseline() {
    for (cfg, name) in [
        (StubTransportConfig::Willow, "Willow"),
        (StubTransportConfig::IrohRoq, "iroh-roq"),
        (StubTransportConfig::IrohLive, "iroh-live"),
    ] {
        let err = stub_select_transport(cfg)
            .expect_err("reserved transport MUST typed-reject at v1-beta");
        assert_eq!(
            err,
            StubTransportError::TransportReservedAtV1Beta(name),
            "reserved transport {name} MUST typed-reject with \
             TransportReservedAtV1Beta — NEVER a silent accept or a silent \
             fallback to gossip (#53)."
        );
    }
}

// ===========================================================================
// RED-PHASE ARMS (ignored until R5) — real benten-sync TransportConfig
// reserve + the Compromise #53 disclosure.
// ===========================================================================

/// PIN 1 — the REAL `benten-sync` `TransportConfig` enum carries the
/// reserved variants AND typed-rejects them at v1-beta while
/// `GossipPlusBlobs` ships. Source doc-coupling. Would-FAIL if a reserved
/// transport is made LIVE or the enum omits the reserves.
#[test]
#[ignore = "RED-PHASE: F-TRANS-1 — benten-sync TransportConfig reserves \
            Willow/iroh-roq/iroh-live (typed-reject) while GossipPlusBlobs \
            ships; un-ignore at R5"]
fn f_trans_1_transport_config_enum_reserves_and_ships() {
    let mut transport_cfg_src = String::new();
    let sync_dir = repo_root().join("crates/benten-sync");
    let mut stack = vec![sync_dir];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if p.file_name().and_then(|s| s.to_str()) != Some("target") {
                    stack.push(p);
                }
            } else if p.extension().and_then(|s| s.to_str()) == Some("rs")
                && let Ok(c) = std::fs::read_to_string(&p)
                && c.contains("TransportConfig")
            {
                transport_cfg_src.push_str(&c);
            }
        }
    }
    assert!(
        transport_cfg_src.contains("TransportConfig"),
        "`TransportConfig` MUST exist in benten-sync at R5."
    );
    assert!(
        transport_cfg_src.contains("GossipPlusBlobs"),
        "`TransportConfig` MUST carry the shipping `GossipPlusBlobs` variant."
    );
    for reserved in ["Willow", "IrohRoq", "IrohLive"] {
        assert!(
            transport_cfg_src.contains(reserved),
            "`TransportConfig` MUST reserve the `{reserved}` variant \
             (typed-reject at v1-beta, #53). R5 adds it."
        );
    }
}

/// PIN 2 — Compromise #53 (TransportConfig reserve) is DISCLOSED in
/// SECURITY-POSTURE.md, naming gossip-as-shipping + the reserved
/// transports as deferred. Doc-coupling (coheres with F-DISC-1's row
/// sweep). Would-FAIL if #53 is claimed-but-undocumented.
#[test]
#[ignore = "RED-PHASE: F-TRANS-1 — Compromise #53 (TransportConfig \
            reserve) disclosed in SECURITY-POSTURE.md; un-ignore at R5"]
fn f_trans_1_compromise_53_disclosed() {
    let doc = security_posture_md();
    // Locate the #53 row window and assert it names the transport posture.
    let lines: Vec<&str> = doc.lines().collect();
    let mut window = String::new();
    for (i, l) in lines.iter().enumerate() {
        if l.contains("Compromise #53") {
            let lo = i.saturating_sub(1);
            let hi = (i + 4).min(lines.len());
            window = lines[lo..hi].join(" ");
            break;
        }
    }
    assert!(
        !window.is_empty(),
        "Compromise #53 (TransportConfig reserve) MUST be disclosed in \
         SECURITY-POSTURE.md. R5 doc-wave adds the row."
    );
    assert!(
        window.contains("Transport")
            || window.contains("transport")
            || window.contains("Willow")
            || window.contains("gossip"),
        "the Compromise #53 row MUST name the transport-reserve posture \
         (Willow/iroh-roq/iroh-live reserved; gossip ships). Got: {:?}",
        window
    );
}

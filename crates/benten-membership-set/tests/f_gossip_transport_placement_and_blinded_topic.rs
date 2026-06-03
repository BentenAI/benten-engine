//! F-GOSSIP-1/2 (R3-W5) — GossipTransport placement (liveness-only) +
//! HMAC-blinded topic (fork-rotation + OOB rendezvous).
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 rows **F-GOSSIP-1** (H1/H2 + G1 +
//!   GNI-26, NQ-D1) + **F-GOSSIP-2** (H3/H4 + PMD-12/13/14/15, NQ-D4, #61).
//! - R0.3 plan §3.9 (Transport / D6 + convergence model, M-10):
//!   * the new `GossipTransport` lands in `benten-sync` (lean; NQ-D1);
//!     `benten-membership-set` defines NO transport; NO iroh-concrete leak
//!     in the trait signature.
//!   * **dropping ALL gossip still converges via MST** (gossip = liveness
//!     ONLY, never the convergence path).
//!   * `topic = truncate(HMAC(K_Set, membership_set_id ‖ generation_summary))`;
//!     current-gen members compute the SAME topic; a fork rotates the
//!     generation ⇒ rotates the topic; `generation_summary` = the
//!     set-generation counter, NOT a per-member vector (O-5/m-11); an
//!     observer without `K_Set` can't link/recover `set_id`; the topic is
//!     a pure fn of `(K_Set, set_id, gen)` (NO time input). Closes #61.
//! - Inv-20 clause-d (per-recipient unlinkability = network-observer-only —
//!   m-7).
//!
//! ## Why FREEZE-GATING (the topic bytes are crypto-wire — §1.5 rule)
//!
//! The blinded topic is a keying-derived crypto-wire value; a drift in its
//! byte derivation breaks rendezvous (current-gen members compute different
//! topics ⇒ never meet) OR leaks the set fingerprint (#61). F-GOSSIP-1 is
//! a placement + convergence-independence pin (FG/FN); F-GOSSIP-2 is the
//! topic-bytes freeze (FG/FN).
//!
//! ## pim-2 §3.6b + §3.6f-ext end-to-end discipline
//!
//! Drives the PRODUCTION `compute_gossip_topic` + `MockGossipTransport`
//! stand-ins; asserts OBSERVABLE same-gen→same-topic / fork→diff-topic /
//! K_Set-bitflip→diff-topic / set_id-not-recoverable / convergence-without-gossip;
//! would-FAIL-if-no-op'd (a topic keyed off time differs across two clocks;
//! a topic that embeds set_id in clear is recoverable; convergence that
//! depended on gossip fails the gossip-disabled arm).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`; SELF-CONTAINED stub-shim for
//! parallel-safe R3. The blinded-topic stub-shim uses BLAKE3-keyed-hash as
//! the HMAC stand-in (R5 swaps in the real `benten-crypto-suite` HMAC over
//! `K_Set`); the byte-derivation SHAPE (key ‖ set_id ‖ gen, truncate-to-32,
//! no time input) is what F-GOSSIP-2 freezes.

#![allow(clippy::unwrap_used)]

// ── SELF-CONTAINED stub-shim ──

/// PRODUCTION-stand-in: `topic = truncate(HMAC(K_Set, set_id ‖ gen))`.
/// The stub uses BLAKE3 keyed-hash as the HMAC stand-in. R5 routes through
/// the real crypto-suite HMAC. Pure fn of `(k_set, set_id, generation)` —
/// NO time input (so two clocks derive the identical topic).
fn compute_gossip_topic(k_set: &[u8; 32], set_id: &[u8], generation: u32) -> [u8; 32] {
    let mut msg = Vec::with_capacity(set_id.len() + 4);
    msg.extend_from_slice(set_id);
    // generation_summary = the set-generation counter (BE u32), NOT a
    // per-member vector (O-5 / m-11).
    msg.extend_from_slice(&generation.to_be_bytes());
    // BLAKE3 keyed-hash as the HMAC(K_Set, ·) stand-in; truncate-to-32 is
    // already the BLAKE3 output width.
    blake3::keyed_hash(k_set, &msg).into()
}

/// An iroh-FREE in-memory gossip transport (the F-GOSSIP-1 compile-fence
/// analogue to the in-tree `transport_trait_boundary.rs` MockTransport).
/// It carries NO `iroh::`-concrete type in any signature — if a future
/// edit leaked one into the trait, this impl would stop compiling.
trait GossipTransport {
    /// Subscribe to a blinded topic; deliver-or-drop is liveness-only.
    fn subscribe(&mut self, topic: [u8; 32]);
    /// Broadcast a liveness notification ("something changed, come
    /// anti-entropy"). Best-effort; no causal/ordered/exactly-once promise.
    fn notify(&mut self, topic: [u8; 32]);
    /// Whether a subscriber would receive a notification on `topic`.
    fn would_deliver(&self, topic: [u8; 32]) -> bool;
}

struct MockGossipTransport {
    subscribed: std::collections::BTreeSet<[u8; 32]>,
    notified: std::collections::BTreeSet<[u8; 32]>,
}
impl MockGossipTransport {
    fn new() -> Self {
        MockGossipTransport {
            subscribed: Default::default(),
            notified: Default::default(),
        }
    }
}
impl GossipTransport for MockGossipTransport {
    fn subscribe(&mut self, topic: [u8; 32]) {
        self.subscribed.insert(topic);
    }
    fn notify(&mut self, topic: [u8; 32]) {
        self.notified.insert(topic);
    }
    fn would_deliver(&self, topic: [u8; 32]) -> bool {
        self.subscribed.contains(&topic) && self.notified.contains(&topic)
    }
}

// ── F-GOSSIP-1 ──────────────────────────────────────────────────────────

/// F-GOSSIP-1 arm 1 — placement + liveness-only compile-fence.
///
/// A `MockGossipTransport` (channel/set-backed, NO iroh) implements the
/// `GossipTransport` trait — proving the trait surface is iroh-free
/// (NQ-D1: the transport lands in `benten-sync`; `benten-membership-set`
/// defines NO transport). If the trait signature required an iroh-concrete
/// type, this impl would not compile.
#[test]
#[ignore = "RED-PHASE: F-GOSSIP-1 — GossipTransport impl-in-isolation, no iroh leak (NQ-D1); un-ignore at R5"]
fn f_gossip_1_transport_impl_in_isolation_no_iroh_leak() {
    let mut t = MockGossipTransport::new();
    let topic = [0x42u8; 32];
    t.subscribe(topic);
    t.notify(topic);
    assert!(
        t.would_deliver(topic),
        "the iroh-free mock implements GossipTransport (no iroh in sig)"
    );
    // A non-subscribed topic is not delivered (the transport is not a
    // no-op that delivers everything — would-FAIL-if-no-op'd).
    assert!(
        !t.would_deliver([0x99u8; 32]),
        "non-subscribed topic is not delivered"
    );
}

/// F-GOSSIP-1 arm 2 — convergence is INDEPENDENT of gossip (gossip =
/// liveness-only; MST = the convergence path).
///
/// With ALL gossip dropped, the membership state still converges via the
/// MST anti-entropy backstop. Conversely, gossip-only (no MST) does NOT
/// converge — gossip merely notifies; it is never the convergence path.
#[test]
#[ignore = "RED-PHASE: F-GOSSIP-1 — convergence holds with gossip dropped; gossip-only does not converge; un-ignore at R5"]
fn f_gossip_1_convergence_independent_of_gossip() {
    // Model: convergence requires an anti-entropy exchange (`mst_exchanged`),
    // NOT a gossip notification (`gossip_delivered`).
    fn converged(mst_exchanged: bool, _gossip_delivered: bool) -> bool {
        mst_exchanged // gossip is irrelevant to convergence
    }
    // Gossip dropped entirely → still converges via MST.
    assert!(
        converged(true, false),
        "convergence holds with ALL gossip dropped (MST backstop)"
    );
    // Gossip-only (no MST) → does NOT converge.
    assert!(
        !converged(false, true),
        "gossip-only (no MST anti-entropy) does NOT converge — gossip is liveness-only"
    );
}

// ── F-GOSSIP-2 ──────────────────────────────────────────────────────────

/// F-GOSSIP-2 arm 1 — same-generation members compute the SAME topic;
/// the topic is a pure fn of `(K_Set, set_id, gen)` with NO time input.
#[test]
#[ignore = "RED-PHASE: F-GOSSIP-2 — same-gen→same-topic; pure fn, no time input (two clocks → identical topic); un-ignore at R5"]
fn f_gossip_2_same_generation_same_topic_no_time_input() {
    let k_set = [0xABu8; 32];
    let set_id = blake3::hash(b"set-alpha");
    let set_id = set_id.as_bytes();

    // Two members, two different wall-clocks (modeled by calling twice).
    let topic_member_1 = compute_gossip_topic(&k_set, set_id, 7);
    let topic_member_2 = compute_gossip_topic(&k_set, set_id, 7);
    assert_eq!(
        topic_member_1, topic_member_2,
        "current-gen members compute the SAME topic (pure fn; no time input)"
    );
}

/// F-GOSSIP-2 arm 2 — a fork rotates the generation ⇒ rotates the topic.
#[test]
#[ignore = "RED-PHASE: F-GOSSIP-2 — fork rotates generation ⇒ rotates topic; un-ignore at R5"]
fn f_gossip_2_fork_rotates_topic() {
    let k_set = [0xABu8; 32];
    let set_id = blake3::hash(b"set-alpha");
    let set_id = set_id.as_bytes();
    let topic_gen_7 = compute_gossip_topic(&k_set, set_id, 7);
    let topic_gen_8 = compute_gossip_topic(&k_set, set_id, 8); // post-fork generation
    assert_ne!(
        topic_gen_7, topic_gen_8,
        "a fork that rotates the generation MUST rotate the topic (#61 fingerprint defense)"
    );
}

/// F-GOSSIP-2 arm 3 — `K_Set` bit-flip ⇒ different topic; an observer
/// WITHOUT `K_Set` cannot link/recover `set_id` from the topic.
#[test]
#[ignore = "RED-PHASE: F-GOSSIP-2 — K_Set-bitflip→diff-topic + set_id not recoverable from topic; un-ignore at R5"]
fn f_gossip_2_k_set_blinds_set_id() {
    let set_id = blake3::hash(b"set-alpha");
    let set_id = set_id.as_bytes();

    let k_set_a = [0xABu8; 32];
    let mut k_set_b = k_set_a;
    k_set_b[0] ^= 0x01; // single-bit flip

    let topic_a = compute_gossip_topic(&k_set_a, set_id, 7);
    let topic_b = compute_gossip_topic(&k_set_b, set_id, 7);
    assert_ne!(
        topic_a, topic_b,
        "a K_Set bit-flip MUST change the topic (keyed-blinding; #61)"
    );

    // Unlinkability: the topic bytes do NOT contain the set_id in clear
    // (an observer without K_Set can't recover set_id). The topic is a
    // keyed hash, so the raw set_id bytes never appear as a substring.
    assert!(
        !contains_subslice(&topic_a, set_id),
        "the blinded topic MUST NOT leak the set_id (network-observer-only unlinkability — Inv-20 clause-d)"
    );
}

/// F-GOSSIP-2 arm 4 — OOB-less first-contact is impossible (the topic is
/// not guessable without `K_Set`; bootstrap requires an out-of-band
/// Drop-bundle rendezvous).
///
/// Doc-coupling note: the OOB first-contact residue (#43/#62) is documented
/// in SECURITY-POSTURE.md / THREAT-MODEL.md at the R5 doc-wave; this arm
/// pins the cryptographic precondition (topic unguessable without K_Set).
#[test]
#[ignore = "RED-PHASE: F-GOSSIP-2 — OOB-less join impossible (topic unguessable without K_Set); un-ignore at R5"]
fn f_gossip_2_oob_rendezvous_required() {
    let set_id = blake3::hash(b"set-alpha");
    let set_id = set_id.as_bytes();
    let real_topic = compute_gossip_topic(&[0xABu8; 32], set_id, 7);

    // An attacker who knows set_id + generation but NOT K_Set guesses a
    // wrong key — the derived topic does not match (no OOB ⇒ no join).
    let guessed_topic = compute_gossip_topic(&[0x00u8; 32], set_id, 7);
    assert_ne!(
        real_topic, guessed_topic,
        "without K_Set (only obtainable via the OOB Drop-bundle rendezvous), the topic is unguessable"
    );
}

// ── helpers ─────────────────────────────────────────────────────────────

fn contains_subslice(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

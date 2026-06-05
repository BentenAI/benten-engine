//! F-GOSSIP-1/2 (R3-W5) — GossipTransport placement (liveness-only) +
//! HMAC-blinded topic (fork-rotation + OOB rendezvous).
//!
//! ## Pin source
//!
//! - F-full R2 test-landscape §1 Group 8 rows **F-GOSSIP-1** (H1/H2 + G1 +
//!   GNI-26, NQ-D1) + **F-GOSSIP-2** (H3/H4 + PMD-12/13/14/15, NQ-D4, #61).
//! - R0.5 plan §3.9 (Transport / D6 + convergence model, M-10):
//!   * the new `GossipTransport` lands in `benten-sync` (lean; NQ-D1);
//!     `benten-membership-set` defines NO transport; NO iroh-concrete leak
//!     in the trait signature.
//!   * **dropping ALL gossip still converges via MST** (gossip = liveness
//!     ONLY, never the convergence path).
//!   * `topic = truncate(HMAC(K_Set, membership_set_id ‖ generation_summary))`;
//!     current-gen members compute the SAME topic; a fork rotates the
//!     generation ⇒ rotates the topic; `generation_summary` = the
//!     set-generation counter (BE u32), NOT a per-member vector (O-5/m-11); an
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
//! topic-bytes freeze (FG/FN) — pinned by an ABSOLUTE golden-hex vector with
//! a BE/LE differentiator (F4-024).
//!
//! ## pim-2 §3.6b + §3.6f-ext end-to-end discipline
//!
//! Drives the PRODUCTION `compute_gossip_topic` + `MockGossipTransport` +
//! `mst_backstop_converges` stand-ins; asserts OBSERVABLE
//! same-gen→same-topic / fork→diff-topic / K_Set-bitflip→diff-topic /
//! set_id-not-recoverable / absolute-golden-bytes / convergence-without-gossip;
//! would-FAIL-if-no-op'd (a topic keyed off time differs across two clocks;
//! a topic that embeds set_id in clear is recoverable; an LE-encoded
//! generation flips the golden hex; convergence that depended on gossip
//! fails the gossip-disabled arm).
//!
//! ## RED-PHASE (pim-12 §3.6e) + SELF-CONTAINED stub-shim
//!
//! Compiles GREEN behind `#[ignore]`; SELF-CONTAINED stub-shim for
//! parallel-safe R3. The blinded-topic stub-shim stub-shims the keyed MAC as
//! `blake3::keyed_hash(K_Set, ·)` — where R0.7 §4.1 clarifies `HMAC` =
//! `blake3::keyed_hash` (native BLAKE3 keyed MAC; no hmac/sha2 dep; bytes
//! unchanged), the SAME primitive the sibling
//! `f_aad_2_nine_tuple_injectivity_opaque_boundary.rs` uses for
//! `membership_set_id_commitment`. R5 routes through the real
//! `benten-crypto-suite` keyed MAC over `K_Set` (NOT HMAC-SHA256 — the crate
//! carries no hmac/sha2 dep; do NOT invent a different HMAC/hash). The
//! byte-derivation SHAPE (key ‖ set_id ‖ BE(gen), truncate-to-32, no time
//! input) is what F-GOSSIP-2 freezes via the absolute golden vector.

#![allow(clippy::unwrap_used)]

// ── SELF-CONTAINED stub-shim ──

/// PRODUCTION-stand-in: `topic = truncate(HMAC(K_Set, set_id ‖ BE(gen)))`.
/// The stub uses `blake3::keyed_hash(K_Set, ·)` — where R0.7 §4.1 clarifies
/// `HMAC` = `blake3::keyed_hash` (native BLAKE3 keyed MAC; no hmac/sha2 dep;
/// bytes unchanged). R5 routes through the real `benten-crypto-suite` keyed
/// MAC over `K_Set` (NOT HMAC-SHA256). Pure fn of `(k_set, set_id, generation)`
/// — NO time input (so two clocks derive the identical topic). The generation
/// is encoded BIG-ENDIAN (the freeze-gating byte-order — an LE encoder would
/// flip the golden vector below).
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

// ── convergence-via-MST stand-in (the sibling of f_mst_*; F4-013) ──
//
// Convergence is backed by MST anti-entropy, NOT gossip (gossip = liveness
// ONLY). The sibling `f_mst_membership_anti_entropy_backstop.rs` owns the
// O(log n) anti-entropy proof; here we route the placement-independence pin
// through a behaviour-equivalent stand-in that ACTUALLY exchanges the two
// peers' state via the MST channel and converges them — so dropping gossip
// is observably irrelevant, while dropping MST observably prevents
// convergence. This is the substantive sibling stand-in (NOT a `_gossip`-
// argument-ignoring tautology).

/// Two peers each hold a set of membership events. Convergence happens iff an
/// MST anti-entropy exchange occurs (it unions the two sets); gossip
/// notifications never move state. Returns whether the two peers ended up
/// holding the SAME event-set. R5 routes through `benten_sync::mst`.
fn mst_backstop_converges(
    peer_a: &mut std::collections::BTreeSet<u64>,
    peer_b: &mut std::collections::BTreeSet<u64>,
    mst_exchange: bool,
    gossip_notifications: u32,
) -> bool {
    // Gossip notifications NEVER mutate state — they only (in production)
    // prompt a peer to initiate an MST exchange. We deliberately consume the
    // count without acting on it to make the "liveness-only" contract
    // observable: no matter how many notifications fire, convergence rides
    // ENTIRELY on `mst_exchange`.
    let _liveness_only = gossip_notifications;
    if mst_exchange {
        // MST anti-entropy: union both peers' event-sets (convergence).
        let union: std::collections::BTreeSet<u64> = peer_a.union(peer_b).copied().collect();
        *peer_a = union.clone();
        *peer_b = union;
    }
    peer_a == peer_b
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
/// F4-013: routes through the `mst_backstop_converges` sibling stand-in
/// (which ACTUALLY moves state across the MST channel) rather than a
/// `_gossip`-argument-ignoring boolean. With ALL gossip dropped, two
/// divergent peers still converge via the MST exchange; with NO MST exchange
/// (even under a storm of gossip notifications), they do NOT converge.
#[test]
#[ignore = "RED-PHASE: F-GOSSIP-1 — convergence holds with gossip dropped; gossip-only does not converge; un-ignore at R5"]
fn f_gossip_1_convergence_independent_of_gossip() {
    use std::collections::BTreeSet;

    // Two peers with DIVERGENT membership event-sets.
    let base_a: BTreeSet<u64> = [1, 2, 3].into_iter().collect();
    let base_b: BTreeSet<u64> = [3, 4, 5].into_iter().collect();
    let expected_union: BTreeSet<u64> = [1, 2, 3, 4, 5].into_iter().collect();

    // Gossip dropped entirely (0 notifications) → still converges via the MST
    // exchange, AND converges to the correct union (observable consequence —
    // a no-op MST stand-in would leave them divergent and FAIL here).
    let mut a = base_a.clone();
    let mut b = base_b.clone();
    let converged =
        mst_backstop_converges(&mut a, &mut b, /* mst */ true, /* gossip */ 0);
    assert!(
        converged,
        "convergence holds with ALL gossip dropped (MST backstop)"
    );
    assert_eq!(
        a, expected_union,
        "the MST exchange produced the correct converged event-set (union)"
    );
    assert_eq!(a, b, "both peers hold the identical converged set");

    // Gossip-only (no MST exchange) → does NOT converge, even under a storm of
    // notifications. Gossip is liveness-only; it never moves state.
    let mut a2 = base_a.clone();
    let mut b2 = base_b.clone();
    let converged_gossip_only = mst_backstop_converges(
        &mut a2, &mut b2, /* mst */ false, /* gossip */ 1000,
    );
    assert!(
        !converged_gossip_only,
        "gossip-only (no MST anti-entropy) does NOT converge — gossip is liveness-only"
    );
    assert_eq!(
        a2, base_a,
        "gossip notifications never mutated peer A's state"
    );
    assert_eq!(
        b2, base_b,
        "gossip notifications never mutated peer B's state"
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

/// F-GOSSIP-2 arm 1b — **ABSOLUTE golden-hex byte freeze (F4-024)** for
/// `truncate_32(HMAC(K_Set, set_id ‖ BE(generation)))`, WITH an explicit
/// BE/LE differentiator.
///
/// The earlier arms only proved relative properties (same-gen→same /
/// fork→differs). They freeze ZERO absolute bytes: an R5 impl that encoded
/// the generation LITTLE-ENDIAN, or reordered `set_id ‖ gen`, would pass
/// every relative arm yet break wire-interop (two engines on different byte
/// orders never meet). This arm pins the exact 32-byte topic for a fixed
/// fixture as a FROZEN literal, then proves the freeze is byte-order-sensitive
/// by showing the SAME generation value LE-encoded yields a DIFFERENT topic.
#[test]
#[ignore = "RED-PHASE: F-GOSSIP-2 — absolute golden-hex topic + BE/LE differentiator (F4-024); un-ignore at R5"]
fn f_gossip_2_topic_absolute_golden_vector_be() {
    let k_set = [0xABu8; 32];
    let set_id = blake3::hash(b"set-alpha");
    let set_id = set_id.as_bytes();

    // Sanity-pin the fixture's set_id so the golden vector below is anchored
    // to a known input (a drift in BLAKE3 framing would surface here first).
    const SET_ID_HEX: &str = "8c1559688fd5c3fa7f19b31656441bff5c5e183093d8d8da7ae4d9e932efbdc7";
    assert_eq!(hex(set_id), SET_ID_HEX, "fixture set_id anchor");

    // FROZEN absolute topic for (K_Set=[0xAB;32], set_id, generation=7).
    // R5 confirms-or-deliberately-updates this frozen literal against the real
    // `benten-crypto-suite` keyed MAC (HMAC = `blake3::keyed_hash`; no hmac/sha2
    // dep; bytes unchanged — R0.7 §4.1) (M-20).
    const TOPIC_GEN7_HEX: &str = "3d28ab3c30ff99adeabdfd08310a97a3f78705494e5de709b96ec34618736527";
    let topic7 = compute_gossip_topic(&k_set, set_id, 7);
    assert_eq!(
        hex(&topic7),
        TOPIC_GEN7_HEX,
        "absolute golden topic for generation 7 (drift in HMAC framing / set_id‖gen order / truncation fails the pin)"
    );

    // ── BE/LE differentiator ──────────────────────────────────────────────
    // Generation 0x01020304 is chosen because its BE encoding (01 02 03 04)
    // differs from its LE encoding (04 03 02 01). The FROZEN golden is the
    // BIG-ENDIAN topic. We then compute what an LE encoder WOULD produce and
    // assert it does NOT match the golden — so any R5 impl that emitted the
    // generation little-endian flips this pin RED.
    const TOPIC_GEN_0X01020304_BE_HEX: &str =
        "ef6af36c02eaad82ecfacfb0e6541fda489df1e9cc13e699fe79ba913df0c450";
    let generation = 0x0102_0304u32;
    let topic_be = compute_gossip_topic(&k_set, set_id, generation);
    assert_eq!(
        hex(&topic_be),
        TOPIC_GEN_0X01020304_BE_HEX,
        "absolute golden topic for generation 0x01020304 encoded BIG-ENDIAN"
    );
    // An LE-encoded generation MUST diverge from the frozen BE golden.
    let mut msg_le = Vec::new();
    msg_le.extend_from_slice(set_id);
    msg_le.extend_from_slice(&generation.to_le_bytes()); // the WRONG byte order
    let topic_le: [u8; 32] = blake3::keyed_hash(&k_set, &msg_le).into();
    assert_ne!(
        hex(&topic_le),
        TOPIC_GEN_0X01020304_BE_HEX,
        "an LE-encoded generation MUST NOT match the frozen BIG-ENDIAN golden (byte-order is freeze-gating)"
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

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

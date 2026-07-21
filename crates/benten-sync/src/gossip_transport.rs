//! Liveness-only gossip transport surface (F-GOSSIP-1 / NQ-D1 / §3.9 D6).
//!
//! ## What this module owns
//!
//! The [`GossipTransport`] trait — a thin pub-sub/notification surface for the
//! iroh-gossip rendezvous channel. It carries **NO `iroh::`-concrete type in
//! any signature** (the F-GOSSIP-1 compile-fence): a topic is an opaque
//! `[u8; 32]`, exactly like the in-tree [`crate::transport_trait`] boundary
//! keeps the iroh connection layer behind an abstract `trait`. The pre-v1
//! concrete iroh-gossip impl (post-canary) implements this trait; the
//! channel-backed `MockGossipTransport` (test/`testing`-gated) proves the
//! surface is iroh-free.
//!
//! ## gossip = liveness ONLY (M-10)
//!
//! Gossip is the **notification** channel ("something changed, come
//! anti-entropy") — it NEVER moves convergence state. Convergence is backed by
//! the EXISTING MST anti-entropy backstop ([`crate::mst`]); dropping ALL gossip
//! still converges, and gossip-only-with-no-MST does NOT converge. The
//! notification carries no causal/ordered/exactly-once promise. This is the
//! load-bearing M-10 separation the `f_gossip_transport_placement_and_blinded_topic`
//! family pins.
//!
//! ## blinded topic
//!
//! The BLINDED rendezvous topic itself is a KEYING-derived value, so it lives
//! in the membership-set keying glue
//! (`benten_membership_set::keying::gossip_topic`) — NOT here. This crate's
//! dependency direction is `membership-set → sync` (the keying crate depends on
//! the transport, never the reverse), so `benten-sync` cannot reference the
//! topic-derivation; it only carries the opaque `[u8; 32]` topic across the
//! transport surface. The topic-byte freeze (F4-024 golden vector) is pinned in
//! `benten-membership-set/tests/`.
//!
//! ## Pin sources
//!
//! - F-full R2 test-landscape §1 Group 8 F-GOSSIP-1 (H1/H2, NQ-D1).
//! - R0.7 plan §3.9 (Transport / D6) + M-10 convergence model.
//! - `benten-membership-set/tests/f_gossip_transport_placement_and_blinded_topic.rs`.

/// A liveness-only gossip notification surface.
///
/// The topic is an opaque `[u8; 32]` (the BLINDED rendezvous label derived by
/// the membership-set keying glue). NO `iroh::`-concrete type appears in any
/// method signature — the F-GOSSIP-1 compile-fence (NQ-D1): the transport lands
/// in `benten-sync` and is implementable in isolation by an iroh-free mock. The
/// pre-v1 concrete iroh-gossip impl implements this same trait as a compile-time
/// engine extension (CLAUDE.md baked-in #19), exactly like the alternate
/// transports behind [`crate::transport_trait::Transport`].
pub trait GossipTransport {
    /// Subscribe to a blinded topic. Deliver-or-drop is best-effort; subscribing
    /// is the precondition for receiving a notification on `topic`.
    fn subscribe(&mut self, topic: [u8; 32]);

    /// Broadcast a liveness notification ("something changed on `topic`, come
    /// anti-entropy"). Best-effort; **no** causal / ordered / exactly-once
    /// promise — a notification NEVER moves convergence state (M-10).
    fn notify(&mut self, topic: [u8; 32]);

    /// Whether a subscriber on `topic` would receive a notification — i.e. the
    /// topic has been both subscribed and notified.
    fn would_deliver(&self, topic: [u8; 32]) -> bool;
}

/// An iroh-FREE in-memory [`GossipTransport`] for tests + the F-GOSSIP-1
/// placement compile-fence.
///
/// Backed by two `BTreeSet<[u8; 32]>` (subscribed / notified) — the
/// channel/set analogue of [`crate::transport_trait`]'s mock. It carries NO
/// `iroh::`-concrete type, so if a future edit leaked one into the
/// [`GossipTransport`] trait signature, this impl would stop compiling — the
/// load-bearing NQ-D1 fence.
#[cfg(any(test, feature = "testing"))]
#[derive(Debug, Default)]
pub struct MockGossipTransport {
    subscribed: std::collections::BTreeSet<[u8; 32]>,
    notified: std::collections::BTreeSet<[u8; 32]>,
}

#[cfg(any(test, feature = "testing"))]
impl MockGossipTransport {
    /// Construct an empty mock transport.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(any(test, feature = "testing"))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_delivers_only_subscribed_and_notified_topics() {
        let mut t = MockGossipTransport::new();
        let topic = [0x42u8; 32];
        // Not delivered until both subscribed AND notified.
        t.subscribe(topic);
        assert!(!t.would_deliver(topic), "subscribe alone does not deliver");
        t.notify(topic);
        assert!(t.would_deliver(topic), "subscribed + notified ⇒ delivered");
        // A never-subscribed topic is never delivered (not a deliver-all no-op).
        assert!(!t.would_deliver([0x99u8; 32]));
    }
}

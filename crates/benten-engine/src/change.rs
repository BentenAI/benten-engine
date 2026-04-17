//! Change-stream channel concretion — Phase 1 reservation.
//!
//! Per the implementation plan's R1 architect addendum (§line-605), the
//! push-shaped [`ChangeSubscriber`](benten_graph::ChangeSubscriber) trait
//! and the [`ChangeEvent`](benten_graph::ChangeEvent) schema live in
//! `benten-graph`, which has no async-runtime dependency. The pull-shaped
//! channel concretion — tokio-broadcast on native, a synchronous
//! `Vec<Box<dyn ChangeSubscriber>>` fan-out on WASM — lives *here* in
//! `benten-engine::change`, which is allowed to depend on tokio.
//!
//! ## Phase 1 status
//!
//! This module is the API reservation only. [`ChangeBroadcast`] is a
//! placeholder type; G3 replaces the `todo!()` bodies with the real
//! tokio-broadcast wiring (native) plus the synchronous fan-out (WASM).
//! Until G3 lands, any consumer reaching this surface sees the panic
//! message as documentation of where the work is scheduled.

#![allow(
    clippy::todo,
    reason = "G3 replaces the channel concretion stubs; reservation lives here per plan"
)]

use std::sync::Arc;

use benten_graph::{ChangeEvent, ChangeSubscriber};

/// Handle to the engine's change-event broadcast.
///
/// Wraps a tokio `broadcast::Sender<ChangeEvent>` on native targets and a
/// synchronous subscriber list on WASM. The underlying representation is
/// **not** part of the public contract — callers interact through the
/// inherent methods below.
///
/// **Phase 1 G3 stub** — every method `todo!()`s. The type is declared here
/// to close architect finding g2-ar-1 (the plan's line-605 ratification that
/// the channel must live in `benten-engine`, not `benten-graph`).
#[derive(Debug, Default)]
pub struct ChangeBroadcast {
    _placeholder: (),
}

impl ChangeBroadcast {
    /// Construct an empty broadcast with no subscribers yet.
    #[must_use]
    pub fn new() -> Self {
        Self { _placeholder: () }
    }

    /// Register a push-subscriber. The broadcast keeps the `Arc` alive and
    /// invokes `on_change` once per successful commit.
    pub fn subscribe(&self, _subscriber: Arc<dyn ChangeSubscriber>) {
        todo!("ChangeBroadcast::subscribe — G3 (tokio-broadcast native + WASM sync-Vec fan-out)")
    }

    /// Publish a change event to every subscriber. Called by the G3
    /// transaction primitive immediately after a successful redb commit.
    pub fn publish(&self, _event: &ChangeEvent) {
        todo!("ChangeBroadcast::publish — G3 (tokio-broadcast native + WASM sync-Vec fan-out)")
    }

    /// Subscriber count — used by thinness tests to assert the broadcast
    /// stays empty when IVM is disabled.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        todo!("ChangeBroadcast::subscriber_count — G3")
    }
}

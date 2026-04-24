//! Scoped binding stack for the evaluator.
//!
//! `EvalContext` holds a stack of name-binding frames the evaluator consults
//! while walking an operation subgraph. Inner frames shadow outer ones; the
//! most recent binding for a given name wins on lookup.
//!
//! The canonical names the evaluator populates are:
//!
//! - `$input` — the top-level input Node handed to the handler.
//! - `$result` — the return value of the most recent primitive.
//! - `$item` — inside an `ITERATE` body, the current element.
//! - `$index` — inside an `ITERATE` body, the current iteration index.
//! - `$results` — the accumulated per-iteration results for an `ITERATE`.
//! - `$error` — inside an error-edge handler, the typed failure payload.
//!
//! `$item`, `$index`, `$results`, and `$error` are deliberately scope-local:
//! they exist only within the frame pushed by the primitive that owns them
//! (`ITERATE` for the iteration bindings; an error-edge handler for
//! `$error`). Popping that frame removes them so they can't leak into
//! sibling branches. See ENGINE-SPEC §5 "Evaluation context scoping".

use benten_core::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Scoped binding stack used by the iterative evaluator.
///
/// Lookup walks the frames from innermost to outermost so an inner `$item`
/// shadows an outer `$item` (nested `ITERATE`s, for example). The outermost
/// frame is always the handler's top-level frame carrying `$input` and the
/// running `$result`.
#[derive(Clone, Default)]
pub struct EvalContext {
    frames: Vec<HashMap<String, Value>>,
    /// Optional injected clock (Phase-2a G3-B-cont). WAIT's resume path
    /// consults this to evaluate deadlines against a test-controlled
    /// time line. `None` means "use process wall clock" — resume treats
    /// the deadline as still in the future.
    clock: Option<Arc<dyn crate::TimeSource>>,
}

impl std::fmt::Debug for EvalContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EvalContext")
            .field("frames", &self.frames)
            .field("has_clock", &self.clock.is_some())
            .finish()
    }
}

impl EvalContext {
    /// Construct a context with a single empty top-level frame.
    #[must_use]
    pub fn new() -> Self {
        Self {
            frames: vec![HashMap::new()],
            clock: None,
        }
    }

    /// Construct a context whose top-level frame is pre-populated with
    /// `$input`.
    #[must_use]
    pub fn with_input(input: Value) -> Self {
        let mut ctx = Self::new();
        ctx.set("$input", input);
        ctx
    }

    /// Phase 2a G3-B: construct a context with a specific clock injected.
    /// The clock drives WAIT's deadline checks at resume time.
    #[must_use]
    pub fn with_clock<T: crate::TimeSource + 'static>(clock: T) -> Self {
        Self {
            frames: vec![HashMap::new()],
            clock: Some(Arc::new(clock)),
        }
    }

    /// Elapsed time (in milliseconds) reported by the injected clock. Phase-
    /// 2a `MockTimeSource::hlc_stamp` returns elapsed microseconds, so we
    /// convert to millis here. Returns `None` when no clock was injected —
    /// WAIT's resume path treats that as "no deadline evaluation possible".
    ///
    /// # Sub-millisecond truncation (G11-A EVAL wave-1)
    ///
    /// The `hlc_stamp() / 1000` conversion is integer division: any
    /// elapsed time below 1 ms reads as 0. A WAIT with `timeout_ms = 1`
    /// resumed 500 µs later therefore reports elapsed=0 and would not
    /// trip the deadline even though half the timeout has passed in
    /// wall-clock terms. The truncation is acceptable under the
    /// Phase-2a WAIT contract — WAIT timeouts are specified in
    /// whole-millisecond units and no Phase-2a handler declares a
    /// sub-millisecond deadline. Phase 2b re-examines this when the
    /// cross-process WAIT durability work (see
    /// `.addl/phase-2b/00-scope-outline.md` §7a) lands a typed
    /// `Duration`-shaped timeout surface; at that point the elapsed-
    /// reporting precision is revisited alongside the envelope format.
    /// Upstream tests that care about sub-ms resolution should read
    /// `TimeSource::hlc_stamp` directly.
    #[must_use]
    pub fn elapsed_ms(&self) -> Option<u64> {
        self.clock.as_ref().map(|c| c.hlc_stamp() / 1000)
    }

    /// Push a new scope onto the binding stack.
    ///
    /// The caller-supplied bindings become the innermost frame and shadow
    /// anything already present. Callers are `ITERATE` (for the loop body,
    /// binding `$item` and `$index`), error-edge handlers (binding `$error`),
    /// and `CALL` with `isolated: true` (fresh frame for the callee).
    pub fn push_scope(&mut self, bindings: HashMap<String, Value>) {
        self.frames.push(bindings);
    }

    /// Pop the innermost scope.
    ///
    /// Returns the popped frame (useful for moving accumulated state to the
    /// parent). If only the top-level frame remains, it is preserved and
    /// `None` is returned; the top-level frame is where `$input` and the
    /// running `$result` live and must not be discarded by the evaluator.
    pub fn pop_scope(&mut self) -> Option<HashMap<String, Value>> {
        if self.frames.len() > 1 {
            self.frames.pop()
        } else {
            None
        }
    }

    /// Look up a binding, searching from innermost to outermost scope.
    ///
    /// Returns `None` if no frame in the stack has the requested key.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Value> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.get(key) {
                return Some(v);
            }
        }
        None
    }

    /// Set a binding in the innermost (current) scope.
    ///
    /// Shadows any same-named binding in outer scopes for the lifetime of
    /// the current frame.
    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        // Invariant: `new()` seeds a frame, `pop_scope()` refuses to drop
        // the last frame. Unwrap is safe but expressed defensively.
        if let Some(frame) = self.frames.last_mut() {
            frame.insert(key.into(), value);
        } else {
            let mut frame = HashMap::new();
            frame.insert(key.into(), value);
            self.frames.push(frame);
        }
    }

    /// Current scope depth (1 for a fresh context, +1 per live `push_scope`).
    #[must_use]
    pub fn depth(&self) -> usize {
        self.frames.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_level_frame_carries_input_and_result() {
        let mut ctx = EvalContext::with_input(Value::text("hi"));
        ctx.set("$result", Value::Int(1));
        assert_eq!(ctx.get("$input"), Some(&Value::text("hi")));
        assert_eq!(ctx.get("$result"), Some(&Value::Int(1)));
    }

    #[test]
    fn inner_scope_shadows_outer() {
        let mut ctx = EvalContext::with_input(Value::text("outer"));
        ctx.set("$item", Value::Int(0));
        let mut inner = HashMap::new();
        inner.insert("$item".to_string(), Value::Int(42));
        ctx.push_scope(inner);
        assert_eq!(ctx.get("$item"), Some(&Value::Int(42)));
        assert_eq!(ctx.get("$input"), Some(&Value::text("outer")));
        ctx.pop_scope();
        assert_eq!(ctx.get("$item"), Some(&Value::Int(0)));
    }

    #[test]
    fn pop_never_discards_top_level_frame() {
        let mut ctx = EvalContext::with_input(Value::Null);
        let popped = ctx.pop_scope();
        assert!(popped.is_none());
        assert_eq!(ctx.depth(), 1);
        assert_eq!(ctx.get("$input"), Some(&Value::Null));
    }

    #[test]
    fn error_binding_is_scope_local() {
        let mut ctx = EvalContext::new();
        let mut err_frame = HashMap::new();
        err_frame.insert("$error".to_string(), Value::text("E_NOT_FOUND"));
        ctx.push_scope(err_frame);
        assert_eq!(ctx.get("$error"), Some(&Value::text("E_NOT_FOUND")));
        ctx.pop_scope();
        assert_eq!(ctx.get("$error"), None);
    }
}

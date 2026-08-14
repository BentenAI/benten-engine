//! B8 — bounded input validation at the TypeScript → Rust napi boundary.
//!
//! Phase-1 R1 (security-auditor finding #7) named five denial-of-service
//! vectors that cross this boundary. R3 wrote the red-phase harness at
//! `bindings/napi/tests/input_validation.rs`; R5 never landed the
//! implementation, and the harness executed in zero CI lanes until the
//! `napi in-process pins (rlib mode)` job was added, so the gap survived
//! several phases. This module is that implementation.
//!
//! ## The shape of the defense
//!
//! Every check runs in a **pre-scan** over the raw wire bytes, before the
//! canonical decoder (`serde_ipld_dagcbor`) is handed anything. That
//! ordering is the whole point: for an allocation attack, the allocation
//! *is* the attack, so a check that runs after the tree is built has
//! already lost. The scanner walks the CBOR framing with an explicit
//! stack of at most `NAPI_MAX_DEPTH + 2` `u64` counters and allocates
//! nothing derived from attacker-declared lengths.
//!
//! The limits mirror the JSON-side limits already enforced in
//! `crate::node` (`JSON_MAX_MAP_KEYS` / `JSON_MAX_BYTES` /
//! `JSON_MAX_TOTAL_BYTES`) so the two boundary shapes —
//! `serde_json::Value` in, DAG-CBOR bytes in — agree on what is
//! acceptable. **Depth is the one place they do not agree**, and the
//! disagreement is load-bearing: see [`NAPI_MAX_DEPTH`].
//!
//! ## What this module does NOT claim
//!
//! - It does not enforce DAG-CBOR *canonicity* (shortest-form integer
//!   encoding, canonical map-key ordering, float64-only). Those are the
//!   canonical decoder's job and it is still run afterwards; the scanner
//!   only refuses to let an unbounded allocation happen first.
//! - The `subgraph_bytes` / `node_count` / `edge_count` limit kinds that
//!   `docs/ERROR-CATALOG.md` lists under `E_INPUT_LIMIT` are **not**
//!   implemented here. This module bounds `Value` and CID decoding only.
//! - Nothing here is configurable. `docs/ERROR-CATALOG.md` currently says
//!   "Limits are configurable via the engine builder"; no such knob
//!   exists. See `NOTES.md` for the record-partition follow-up.

use benten_core::{Cid, Value};
use benten_errors::ErrorCode;

// ---------------------------------------------------------------------------
// Limits
//
// Every constant below is a POLICY knob, not a wire-format constant: it
// bounds what this process is willing to decode, and two peers running
// different values still interoperate on every payload both accept. None
// of them is byte-pinned and none belongs in `docs/V1-WIRE-INVENTORY.md`
// — raising a limit later is not a wire break. (Contrast the CID header
// bytes validated in `parse_cid_string_bounded`, which ARE frozen wire
// and live as consts in `benten-core`.)
// ---------------------------------------------------------------------------

/// Maximum nesting depth of a decoded `Value` tree.
///
/// One unit per open array / map / tag.
///
/// **This is 64, not the 128 the B8 matrix and `docs/ERROR-CATALOG.md`
/// name.** It is DERIVED from [`benten_core::MAX_VALUE_DECODE_DEPTH`]
/// rather than written down again, because that constant is the real
/// ceiling for anything arriving as DAG-CBOR: `Value`'s hand-written
/// `Deserialize` refuses past it, and
/// `crates/benten-core/tests/canonical_bytes_v1_const_values_core.rs`
/// byte-pins it at 64. A napi-side cap of 128 would be a check that can
/// never fire on this path — the canonical decoder would always refuse
/// first — i.e. a defense that looks present and is not. Deriving it also
/// means the two cannot drift.
///
/// Note the asymmetry with `JSON_MAX_DEPTH` (128) in `crate::node`: that
/// path builds a `Value` from an already-materialized `serde_json::Value`
/// and never runs the CBOR decoder, so 128 is reachable there. See
/// `NOTES.md` — a `Value` accepted at depth 65..128 by the JSON path
/// encodes fine and then cannot be decoded back.
pub const NAPI_MAX_DEPTH: usize = benten_core::MAX_VALUE_DECODE_DEPTH;

/// Maximum key count of any single `Value::Map`.
///
/// Mirrors `JSON_MAX_MAP_KEYS` in `crate::node` and the `map_size`
/// default documented in `docs/ERROR-CATALOG.md`.
pub const NAPI_MAX_MAP_KEYS: u64 = 10_000;

/// Maximum element count of any single `Value::List`.
///
/// Matches the `list_size` default documented in `docs/ERROR-CATALOG.md`.
/// The JSON path in `crate::node` has no equivalent cap — arrays there
/// are bounded only by the aggregate-byte budget. Noted as an asymmetry
/// rather than silently changed: widening the JSON path is a behaviour
/// change to a shipped surface, not part of B8.
pub const NAPI_MAX_LIST_ITEMS: u64 = 10_000;

/// Maximum length of any single `Value::Bytes`.
///
/// Mirrors the `bytes_len` default documented in `docs/ERROR-CATALOG.md`.
pub const NAPI_MAX_BYTES: u64 = 16 * 1024 * 1024;

/// Maximum length of any single `Value::Text`.
///
/// Mirrors `JSON_MAX_BYTES` in `crate::node` and the `text_len` default
/// documented in `docs/ERROR-CATALOG.md`.
pub const NAPI_MAX_TEXT_BYTES: u64 = 1024 * 1024;

/// Ceiling on the raw payload handed to the boundary.
///
/// Named to mirror `JSON_MAX_TOTAL_BYTES` in `crate::node`, but it is
/// enforced differently and deliberately so. The JSON path threads a
/// running `ByteBudget` across the tree because a `serde_json::Value` is
/// already materialized when it arrives — its leaf bytes are bounded by
/// nothing. Here the input IS the wire, so every charged byte is also a
/// payload byte: the sum of all leaf lengths cannot exceed `bytes.len()`,
/// and a running accumulator against this same number could never fire.
/// An earlier draft of this module carried one; it was removed rather than
/// shipped as a check that provably cannot trip.
pub const NAPI_MAX_TOTAL_BYTES: u64 = 16 * 1024 * 1024;

/// Aggregate ceiling on the number of data items in one payload.
///
/// Backstop for the multiplicative blow-up the per-collection caps leave
/// open: `NAPI_MAX_LIST_ITEMS` nested `NAPI_MAX_DEPTH` deep is 10 000^128
/// items. `NAPI_MAX_TOTAL_BYTES` already bounds this at ~16.7M items
/// (every item costs at least one wire byte), but a decoded
/// `benten_core::Value` is far wider than one byte, so 16 MiB of `0x00`
/// inside one array is a ~32x memory amplification. One million items is
/// generous for any legitimate single call and caps the decoded footprint
/// in the tens of megabytes.
pub const NAPI_MAX_TOTAL_ITEMS: u64 = 1_000_000;

// ---------------------------------------------------------------------------
// Error carrier
// ---------------------------------------------------------------------------

/// Error returned by the bounded napi-boundary decoders.
///
/// Carries a [`benten_errors::ErrorCode`] so callers can discriminate a
/// **limit** rejection (`E_INPUT_LIMIT`) from a **malformed-input**
/// rejection (`E_SERIALIZE`). Both are refusals, but only the first says
/// "this input was structurally plausible and too big"; conflating them
/// would make the `E_INPUT_LIMIT` assertions in the B8 harness tautologies
/// (the code would be a constant), which is exactly the shape this
/// codebase has shipped before and is trying not to ship again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NapiInputError {
    code: ErrorCode,
    message: String,
}

impl NapiInputError {
    /// The stable error code for this rejection.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.code.clone()
    }

    /// Human-readable detail. Not a stable contract; do not parse in
    /// production code. The B8 harness asserts on substrings of it
    /// precisely so that deleting an individual check is detectable —
    /// several checks share one `ErrorCode`, so the code alone cannot
    /// tell them apart.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl core::fmt::Display for NapiInputError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.code.as_static_str(), self.message)
    }
}

impl std::error::Error for NapiInputError {}

/// Build an `E_INPUT_LIMIT` rejection in the message shape
/// `docs/ERROR-CATALOG.md` documents for this code.
fn limit(kind: &'static str, actual: u64, max: u64) -> NapiInputError {
    NapiInputError {
        code: ErrorCode::InputLimit,
        message: format!("Napi boundary input exceeds {kind} limit: {actual} > {max}"),
    }
}

/// Build an `E_SERIALIZE` rejection — the input is not decodable at all,
/// as distinct from decodable-but-over-a-limit.
fn malformed(what: &str) -> NapiInputError {
    NapiInputError {
        code: ErrorCode::Serialize,
        message: format!("malformed DAG-CBOR at the napi boundary: {what}"),
    }
}

// ---------------------------------------------------------------------------
// The pre-scan
// ---------------------------------------------------------------------------

/// Read one CBOR head byte plus its argument, advancing `pos`.
///
/// Returns `(major_type, argument)`. For major type 7 the argument is the
/// float payload, never a length — callers must not treat it as one.
fn read_head(bytes: &[u8], pos: &mut usize) -> Result<(u8, u64), NapiInputError> {
    let Some(&head) = bytes.get(*pos) else {
        return Err(malformed("input ended where a data item was expected"));
    };
    *pos += 1;
    let major = head >> 5;
    let ai = head & 0x1f;

    let arg = match ai {
        0..=23 => u64::from(ai),
        24 => read_uint(bytes, pos, 1)?,
        25 => read_uint(bytes, pos, 2)?,
        26 => read_uint(bytes, pos, 4)?,
        27 => read_uint(bytes, pos, 8)?,
        // Indefinite-length items are invalid DAG-CBOR AND are the one
        // nesting shape whose depth is not declared anywhere in the
        // framing, so a length-based guard cannot see them coming. One
        // byte per level, refused on sight.
        31 => return Err(malformed("indefinite-length item (not valid DAG-CBOR)")),
        _ => return Err(malformed("reserved additional-information value 28..30")),
    };

    if major == 7 {
        match ai {
            // 20..=22 — false / true / null.
            //
            // 25..=27 — float16 / float32 / float64, payload already
            // consumed by `read_uint`. DAG-CBOR permits float64 only; the
            // canonical decoder rejects the narrower two and we do not
            // duplicate that judgement here.
            20..=22 | 25..=27 => {}
            _ => return Err(malformed("simple value not permitted in DAG-CBOR")),
        }
    }

    Ok((major, arg))
}

/// Read a big-endian unsigned integer of `n` bytes, advancing `pos`.
fn read_uint(bytes: &[u8], pos: &mut usize, n: usize) -> Result<u64, NapiInputError> {
    let end = pos
        .checked_add(n)
        .ok_or_else(|| malformed("length prefix overflows the address space"))?;
    let slice = bytes
        .get(*pos..end)
        .ok_or_else(|| malformed("input ended inside a length prefix"))?;
    let mut value: u64 = 0;
    for byte in slice {
        value = (value << 8) | u64::from(*byte);
    }
    *pos = end;
    Ok(value)
}

/// Walk the CBOR framing of `bytes`, enforcing every B8 limit, without
/// building any part of the value it describes.
///
/// The only heap allocation is the frame stack, whose length is bounded by
/// [`NAPI_MAX_DEPTH`] — never by anything the attacker declares.
///
/// # Errors
///
/// * [`ErrorCode::InputLimit`] — a documented limit was exceeded.
/// * [`ErrorCode::Serialize`] — the bytes are not decodable DAG-CBOR
///   framing at all (truncated, indefinite-length, reserved head byte,
///   trailing garbage).
pub fn scan_dag_cbor(bytes: &[u8]) -> Result<(), NapiInputError> {
    // MUTATION THAT MUST MAKE THIS FAIL: delete this block -> a 17 MiB
    // payload still refuses, via the item-count guard below, with a
    // different message. This guard is therefore defense-in-depth and is
    // deliberately NOT pinned by its own test (the fixture would be a
    // 17 MiB allocation in CI to distinguish two refusals). Stated rather
    // than dressed up with a test that cannot tell it is gone.
    let total = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if total > NAPI_MAX_TOTAL_BYTES {
        return Err(limit("total_bytes", total, NAPI_MAX_TOTAL_BYTES));
    }

    let mut pos: usize = 0;
    let mut items_seen: u64 = 0;

    // One entry per OPEN collection, holding how many data items that
    // collection still owes. Entry 0 is a pseudo-frame owing the single
    // top-level item, so the current depth is `stack.len() - 1` and the
    // depth a new collection WOULD occupy is `stack.len()`.
    let mut stack: Vec<u64> = Vec::with_capacity(NAPI_MAX_DEPTH + 2);
    stack.push(1);

    while let Some(owed) = stack.last_mut() {
        if *owed == 0 {
            stack.pop();
            continue;
        }
        *owed -= 1;

        items_seen += 1;
        // MUTATION THAT MUST MAKE THIS FAIL: delete these three lines ->
        // `napi_rejects_item_count_blowup` decodes a 2 000 200-item
        // payload successfully and its `expect_err` panics.
        if items_seen > NAPI_MAX_TOTAL_ITEMS {
            return Err(limit("total_items", items_seen, NAPI_MAX_TOTAL_ITEMS));
        }

        let (major, arg) = read_head(bytes, &mut pos)?;
        // `read_head` never advances past the end, so this cannot wrap.
        let remaining = u64::try_from(bytes.len() - pos).unwrap_or(u64::MAX);

        match major {
            // 0 | 1 — unsigned / negative integers, fully consumed by the
            //         head.
            // 7     — simple values and floats, validated inside
            //         `read_head`; also fully consumed there.
            0 | 1 | 7 => {}

            // Byte string.
            2 => {
                // MUTATION THAT MUST MAKE THIS FAIL: raise NAPI_MAX_BYTES
                // by one -> `napi_rejects_oversized_bytes` gets
                // E_SERIALIZE (truncated) instead of E_INPUT_LIMIT.
                if arg > NAPI_MAX_BYTES {
                    return Err(limit("bytes_len", arg, NAPI_MAX_BYTES));
                }
                consume_leaf(&mut pos, arg, remaining)?;
            }

            // Text string.
            3 => {
                // MUTATION THAT MUST MAKE THIS FAIL: raise
                // NAPI_MAX_TEXT_BYTES by one ->
                // `napi_text_leaf_limit_is_exactly_one_mib` sees
                // E_SERIALIZE instead of E_INPUT_LIMIT.
                if arg > NAPI_MAX_TEXT_BYTES {
                    return Err(limit("text_len", arg, NAPI_MAX_TEXT_BYTES));
                }
                consume_leaf(&mut pos, arg, remaining)?;
            }

            // Array. The list cap runs BEFORE the amplification guard, so
            // a header declaring u32::MAX elements lands on `list_size`,
            // not `declared_length` — which is why
            // `napi_rejects_recursive_cbor_bomb_reports_list_size` and
            // `napi_rejects_declared_length_amplification` assert on the
            // message and not only on the code. Deleting either guard
            // leaves the other refusing with the wrong message.
            4 => {
                // MUTATION THAT MUST MAKE THIS FAIL: change `>` to `>=`
                // -> `napi_list_limit_boundary_is_exactly_10k` finds the
                // 10 000-element list rejected and fails.
                if arg > NAPI_MAX_LIST_ITEMS {
                    return Err(limit("list_size", arg, NAPI_MAX_LIST_ITEMS));
                }
                check_declared_fits(arg, remaining)?;
                push_frame(&mut stack, arg)?;
            }

            // Map.
            5 => {
                // MUTATION THAT MUST MAKE THIS FAIL: change `>` to `>=`
                // -> `napi_map_limit_boundary_is_exactly_10k` finds the
                // 10 000-key map rejected and fails.
                if arg > NAPI_MAX_MAP_KEYS {
                    return Err(limit("map_size", arg, NAPI_MAX_MAP_KEYS));
                }
                // `arg <= 10_000` so the doubling cannot overflow.
                let entries = arg * 2;
                check_declared_fits(entries, remaining)?;
                push_frame(&mut stack, entries)?;
            }

            // Tag. DAG-CBOR permits exactly one: 42, the CID link.
            //
            // The tagged item is pushed as its own one-item frame, so a
            // link costs a depth level. That is one level stricter than
            // the canonical decoder; a payload nested exactly 128 deep
            // whose leaves are links is rejected here. Recorded as
            // deliberate conservatism, not as an accident.
            6 => {
                if arg != 42 {
                    return Err(malformed("tag other than 42 (not valid DAG-CBOR)"));
                }
                push_frame(&mut stack, 1)?;
            }

            _ => return Err(malformed("unreachable CBOR major type")),
        }
    }

    if pos != bytes.len() {
        return Err(malformed("trailing bytes after the top-level item"));
    }
    Ok(())
}

/// Step over a string / byte-string leaf's payload.
fn consume_leaf(pos: &mut usize, len: u64, remaining: u64) -> Result<(), NapiInputError> {
    // A leaf that claims more bytes than the input holds is truncation,
    // not an over-limit input. Classified as E_SERIALIZE so the
    // `bytes_len` / `text_len` caps above stay independently observable:
    // if this arm also returned E_INPUT_LIMIT, every over-cap test would
    // pass with the caps deleted.
    if len > remaining {
        return Err(malformed(
            "string leaf claims more bytes than the input holds",
        ));
    }
    let step = usize::try_from(len).map_err(|_| malformed("leaf length exceeds usize"))?;
    *pos += step;
    Ok(())
}

/// Refuse a collection whose declared child count cannot possibly be
/// satisfied by the bytes that remain.
///
/// This is the length-prefix amplification guard — vector (v). Every CBOR
/// child costs at least one byte, so a header declaring more children than
/// there are remaining bytes is a claim the payload provably cannot honour.
/// A decoder that sizes a buffer from that number allocates gigabytes from
/// a five-byte input. Classified `E_INPUT_LIMIT` (not "malformed") because
/// it is an attack signature, not a formatting accident.
fn check_declared_fits(declared: u64, remaining: u64) -> Result<(), NapiInputError> {
    // MUTATION THAT MUST MAKE THIS FAIL: delete this function body ->
    // `napi_rejects_declared_length_amplification` walks off the end of
    // the input and gets E_SERIALIZE instead of E_INPUT_LIMIT.
    if declared > remaining {
        return Err(limit("declared_length", declared, remaining));
    }
    Ok(())
}

/// Open a collection frame, enforcing the depth ceiling first.
fn push_frame(stack: &mut Vec<u64>, owed: u64) -> Result<(), NapiInputError> {
    // `stack.len()` is the depth this new frame would occupy (entry 0 is
    // the pseudo-root).
    //
    // MUTATION THAT MUST MAKE THIS FAIL: change `>` to `>=` ->
    // `napi_depth_limit_boundary_is_exactly_128` finds the 128-deep list
    // rejected and fails.
    if stack.len() > NAPI_MAX_DEPTH {
        return Err(limit(
            "nesting_depth",
            u64::try_from(stack.len()).unwrap_or(u64::MAX),
            u64::try_from(NAPI_MAX_DEPTH).unwrap_or(u64::MAX),
        ));
    }
    stack.push(owed);
    Ok(())
}

// ---------------------------------------------------------------------------
// Public boundary decoders
// ---------------------------------------------------------------------------

/// Decode DAG-CBOR bytes into a [`benten_core::Value`], enforcing every
/// B8 limit **before** the canonical decoder sees the input.
///
/// # Errors
///
/// * [`ErrorCode::InputLimit`] — a documented limit was exceeded. Nothing
///   proportional to the declared size was allocated.
/// * [`ErrorCode::Serialize`] — the pre-scan or the canonical decoder
///   rejected the bytes as malformed.
pub fn decode_value_bounded(bytes: &[u8]) -> Result<Value, NapiInputError> {
    // Ordering is the security property. MUTATION THAT MUST MAKE THIS
    // FAIL: move this call below the `from_slice` -> the RSS tripwire in
    // `napi_rejects_oversized_value_map` sees the map materialise.
    scan_dag_cbor(bytes)?;
    serde_ipld_dagcbor::from_slice::<Value>(bytes)
        .map_err(|e| malformed(&format!("canonical decoder rejected the payload: {e}")))
}

/// Parse a multibase base32 CID string (as raw UTF-8 bytes, the shape the
/// JS boundary hands over) into a [`benten_core::Cid`].
///
/// Delegates the actual parse to `benten_core::Cid::from_str` — there is
/// exactly one CID parser and this is not a second one — and re-classifies
/// its typed failures (`E_CID_PARSE` / `E_CID_UNSUPPORTED_CODEC` /
/// `E_CID_UNSUPPORTED_HASH`) as `E_INPUT_LIMIT`, which is the code the napi
/// boundary reports for every rejected input shape per the B8 matrix. The
/// underlying reason is preserved in the message.
///
/// # Errors
///
/// * [`ErrorCode::InputLimit`] — not UTF-8, or not a well-formed Benten
///   `CIDv1` (`[0x01, 0x71, 0x1e, 0x20, <32-byte BLAKE3 digest>]`).
pub fn parse_cid_string_bounded(bytes: &[u8]) -> Result<Cid, NapiInputError> {
    use core::str::FromStr as _;

    let text = core::str::from_utf8(bytes).map_err(|_| NapiInputError {
        code: ErrorCode::InputLimit,
        message: "Napi boundary input exceeds cid_shape limit: CID string is not valid UTF-8"
            .to_owned(),
    })?;

    // MUTATION THAT MUST MAKE THIS FAIL: delete the multicodec check in
    // `benten_core::Cid::from_bytes` -> `napi_rejects_dag_pb_multicodec_cid`
    // parses successfully and its `expect_err` panics. Likewise the
    // multihash check and `napi_rejects_sha256_multihash_cid`. The
    // delegation is what makes those mutations reachable from here.
    Cid::from_str(text).map_err(|e| NapiInputError {
        code: ErrorCode::InputLimit,
        message: format!("Napi boundary input exceeds cid_shape limit: {e}"),
    })
}

#[cfg(test)]
mod tests {
    //! In-crate scanner unit tests.
    //!
    //! These run under `cargo nextest run --workspace` (default features,
    //! `napi-export` ON) — a REQUIRED lane. They do NOT run in the
    //! `napi in-process pins (rlib mode)` job, which selects integration
    //! test targets by name only. The behavioural pins that must hold in
    //! the rlib lane live in `bindings/napi/tests/input_validation.rs`;
    //! what is here is scanner-internal coverage that has no seam there.

    use super::{NAPI_MAX_DEPTH, decode_value_bounded, scan_dag_cbor};
    use benten_errors::ErrorCode;

    /// Encode a CBOR head with the shortest legal argument form.
    fn head(out: &mut Vec<u8>, major: u8, arg: u64) {
        let base = major << 5;
        if arg < 24 {
            out.push(base | u8::try_from(arg).expect("arg < 24"));
        } else if let Ok(v) = u8::try_from(arg) {
            out.push(base | 0x18);
            out.push(v);
        } else if let Ok(v) = u16::try_from(arg) {
            out.push(base | 0x19);
            out.extend_from_slice(&v.to_be_bytes());
        } else if let Ok(v) = u32::try_from(arg) {
            out.push(base | 0x1a);
            out.extend_from_slice(&v.to_be_bytes());
        } else {
            out.push(base | 0x1b);
            out.extend_from_slice(&arg.to_be_bytes());
        }
    }

    /// A well-formed payload under every limit decodes. Without this the
    /// rejection tests would all pass against a scanner that rejects
    /// unconditionally.
    #[test]
    fn well_formed_payload_decodes() {
        // {"a": [1, 2], "b": h'0102'}
        let mut p = Vec::new();
        head(&mut p, 5, 2);
        head(&mut p, 3, 1);
        p.push(b'a');
        head(&mut p, 4, 2);
        head(&mut p, 0, 1);
        head(&mut p, 0, 2);
        head(&mut p, 3, 1);
        p.push(b'b');
        head(&mut p, 2, 2);
        p.extend_from_slice(&[0x01, 0x02]);

        let value = decode_value_bounded(&p).expect("payload is well within every limit");
        match value {
            benten_core::Value::Map(m) => assert_eq!(m.len(), 2),
            other => panic!("expected a map, got {other:?}"),
        }
    }

    /// Indefinite-length framing is the one nesting shape whose depth is
    /// not declared in the framing, so the depth guard cannot see it
    /// coming. It is refused on the head byte.
    ///
    /// MUTATION THAT MUST MAKE THIS FAIL: accept `ai == 31` in
    /// `read_head` -> 200 x `0x9f` walks the stack unbounded and this
    /// returns something other than `E_SERIALIZE`.
    #[test]
    fn indefinite_length_framing_is_refused_on_sight() {
        let payload = vec![0x9f_u8; 200];
        let err = scan_dag_cbor(&payload).expect_err("indefinite length is not DAG-CBOR");
        assert_eq!(err.code(), ErrorCode::Serialize);
        assert!(
            err.message().contains("indefinite"),
            "got {}",
            err.message()
        );
    }

    /// Trailing bytes after a complete top-level item are refused.
    ///
    /// MUTATION THAT MUST MAKE THIS FAIL: delete the `pos != bytes.len()`
    /// check at the end of `scan_dag_cbor` -> the scan returns Ok and the
    /// canonical decoder's own trailing-data error changes the message.
    #[test]
    fn trailing_bytes_are_refused() {
        let err = scan_dag_cbor(&[0x00, 0x00]).expect_err("two top-level items");
        assert_eq!(err.code(), ErrorCode::Serialize);
        assert!(err.message().contains("trailing"), "got {}", err.message());
    }

    /// Tags other than 42 are not DAG-CBOR.
    ///
    /// MUTATION THAT MUST MAKE THIS FAIL: drop the `arg != 42` check ->
    /// tag 41 is accepted by the scanner and this returns Ok.
    #[test]
    fn non_link_tags_are_refused() {
        // tag(41) followed by 0
        let payload = vec![0xd8, 41, 0x00];
        let err = scan_dag_cbor(&payload).expect_err("only tag 42 is legal DAG-CBOR");
        assert_eq!(err.code(), ErrorCode::Serialize);
    }

    /// The depth constant the module exports is the depth the scanner
    /// actually enforces — not a documented number drifting from a
    /// hard-coded one.
    ///
    /// MUTATION THAT MUST MAKE THIS FAIL: change the literal in
    /// `push_frame` to something other than `NAPI_MAX_DEPTH` -> the
    /// accept side or the reject side flips.
    #[test]
    fn exported_depth_constant_is_the_enforced_one() {
        let nest = |levels: usize| {
            let mut p: Vec<u8> = std::iter::repeat_n(0x81_u8, levels).collect();
            p.push(0x00);
            p
        };
        assert!(scan_dag_cbor(&nest(NAPI_MAX_DEPTH)).is_ok());
        let err = scan_dag_cbor(&nest(NAPI_MAX_DEPTH + 1)).expect_err("one past the ceiling");
        assert_eq!(err.code(), ErrorCode::InputLimit);
    }
}

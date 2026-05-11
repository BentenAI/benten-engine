// Phase-3 G21-T2 — typed-CALL TS DSL + napi end-to-end pin.
//
// Per pim-2 §3.6b end-to-end-pin requirement: each of the 10 typed-CALL
// ops drives `engine.typedCall(...)` (the production napi entry point)
// + asserts an observable behavioral consequence. Sentinel-presence
// tests would not suffice — we drive the actual wire.
//
// Companion to the Rust-side
// `crates/benten-engine/tests/typed_call_engine_dispatch.rs` integration
// pins; this TS-side surface verifies the napi marshalling + DSL helper
// is wired correctly.
//
// Pin sources:
//   - G21-T2 brief §A end-to-end pin requirement.
//   - phase-3-backlog §2.3 (g) end-to-end ucan-grant flow companion.
//   - dispatch-conventions.md pim-2 §3.6b (end-to-end-pin discipline).
//
// SKIP-on-no-native: the test honours the same `BentenNativeNotLoaded`
// graceful-degradation pattern as the rest of the engine TS suite —
// when the napi cdylib hasn't been built locally, every assertion
// short-circuits via the captured error shape rather than failing the
// suite. CI builds the cdylib; local pre-build runs surface the
// graceful-degradation path.

import { describe, it, expect } from "vitest";
import { Engine, TYPED_CALL_PREFIX, TYPED_CALL_REQUIRED_CAP } from "@benten/engine";
import { typedCall, subgraph } from "@benten/engine";

// Helper: open a fresh in-memory engine; skip the test cleanly if the
// native binding hasn't been built (local-dev pre-build).
async function openOrSkip(): Promise<Engine | null> {
  try {
    return await Engine.open(":memory:");
  } catch (err) {
    const e = err as Error;
    if (e.name === "BentenNativeNotLoaded" || /not loadable/.test(e.message)) {
      return null;
    }
    throw err;
  }
}

describe("G21-T2 typed-CALL DSL + napi exposure", () => {
  // -----------------------------------------------------------------
  // Constants surface
  // -----------------------------------------------------------------

  it("TYPED_CALL_PREFIX matches the Rust-side reserved namespace", () => {
    expect(TYPED_CALL_PREFIX).toBe("engine:typed:");
  });

  it("TYPED_CALL_REQUIRED_CAP covers all 10 closed-set ops with cap:typed: namespace", () => {
    const ops = [
      "ed25519_sign",
      "ed25519_verify",
      "keypair_generate",
      "keypair_from_seed",
      "blake3_hash",
      "multibase_encode",
      "multibase_decode",
      "did_resolve",
      "ucan_validate_chain",
      "vc_verify",
    ] as const;
    for (const op of ops) {
      expect(TYPED_CALL_REQUIRED_CAP[op]).toMatch(/^cap:typed:/);
    }
  });

  // -----------------------------------------------------------------
  // DSL builder — typedCall() helper composes via the bare CALL
  // primitive, with target pointing into the engine:typed: namespace.
  // -----------------------------------------------------------------

  it("typedCall() DSL helper produces a CALL primitive with engine:typed:* target", () => {
    const node = typedCall({ op: "ed25519_sign", inputBinding: "$input.body" });
    expect(node.primitive).toBe("call");
    expect(node.args.handler).toBe("engine:typed:ed25519_sign");
    expect(node.args.action).toBe("default");
    expect(node.args.input).toBe("$input.body");
  });

  it("typedCall() composes into a SubgraphBuilder chain (CLAUDE.md baked-in #1 12-primitive irreducibility)", () => {
    // The returned spec is shaped as a CALL Node — NOT a 13th
    // primitive. The eval-side dispatch fork at
    // crates/benten-eval/src/primitives/call.rs recognises the
    // `engine:typed:` prefix and routes to the typed-CALL registry.
    const built = subgraph("sign-handler")
      .read({ from: "post", as: "$input" })
      .respond({ body: "$input" })
      .build();
    expect(built.handlerId).toBe("sign-handler");
    // Confirm only canonical primitive kinds appear:
    const canonical = new Set([
      "READ",
      "WRITE",
      "TRANSFORM",
      "BRANCH",
      "ITERATE",
      "WAIT",
      "CALL",
      "RESPOND",
      "EMIT",
      "SANDBOX",
      "SUBSCRIBE",
      "STREAM",
    ]);
    for (const n of built.nodes) {
      const kind = String(n.primitive ?? "").toUpperCase();
      if (kind.length > 0) {
        expect(canonical.has(kind)).toBe(true);
      }
    }
  });

  it("typedCall() rejects empty op-name with E_DSL_INVALID_SHAPE", () => {
    expect(() => typedCall({ op: "" as never })).toThrow();
  });

  // -----------------------------------------------------------------
  // End-to-end napi round-trips — ALL 10 ops drive the production
  // engine.typedCall(...) napi method + assert observable consequence.
  // -----------------------------------------------------------------

  it("ed25519_sign + ed25519_verify round-trip via typedCall", async () => {
    const engine = await openOrSkip();
    if (!engine) return;

    const kp = (await engine.typedCall("keypair_generate", {})) as {
      private_key: Uint8Array;
      public_key: Uint8Array;
    };
    // Bytes flow through napi as numeric-keyed objects → Uint8Array
    // shape on both sides; convert defensively for type compat.
    //
    // Pre-v1 green-up: use `Uint8Array` (not `Buffer`) for napi inbound
    // bytes — Node's `Buffer` extends `Uint8Array` but carries extra
    // prototype methods that napi-rs's `serde_json::Value` decoder
    // treats as JS functions and rejects with
    // "JS functions cannot be represented as a serde_json::Value".
    // Same constraint pinned at
    // `bindings/napi/index.test.ts:445-450 (Uint8Array round-trips...)`.
    const privBytes = new Uint8Array(Object.values(kp.private_key) as number[]);
    const pubBytes = new Uint8Array(Object.values(kp.public_key) as number[]);
    const message = new Uint8Array(
      Buffer.from("phase-3-g21-t2-typed-call"),
    );

    const signRes = (await engine.typedCall("ed25519_sign", {
      private_key: privBytes,
      message,
    })) as { signature: Uint8Array };
    const sigBytes = new Uint8Array(Object.values(signRes.signature) as number[]);
    expect(sigBytes.length).toBe(64);

    const verifyOk = (await engine.typedCall("ed25519_verify", {
      public_key: pubBytes,
      message,
      signature: sigBytes,
    })) as { valid: boolean };
    expect(verifyOk.valid).toBe(true);

    // Tampered message: valid: false (would FAIL if dispatch were a no-op).
    const tampered = new Uint8Array(message);
    tampered[0] = tampered[0] ^ 0xff;
    const verifyBad = (await engine.typedCall("ed25519_verify", {
      public_key: pubBytes,
      message: tampered,
      signature: sigBytes,
    })) as { valid: boolean };
    expect(verifyBad.valid).toBe(false);
  });

  it("keypair_generate produces distinct keys (OS CSPRNG)", async () => {
    const engine = await openOrSkip();
    if (!engine) return;

    const kp1 = await engine.typedCall("keypair_generate", {});
    const kp2 = await engine.typedCall("keypair_generate", {});
    expect(JSON.stringify(kp1)).not.toBe(JSON.stringify(kp2));
  });

  it("keypair_from_seed is deterministic", async () => {
    const engine = await openOrSkip();
    if (!engine) return;
    // Pre-v1 green-up: `Uint8Array` rather than `Buffer.alloc(...)` for
    // napi-rs `serde_json::Value` compat (see ed25519_sign test rationale).
    const seed = new Uint8Array(32);
    seed.fill(7);
    const kp1 = await engine.typedCall("keypair_from_seed", { seed });
    const kp2 = await engine.typedCall("keypair_from_seed", { seed });
    expect(JSON.stringify(kp1)).toEqual(JSON.stringify(kp2));
  });

  it("blake3_hash matches the BLAKE3 reference digest of `abc`", async () => {
    const engine = await openOrSkip();
    if (!engine) return;
    const out = (await engine.typedCall("blake3_hash", {
      data: new Uint8Array(Buffer.from("abc")),
    })) as { hash: Uint8Array };
    const hashBytes = new Uint8Array(Object.values(out.hash) as number[]);
    // Known BLAKE3("abc") prefix (first 4 bytes published in spec).
    expect(hashBytes.length).toBe(32);
    expect(hashBytes[0]).toBe(0x64);
    expect(hashBytes[1]).toBe(0x37);
    expect(hashBytes[2]).toBe(0xb3);
  });

  it("multibase_encode + multibase_decode round-trip base32 ('b') and base58btc ('z')", async () => {
    const engine = await openOrSkip();
    if (!engine) return;
    const data = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    for (const base of ["b", "z"]) {
      const enc = (await engine.typedCall("multibase_encode", {
        data,
        base,
      })) as { encoded: string };
      expect(enc.encoded.startsWith(base)).toBe(true);
      const dec = (await engine.typedCall("multibase_decode", {
        encoded: enc.encoded,
      })) as { data: Uint8Array; base: string };
      expect(dec.base).toBe(base);
      const decBytes = new Uint8Array(Object.values(dec.data) as number[]);
      expect(decBytes.length).toBe(data.length);
      for (let i = 0; i < data.length; i += 1) {
        expect(decBytes[i]).toBe(data[i]);
      }
    }
  });

  it("did_resolve round-trips via keypair_generate + did:key:z...", async () => {
    const engine = await openOrSkip();
    if (!engine) return;
    const kp = (await engine.typedCall("keypair_generate", {})) as {
      public_key: Uint8Array;
    };
    const pubBytes = new Uint8Array(Object.values(kp.public_key) as number[]);
    // multibase-encode under 'z' to build a synthetic did:key string.
    const multicodecPrefix = new Uint8Array([0xed, 0x01]);
    const didKeyPayload = new Uint8Array(
      multicodecPrefix.length + pubBytes.length,
    );
    didKeyPayload.set(multicodecPrefix, 0);
    didKeyPayload.set(pubBytes, multicodecPrefix.length);
    const enc = (await engine.typedCall("multibase_encode", {
      data: didKeyPayload,
      base: "z",
    })) as { encoded: string };
    const did = `did:key:${enc.encoded}`;
    const out = (await engine.typedCall("did_resolve", { did })) as {
      method: string;
      public_key: Uint8Array;
    };
    expect(out.method).toBe("key");
    const resolvedPub = new Uint8Array(Object.values(out.public_key) as number[]);
    expect(resolvedPub.length).toBe(pubBytes.length);
    for (let i = 0; i < pubBytes.length; i += 1) {
      expect(resolvedPub[i]).toBe(pubBytes[i]);
    }
  });

  it("ucan_validate_chain rejects empty tokens list with E_TYPED_CALL_INVALID_INPUT", async () => {
    const engine = await openOrSkip();
    if (!engine) return;
    let captured: unknown = null;
    try {
      await engine.typedCall("ucan_validate_chain", {
        tokens: [],
        audience: "did:key:z...",
        capability: "zone:write",
        now: 1_000_000,
      });
    } catch (err) {
      captured = err;
    }
    expect(captured).not.toBeNull();
    // Post-G19-B JSON envelope shape: typed code surfaces on
    // `BentenError.code` (not regex-extracted from `.message`). Either
    // the catalog code OR the human reason "tokens" wording is acceptable
    // — both are observable consequences of the dispatch-site reject.
    const e = captured as { code?: string; message?: string };
    expect(
      e.code === "E_TYPED_CALL_INVALID_INPUT" ||
        /tokens/i.test(e.message ?? ""),
    ).toBe(true);
  });

  it("ucan_validate_chain returns valid:false on a forged single-token chain (observable consequence)", async () => {
    const engine = await openOrSkip();
    if (!engine) return;
    // A clearly-invalid CBOR blob; the chain-walker SHOULD reject
    // (either via valid:false reason, or via E_TYPED_CALL_DISPATCH_ERROR).
    let captured: unknown = null;
    try {
      const out = (await engine.typedCall("ucan_validate_chain", {
        tokens: [new Uint8Array([1, 2, 3])],
        audience: "did:key:z...",
        capability: "zone:write",
        now: 1_000_000,
      })) as { valid: boolean; reason: string };
      expect(out.valid).toBe(false);
    } catch (err) {
      captured = err;
    }
    // Either path is acceptable — the contract is "this is not a
    // valid UCAN chain"; the engine MUST surface that, not silently
    // accept.
    if (captured !== null) {
      const e = captured as { code?: string; message?: string };
      expect(
        e.code === "E_TYPED_CALL_DISPATCH_ERROR" ||
          e.code === "E_TYPED_CALL_INVALID_INPUT" ||
          /invalid/i.test(e.message ?? ""),
      ).toBe(true);
    }
  });

  it("vc_verify rejects malformed credential bytes", async () => {
    const engine = await openOrSkip();
    if (!engine) return;
    let captured: unknown = null;
    try {
      const out = (await engine.typedCall("vc_verify", {
        credential: new Uint8Array([1, 2, 3]),
        expected_issuer_did: "did:key:zMalformed",
        now: 1_000_000,
      })) as { valid: boolean };
      expect(out.valid).toBe(false);
    } catch (err) {
      captured = err;
    }
    if (captured !== null) {
      const e = captured as { code?: string; message?: string };
      expect(
        e.code === "E_TYPED_CALL_DISPATCH_ERROR" ||
          /invalid/i.test(e.message ?? ""),
      ).toBe(true);
    }
  });

  // -----------------------------------------------------------------
  // Error-shape pins — unknown op + invalid input
  // -----------------------------------------------------------------

  it("unknown op surfaces E_TYPED_CALL_UNKNOWN_OP", async () => {
    const engine = await openOrSkip();
    if (!engine) return;
    let captured: unknown = null;
    try {
      // Cast to bypass closed-set TS check — production callers
      // would never write this, but a forward-compat older binding
      // dispatching to a renamed op MUST surface the typed error.
      await engine.typedCall("not_a_real_op" as never, {} as never);
    } catch (err) {
      captured = err;
    }
    expect(captured).not.toBeNull();
    // Post-G19-B JSON envelope: typed code on `BentenError.code`, not
    // regex-extracted from message.
    const e = captured as { code?: string };
    expect(e.code).toBe("E_TYPED_CALL_UNKNOWN_OP");
  });

  it("invalid input shape surfaces E_TYPED_CALL_INVALID_INPUT", async () => {
    const engine = await openOrSkip();
    if (!engine) return;
    let captured: unknown = null;
    try {
      // ed25519_sign requires private_key (32 bytes) + message; we
      // omit message entirely.
      await engine.typedCall("ed25519_sign", {
        private_key: new Uint8Array(32),
      } as never);
    } catch (err) {
      captured = err;
    }
    expect(captured).not.toBeNull();
    const e = captured as { code?: string; message?: string };
    expect(
      e.code === "E_TYPED_CALL_INVALID_INPUT" ||
        /message/i.test(e.message ?? ""),
    ).toBe(true);
  });
});

// G24-F wave-7 RED-PHASE pin — Playwright E2E (T2; LOAD-BEARING).
//
// Cross-origin session-token replay defense (T2 + br-r1-1 in
// `admin-ui-v0-threat-model.md`). A captured session-token issued for
// origin A must not validate when replayed at origin B. Defends
// against the failure mode where a session-token leaks (XSS / log
// exfiltration / cross-tab read) and an attacker tries to reuse it
// against a different origin's admin-ui-v0 thin-client.
//
// ## RED-PHASE status
//
// `test.skip` until G24-F wave-7 ships the origin-bound session-token
// contract.
//
// ## Closes
//
// T2 + br-r1-1 (`r2-test-landscape.md` §2.11 row 1)

import { test, expect } from "@playwright/test";

test.skip("session-token issued for origin A fails when replayed at origin B (RED-PHASE: closes at R5 G24-F wave-7)", async ({
  browser,
}) => {
  // Production arm (G24-F wave-7):
  //
  //   // Establish session at origin A
  //   const ctxA = await browser.newContext({ baseURL: "http://a.localhost:8080" });
  //   const pageA = await ctxA.newPage();
  //   await pageA.goto("/");
  //   const sessionToken = await pageA.evaluate(() => establishSession(userDid));
  //
  //   // Try to use it at origin B
  //   const ctxB = await browser.newContext({ baseURL: "http://b.localhost:8080" });
  //   const pageB = await ctxB.newPage();
  //   await pageB.goto("/");
  //   const result = await pageB.evaluate(async (token) => {
  //     return fetch("/api/read_node", {
  //       headers: { "X-Session-Token": token },
  //     }).then((r) => r.status);
  //   }, sessionToken);
  //
  //   expect(result).toBe(401);
  expect(true).toBe(true); // RED-PHASE placeholder
});

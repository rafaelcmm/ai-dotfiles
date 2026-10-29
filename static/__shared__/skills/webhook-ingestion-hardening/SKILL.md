---
name: webhook-ingestion-hardening
description: >-
  Receive third-party webhooks safely — HMAC verification over raw bytes,
  fail-closed signature checks, correct status codes so a provider does not
  retry-storm you, idempotency for at-least-once delivery, and PII-safe logging.
  Use this skill when the task involves "webhook handler", "verify webhook
  signature", "HMAC", "X-Shopify-Hmac-Sha256", "webhook returns 401", "webhook
  keeps retrying", "orders/create", "payment/review/shipping provider callback",
  "signature mismatch", or adding any inbound endpoint a third party calls.
---

# Webhook ingestion hardening

An inbound webhook is an **unauthenticated public endpoint until you prove
otherwise**. It can fabricate orders, revenue, published content, and state
changes. Everything below follows from that.

## Verify over the raw bytes

Read the raw body **before anything parses it**, and compute the HMAC over those
exact bytes.

```ts
const rawBody = await request.arrayBuffer();          // first, always
const signature = request.headers.get(SIGNATURE_HEADER);

if (!(await verifySignature(rawBody, signature))) {
  return new Response("Invalid signature", { status: 401 });
}

const payload = JSON.parse(new TextDecoder().decode(rawBody));
```

`JSON.stringify(JSON.parse(x))` is **not** byte-identical to `x` — it reorders
keys, normalises whitespace, and reformats numbers. A verifier fed a
re-serialised body silently stops matching. This is the single most common way
webhook verification is written wrong, and it fails *closed*, so it presents as
"the provider's signatures are broken" rather than as your own bug.

## Fail closed, in every ambiguous case

```ts
const secret = process.env.WEBHOOK_SECRET;
if (!secret) {
  console.warn("[webhook] secret not set — rejecting all requests");
  return false;
}
if (!signature) return false;
```

Reject on: absent secret, absent signature header, length mismatch, malformed
encoding, any thrown comparison. There is **no degraded mode** where an
unverified request is processed anyway — "accept while the secret is missing"
turns an environment misconfiguration into an open endpoint that writes to your
data.

A useful consequence: because the route fails closed, a **typo'd header-name
constant** reads as "no signature present" and is correctly rejected. Fail-closed
is what makes that constant safe to export. Under a fail-open design the same
typo would accept everything.

## Constant-time comparison

```ts
function equals(a: Buffer, b: Buffer): boolean {
  // timingSafeEqual throws on differing lengths. Compare length first and treat
  // a mismatch as a plain failure — a digest's length is fixed and public, so
  // short-circuiting on it leaks nothing.
  if (a.length !== b.length) return false;
  try {
    return timingSafeEqual(a, b);
  } catch {
    return false;
  }
}
```

If the provider does not document whether the digest is hex or base64, compare
**both encodings and always run both comparisons**, so elapsed time reveals
neither which matched nor that neither did. Guessing one encoding silently
disables the whole path: deliveries keep arriving, the endpoint keeps answering,
and nothing ever happens.

Likewise, if the provider's own material shows more than one header *name*
(not just casing — `Headers.get` is already case-insensitive), accept the known
list rather than betting on one.

## One shared verifier

Every webhook route calls the same verifier module. A copy-pasted verifier
decays dangerously: one copy gets a fix the other does not, and the divergence
is **invisible until an unsigned request is accepted**. Extract it the moment
there is a second route — not later.

## Status codes: two distinct cases

This is where retry storms come from.

**Unauthenticated request → reject outright (401), do no work.**

**Authenticated request whose processing fails → still answer 2xx**, once the
event is durably accepted. A verified request whose body you cannot use will
fail identically on every retry, so asking the provider to resend only burns its
retry budget — and a retry storm against a handler with a side effect is far
worse than one dropped event.

```ts
// Verified but unusable: log the topic for diagnostics, never the body.
const topic = request.headers.get("X-Provider-Topic") ?? "unknown";
console.warn("[webhook] could not map verified payload, topic:", topic);
return new Response("OK", { status: 200 });
```

Respond **fast**. Do the real work after acknowledging, or make it idempotent
and queue it.

## Post-response work must be kept alive

On serverless, a floating promise is not safe: the runtime may freeze or reclaim
the invocation the moment the response is returned, cancelling an outbound
`fetch` that was started but never awaited. Use the platform's
after-response hook rather than a bare `void` call.

```ts
after(() => sendDownstreamEvent(context, payload));   // response already sent
return new Response("OK", { status: 200 });
```

Same intent as fire-and-forget — the provider gets its 2xx immediately, because
a slow response is a retried webhook — but the invocation stays alive until the
callback settles.

## Idempotency

Webhook delivery is **at-least-once**. Assume duplicates.

- Key on the provider's own event/delivery id, **not** a value you derive from
  the payload. A derived key changes when your mapping changes, and then
  historical deduplication silently stops working.
- Make the handler safe to run twice: upsert rather than insert, check-then-set
  under a lock, or accept that the side effect is naturally idempotent and say
  so in a comment.
- A shared store is the usual home for the seen-ids set — one of the few cases
  where instances genuinely must agree. See `redis-state-patterns`.

## A valid signature proves provenance, not shape

Validate the payload at the boundary and treat every field as untrusted even
after verification. The signature says the provider sent it; it says nothing
about whether the field you need is present, the right type, or sane.

Accept more than one plausible payload shape when the provider's schema is
undocumented (`{ resource: {...} }` and a bare resource object both appear in
the wild). Assuming one shape produces the same silent failure as guessing a
digest encoding.

Coerce via explicit readers that return `null` rather than throwing, so a
partial payload degrades to "skip" instead of a 500.

## Logging and PII

Webhook payloads carry customer names, emails, phone numbers, and addresses.

- Log **identifiers and the event topic**, never bodies.
- Constant messages, never an interpolated payload or a caught value — a caught
  value from `fetch` or `JSON.parse` can echo request fragments you did not
  author and cannot redact.
- Never log the signature or the secret.

## Registration: whose secret are you verifying?

A subscription created through a platform's **API** belongs to the app that
created it and is signed with **that app's** client secret — not the webhook
secret shown in an admin panel. A handler verifying against the panel secret
therefore rejects every delivery from an API-created subscription, with a
perfectly correct 401 and no clue why.

Register through the surface whose secret you actually verify against, and write
down which one that is next to the handler. If you must support both, resolve
the secret per subscription rather than from one env var.

## Identifier brittleness

When the payload gives you an id and you must recover a related record, use the
platform's **canonical numeric identifier**, not a human-facing display name.

Display names are frequently merchant-configurable — an order display name may
carry a configurable prefix while the matching key downstream is the bare
number. Deriving one from the other by stripping characters makes **every**
lookup fail, with no error anywhere, and the derivation looks obviously correct
in review.

## Pre-merge checklist for a new webhook route

- [ ] Raw body read before any parsing; HMAC computed over those bytes.
- [ ] Shared verifier module, not a local copy.
- [ ] Fails closed on: missing secret, missing header, length mismatch, bad encoding, thrown compare.
- [ ] Constant-time comparison; both digest encodings if undocumented.
- [ ] 401 only for unverified; 2xx for verified-but-unusable.
- [ ] Responds before doing slow work; post-response work uses the platform's keep-alive hook.
- [ ] Idempotent, keyed on the provider's event id.
- [ ] Payload fields validated; unknown shapes tolerated; readers return null, not throw.
- [ ] No body, PII, secret, signature, or caught value in any log.
- [ ] Route marked non-cached / dynamic, and pinned to a runtime that has `crypto` if needed.
- [ ] Registration surface documented, matching the secret the handler verifies.
- [ ] Related-record lookups use canonical ids, not display names.
- [ ] A test posts an unsigned request and asserts 401 with zero side effects.

## Related skills

- `oauth-token-lifecycle` — the outbound half of most provider integrations.
- `redis-state-patterns` — where idempotency keys live.

---
name: oauth-token-lifecycle
description: >-
  Design server-side storage and refresh for third-party OAuth2 credentials that
  must survive restarts and scale across instances — especially providers whose
  refresh token rotates on every use. Use this skill when the task involves
  "store an OAuth token", "refresh token keeps breaking", "rotating refresh
  token", "token expired in production", "OAuth callback route", "integrate a
  carrier/payment/review provider API", "where do I keep the access token",
  "invalid_grant", or any integration where a human must click Authorize once and
  the server must stay authorized afterwards without them.
---

# OAuth token lifecycle

A long-lived server integration where a human authorizes once. The token must
then survive deploys, cold starts, and concurrent instances with nobody watching.

## The constraint that forces a store

Many providers issue a long-lived access token **alongside a refresh token that
rotates on every use** — the moment you refresh, the old refresh token dies.
Platform environment variables are immutable at runtime, so the current
credential cannot live in one. Something durable and writable must hold it.

That is the whole reason a store exists here. It is not caching.

## Shape: one key, not a keyed cache

Store the pair under a **single fixed key**. This is one service credential for
the whole application, not per-user state.

```ts
const TOKEN_KEY = "provider:oauth-tokens";
const REFRESH_LOCK_KEY = "provider:oauth-refresh-lock";
```

The keyed-cache shape is the wrong instinct and worth resisting explicitly: a
`Map`/hash of tokens invites per-user reasoning that does not apply, and the
in-process mirror of a single credential is **one slot, not a cache**. Say so in
the code, or the next reader will "fix" it into a cache.

## One module owns the store and the token endpoint

Exactly one module may talk to the credential store and to `/oauth/token`.

- The **consumer** (an API adapter) asks for a bearer token. It knows nothing
  about grants, rotation, or storage.
- The **authorization-callback route** hands over a one-time code. It knows
  nothing beyond turning a throw into an HTTP response.

Anything else leaks the grant model into callers, and then two call sites
refresh independently.

## Persist before returning

A rotation is only safe if the new pair replaces the old **before any caller
sees it**. Otherwise there is a window where the rotated-away refresh token is
still the stored one, and the next refresh uses a dead token.

```ts
// In-process first, store second: if the store write throws, this instance
// still holds the freshly rotated pair and keeps serving, instead of losing it
// from both places.
memoryCache = pair;
await redis.set(TOKEN_KEY, pair);
return pair;
```

## Concurrency: a lock is mandatory, not defensive

Rotation plus N serverless instances is a correctness bug waiting for traffic. A
spike near expiry fires N concurrent refreshes; each rotates the refresh token
and all but one is invalidated mid-flight. The stored credential is then
**permanently** broken and needs a human to re-authorize.

```ts
const lock = await redis.set(REFRESH_LOCK_KEY, 1, { nx: true, ex: LOCK_TTL_SECONDS });
if (lock !== "OK") return fallback;   // another instance owns the refresh
```

- **The loser degrades; it never waits.** Queueing behind the winner turns one
  slow refresh into a pile-up on a request path.
- **TTL reasoning:** long enough to cover one timeout-capped token request plus
  its write, short enough that a crashed holder cannot wedge the credential. A
  5s outbound cap suggests ~10s.
- **Let the lock expire; do not delete it on success.** An instance arriving
  moments later must read the freshly persisted pair, not retry with the refresh
  token that was just rotated away.

## Read budget: the in-process copy is load-bearing

A managed Redis free tier has a hard request allowance, so the store must not be
read per call. Three layers:

1. **In-process copy** — serves any call arriving while the credential is
   comfortably valid. No store call at all. This is the common path on a warm
   instance.
2. **Store read** — only to hydrate a cold instance, or when inside the refresh
   window.
3. **Token endpoint** — only on an actual rotation.

## Refresh ahead of expiry, not at it

Refresh when the token is **within a window of** expiry, not after it dies.
Refreshing at expiry makes the first request after death pay for a round trip,
and loses it entirely if the provider is down at that moment. A generous window
(hours, even a day on a 30-day token) buys many lock-guarded retries while the
old token still works.

## Negative cache: the subtle one

Inside the refresh window, a **failing** refresh would otherwise cost a full
store read plus a lock attempt on *every single request* — defeating the
in-process cache in exactly the window where it matters most.

Record when the last attempt concluded without producing a fresh token and
suppress the next round trip for a short backoff. Every non-success exit records
it, including "another instance holds the lock" — nothing failed there, but
there is equally nothing to gain from re-reading immediately.

```ts
if (lastFailureAt !== null && now - lastFailureAt < FAILURE_BACKOFF_MS) {
  const stillValid = memoryCacheIfNotHardExpired(now);
  if (stillValid) return stillValid;   // only short-circuit if there IS a fallback
}
```

Only short-circuit when there is something valid to serve. If the in-process
copy has since hard-expired, the real store read is worth its cost — another
instance may have completed the refresh meanwhile.

## Failure posture: never throw on the read path

The accessor **never throws**. Every failure degrades to `null` and the caller
falls back to a graceful alternative (a generic "calculated later" message, a
skipped enrichment). An unconfigured store means "no token", not an error —
importing the module with no environment set must not throw, because builds run
with secrets absent.

**One exception, and it matters:** a *refresh* failure must also reach
production-visible logging. Everything else degrading silently is by design,
but a dead credential looks identical to a momentary upstream hiccup. Without a
production signal, nobody learns a human must re-run the authorize flow until
the whole refresh window elapses and the access token expires.

The callback route is the inverse: it **throws**, because a human is watching
and must learn that setup failed.

## Logging discipline

No token, refresh token, client secret, authorization code, or raw upstream
payload may reach a logger. Production log calls take **constant, human-authored
messages** — never a caught value.

```ts
prodError(LOG_CONTEXT, "token refresh failed — credential may need re-authorization");
```

Why constant messages are the enforceable rule: a reviewer verifies "no variable
is interpolated here" at a glance, whereas "this particular variable happens to
be safe" has to be re-verified on every edit. `fetch` and `JSON.parse` throw
errors your module did not author, and those can echo request or response
fragments — so a caught value is never loggable, even when the code path looks
harmless.

## Parse untrusted reads, never coerce

Both the store read and the token-endpoint body are untrusted JSON. Validate
into the pair type and treat a partial value as **absent** rather than coercing
it — a half-written entry must degrade to "not authorized yet", not produce a
malformed bearer token. `expires_in` is mandatory: without it there is no honest
expiry, and inventing one either refreshes forever or leaves a dead token
stored.

Keep that parsing in its own pure module with an injected `now`. Every branch is
then testable without stubbing the store or `fetch`.

## Test seam

Module-level state that legitimately lives for the process lifetime is otherwise
untestable in isolation. Expose a clearly-named reset that **no production path
calls**:

```ts
export function __resetTokenCacheForTests(): void {
  memoryCache = null;
  lastFailureAt = null;
}
```

## Callback-route hardening

- Verify a single-use `state` against an httpOnly cookie — CSRF control for a
  URL the provider, not you, causes the browser to hit.
- Cap the length of the `code` you forward, so a bogus multi-megabyte query is
  not relayed upstream.
- `Cache-Control: private, no-store` on every response.
- Expire the state cookie on **every** outcome, success or failure.
- Never render, return, or log the token. The point of the store is that no
  human has to copy it.

## Checklist for a new provider integration

- [ ] Does the refresh token rotate? If yes, a durable store and a lock are not optional.
- [ ] One fixed key for the pair; one module owns store + token endpoint.
- [ ] New pair persisted before any caller sees it; in-process write first.
- [ ] Refresh guarded by `SET NX` + TTL; loser degrades, never waits.
- [ ] Lock left to expire on success.
- [ ] Refresh threshold set ahead of expiry, with the reasoning written down.
- [ ] Negative cache with backoff on every non-success exit.
- [ ] Accessor never throws; callback route does.
- [ ] Refresh failure surfaced to production logs with a constant message.
- [ ] No secret, token, code, or caught value in any log call.
- [ ] Untrusted reads parsed, partial values treated as absent.
- [ ] Test-only reset exported and unused in production.

## Related skills

- `redis-state-patterns` — whether this state needs Redis at all, and key/TTL design.
- `webhook-ingestion-hardening` — the other half of most provider integrations.

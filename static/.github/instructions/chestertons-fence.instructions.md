---
description: "Use when removing, deleting, simplifying, or refactoring code. Before changing code, understand why it exists. Do not delete what you cannot yet explain. Covers try/catch blocks, null guards, feature flags, sleep calls, database indexes."
---

# Chesterton's Fence

> "Do not remove a fence until you understand why it was built."
> — G.K. Chesterton, _The Thing_ (1929)

Applied to code: never delete, simplify, or refactor a piece of code — especially one that looks unnecessary — until you can state the reason it was written. If you cannot explain it, learn it first.

## The Core Question

Before touching unfamiliar code, ask: **"Why does this exist?"**

If the answer is "I don't know" or "it looks redundant", stop. Research before acting.

## Where This Rule Applies

- Removing `try/catch` blocks that seem to catch nothing
- Deleting null/undefined guards that look overly defensive
- Stripping type assertions or casts that appear redundant
- Removing feature flags, environment checks, or config overrides
- Deleting "unused" tests, mocks, or fixtures
- Simplifying retry logic, debounce, or throttle wrappers
- Removing delay/sleep calls that seem arbitrary
- Clearing commented-out code blocks with no visible explanation
- Stripping middleware or interceptors from HTTP/event pipelines
- Removing database index hints or query annotations

---

## Bad Examples

### 1. Removing a "redundant" null guard

```typescript
// BEFORE — original code
function getUserEmail(user: User | null): string {
  if (!user) return ""; // ← "dead code", user is always passed
  if (!user.profile) return ""; // ← looks defensive and unnecessary
  return user.profile.email ?? "";
}

// AFTER — "cleaned up" by removing the guards
function getUserEmail(user: User): string {
  return user.profile.email ?? "";
}
```

**Why this is wrong:** The guards exist because upstream callers occasionally pass `null` due to a race condition during logout. Removing them causes a runtime crash that only surfaces in production under concurrent requests.

---

### 2. Deleting a catch block that "does nothing"

```typescript
// BEFORE
async function syncToAnalytics(event: TrackingEvent) {
  try {
    await analyticsClient.send(event);
  } catch {
    // silently swallow — analytics must never break checkout
  }
}

// AFTER — "cleaned up"
async function syncToAnalytics(event: TrackingEvent) {
  await analyticsClient.send(event);
}
```

**Why this is wrong:** The empty catch is intentional fault isolation. A failure in the analytics service now propagates and crashes the checkout flow — a critical business regression.

---

### 3. Removing a sleep/delay call

```typescript
// BEFORE
async function activateAccount(userId: string) {
  await db.createUser(userId);
  await sleep(500); // ← "why is this here?"
  await emailService.sendWelcome(userId);
}

// AFTER — delay removed as "unnecessary"
async function activateAccount(userId: string) {
  await db.createUser(userId);
  await emailService.sendWelcome(userId);
}
```

**Why this is wrong:** The delay works around an eventual-consistency lag in the email service's user-lookup step. Without it, 10–15% of welcome emails bounce because the user record is not yet visible in the read replica.

---

### 4. Stripping a "pointless" feature flag

```typescript
// BEFORE
function renderDashboard(user: User) {
  if (!featureFlags.isEnabled("new-dashboard", user.id)) {
    return <LegacyDashboard />;
  }
  return <NewDashboard />;
}

// AFTER — legacy branch removed, flag considered dead
function renderDashboard() {
  return <NewDashboard />;
}
```

**Why this is wrong:** The flag is used for a gradual rollout and an emergency kill-switch. Removing it also deletes the rollback path for a feature that is still being monitored in production.

---

### 5. Deleting a "duplicate" database index

```sql
-- BEFORE
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_user_status ON orders(user_id, status); -- ← "covered by the one above"

-- AFTER — idx_orders_user_status dropped
```

**Why this is wrong:** `idx_orders_user_status` is a covering index used exclusively by the most expensive query in the billing pipeline. Its removal causes a full table scan and a 40× slowdown under load.

---

## Good Examples

### 1. Investigate before removing the guard

```typescript
// Step 1: search callers before touching the guard
// grep: getUserEmail(
// found: AuthProvider.tsx passes `null` during logout transition

// Step 2: document the finding inline
function getUserEmail(user: User | null): string {
  // user can be null during logout — AuthProvider passes null before clearing state
  if (!user) return "";
  // profile is undefined until lazy-loaded by ProfileProvider
  if (!user.profile) return "";
  return user.profile.email ?? "";
}

// Step 3: now safe to refactor — knowledge is captured
```

---

### 2. Preserve the intent when simplifying

```typescript
// Original intent: analytics failures must never surface to the user
async function syncToAnalytics(event: TrackingEvent) {
  try {
    await analyticsClient.send(event);
  } catch (err) {
    // Non-critical: analytics failure must not affect user-facing flows
    logger.warn("analytics sync failed", { event, err });
  }
}
```

---

### 3. Replace the workaround with a proper fix — or keep it explained

```typescript
// Option A: fix the root cause and remove the delay
async function activateAccount(userId: string) {
  await db.createUser(userId);
  await db.waitForReadReplica(userId); // explicit, tested consistency check
  await emailService.sendWelcome(userId);
}

// Option B: keep the delay with a clear explanation if the fix is out of scope
async function activateAccount(userId: string) {
  await db.createUser(userId);
  // WORKAROUND: email service reads from a replica with ~400ms lag.
  // Tracked in: https://github.com/org/repo/issues/1234
  await sleep(500);
  await emailService.sendWelcome(userId);
}
```

---

### 4. Confirm a flag is truly dead before removing it

```bash
# Check flag usage across all services before deleting
grep -r "new-dashboard" ./apps ./packages

# Verify in feature flag service that 100% of users are enrolled
# Confirm with the owning team that rollback is no longer needed
# Then remove both the flag and the legacy branch together
```

---

## Decision Process

```text
1. Read the code you intend to change or delete.
2. Ask: can I explain exactly why this exists?
   - YES → proceed; preserve the explanation in a comment or commit message.
   - NO  → research: check git log, grep call sites, search issue tracker, ask the team.
3. After understanding:
   - If the reason is still valid → keep it, improve the comment.
   - If the reason is obsolete → remove it with a commit message explaining WHY it is now safe.
   - If the reason is a workaround → fix the root cause or document the tracking issue.
```

## Commit Message Pattern

When removing code that existed for a specific reason, state that reason in the commit:

```
remove null guard in getUserEmail

Guard was added in a3f91c2 to handle logout race condition.
Auth refactor in #892 eliminated the race at the source.
All callers now guarantee a non-null User or use the Optional<User> path.
```

## Summary

| Situation                                      | Action                                    |
| ---------------------------------------------- | ----------------------------------------- |
| Code looks redundant but you don't know why    | Research first, change nothing            |
| You understand the reason and it's still valid | Keep it; add or improve the comment       |
| You understand the reason and it's obsolete    | Remove it; explain why in the commit      |
| It's a workaround for a known bug              | Fix root cause or document the issue link |
| You're reviewing someone else's removal        | Ask "did you check why this existed?"     |

# Paid traffic review — <ACCOUNT LABEL> — <YYYY-MM-DD>

**Window:** <START> to <END> (<N> days) · **Prior window:** <START> to <END>
**Snapshot:** `<path/to/snapshot.json>` · **Currency:** <CODE>
**Nothing in this report has been applied.**

---

## 0. Blockers

> Blockers change what the rest of this data is allowed to mean. If any are
> live, fixing them outranks every optimisation below. Delete this section only
> when all three checks pass, and say that they passed.

| # | Blocker | State | Consequence if live |
| --- | --- | --- | --- |
| A | Primary purchase conversion action recorded nothing in 60d | <CLEAR / LIVE> | No campaign can be judged on ROAS |
| B | Paid clicks not arriving as analytics sessions | <CLEAR / LIVE> (<X>% capture, <CLICKS> clicks → <SESSIONS> sessions) | Spend is not reaching the site, or tagging drops it |
| C | Conversions losing their source to `(direct)` | <CLEAR / LIVE> (<N> of <M> on-site orders) | Revenue cannot be credited to the campaign that produced it |

**Proxies in use:** <none / list them and say they are proxies>

**Structurally unattributable orders excluded by design:** <N> from <channel>.
These never touch the site and arrive without a client identifier. They are not
a defect, are not counted in any ROAS below, and no task should be opened to
"fix" them.

---

## 1. Week over week

| Metric | Prior | Recent | Δ |
| --- | --- | --- | --- |
| Spend | | | |
| Impressions | | | |
| Clicks | | | |
| CTR | | | |
| Avg CPC | | | |
| Conversions | | | |
| Conversion value | | | |
| Paid sessions (analytics) | | | |

Per campaign:

| Campaign | Status | Daily budget | Spend | Clicks | CTR | Conv | IS lost to budget | IS lost to rank | Negatives |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| | | | | | | | | | |

**What changed, in one paragraph:** <…>

---

## 2. Findings

> Ranked by money at stake, not by interest. State the amount.

### F1 — <title> · ~<AMOUNT>/month at stake

- **Evidence:** <numbers from the snapshot, with the window they came from>
- **Reading:** <what it means>
- **Confidence:** <high / medium / low, and why>

### F2 — <title> · ~<AMOUNT>/month at stake

- **Evidence:**
- **Reading:**
- **Confidence:**

---

## 3. Proposed changes

> One table row per change. Every row needs all three columns filled.

### C1 — <change>

| | |
| --- | --- |
| **Change** | <exact before → after> |
| **Evidence** | <the specific rows that justify it, and the thresholds they clear> |
| **Expected effect** | <what should move, by roughly how much, by when> |
| **Risk** | <what happens if the reading is wrong> |
| **Reversible?** | <yes, how / no, why> |
| **Idempotent?** | <absolute-value write, re-runnable / criterion creation, not re-runnable> |

### C2 — <change>

| | |
| --- | --- |
| **Change** | |
| **Evidence** | |
| **Expected effect** | |
| **Risk** | |
| **Reversible?** | |
| **Idempotent?** | |

**Threshold deviations:** <none / which default was changed, to what, and why>

---

## 4. Deliberately not changing

| Item | Why not |
| --- | --- |
| <item> | Below average but above the exclusion threshold — noise at this budget |
| <item> | Correct, but it would make C1's outcome unattributable. Next review. |
| <item> | Impression share lost to rank, not budget. More budget buys nothing. |

---

## 5. Recommendation

> One recommendation. Not a menu.

**<Do this.>**

Reasoning in two or three sentences: <…>

Apply order if approved:

1. <step>
2. <step>
3. Verification query to run afterwards: <query or description>

---

## 6. Assumptions on the record

| Assumption | Value used | Measured? | How to replace it |
| --- | --- | --- | --- |
| Gross margin | <X>% | **No — assumed** | Real unit costs from the merchant |
| Target conversion rate | <X>% | <No / observed over N days> | <…> |
| Gross profit per order | <AMOUNT> | derived from the above | <…> |
| Breakeven CVR at current CPC | <X>% | derived (`CPC / gross profit per order`) | <…> |

---

**Nothing above has been applied. Approve, amend, or reject before I touch the
account.**

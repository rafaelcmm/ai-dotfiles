# Verifying a card-data contract yourself

The method that produced every censused number in this skill — the TCGdex, LorcanaJSON and
Riftcodex references. Use it on a source nobody here has mapped, and re-use it on one that has:
a recorded contract is a hypothesis with a date on it.

## Contents

- [Why the documentation is not the contract](#why-the-documentation-is-not-the-contract)
- [The protocol](#the-protocol)
- [Step 1 — read the terms before the schema](#step-1--read-the-terms-before-the-schema)
- [Step 2 — probe one record](#step-2--probe-one-record)
- [Step 3 — get a corpus](#step-3--get-a-corpus)
- [Step 4 — census the fields](#step-4--census-the-fields)
- [Step 5 — read the census](#step-5--read-the-census)
- [Step 6 — write the contract down](#step-6--write-the-contract-down)
- [What a census catches that reading cannot](#what-a-census-catches-that-reading-cannot)
- [Re-verification triggers](#re-verification-triggers)
- [Anti-patterns](#anti-patterns)

## Why the documentation is not the contract

Card-data sources are maintained by volunteers who ship data faster than prose. Three
measured examples, all from this skill's own sources:

- LorcanaJSON's documentation page stated format version 2.3.0 while the served data was
  2.4.0. The docs described a schema one minor version behind the bytes.
- Riftcodex answered `403` to a header-less request when measured on 2026-08-12 and `200` on
  2026-11-02. The requirement was dropped with no announcement.
- TCGdex gained embedded Cardmarket and TCGplayer pricing with no mention in any note anyone
  held.

None of this is a criticism of the maintainers. It is the normal state of a free hobby API,
and the only workable response is to measure rather than read.

The documentation remains essential for two things the data cannot tell you: **the terms you
are allowed to use it under**, and **what a field means**. A census tells you `moveCost` is
present on 124 of 3556 cards; only the docs tell you it is the cost to move a character to a
Location.

## The protocol

1. Read the terms.
2. Probe one record.
3. Get a corpus.
4. Census the fields.
5. Read the census.
6. Write the contract down, with its date and its counts.

Budget twenty minutes. It is cheaper than one afternoon spent on a field that is absent on 30%
of records.

## Step 1 — read the terms before the schema

Terms decide architecture, so they come first.

Look for: a publisher community policy (LorcanaJSON operates under Ravensburger's and may not
be charged for), separate terms for images versus data (Scryfall), an attribution requirement,
a rate limit, and whether caching or redistribution is addressed at all.

Record what you find next to the data you cache. A licence nobody wrote down is a licence
nobody will honour six months later.

## Step 2 — probe one record

```bash
node scripts/probe-json-source.mjs <url>
```

What matters in the output, and why:

- **Whether a `User-Agent` is required.** The probe asks twice, with and without. Never assume
  this in either direction; it has flipped on a source in this skill.
- **Redirect behaviour.** A 3xx on a data endpoint means something other than the expected
  service may be answering. Refuse to follow redirects in production code for the same reason.
- **`ETag` and `Last-Modified`.** On a multi-megabyte static file these turn every subsequent
  sync into a conditional request that usually returns 304.
- **Content length.** This is the number that decides your architecture. A 9.5 MB file cannot
  live in a request-path cache; a 2 KB record can.
- **The envelope.** Paged (`{items, total, page, size, pages}`), bare array, or id-keyed object
  — each needs different ingestion code.

## Step 3 — get a corpus

One record proves the shape exists. It proves nothing about which fields are optional.

- **Static file source**: download the whole thing.
- **Paged API**: sweep every page. Fetch page one, read the page count, then pull the rest in
  small concurrent chunks — four at a time is a safe courtesy ceiling for an unmetered free
  service. **Cap the loop independently of the reported page count**, which is
  upstream-controlled.
- **Per-record API with no list endpoint**: take a sample that deliberately includes the
  awkward cases — promos, tokens, alternate art, the newest set, the oldest set. A census over
  a hundred ordinary cards will tell you every field is always present and then break on the
  first promo.

Save the corpus to a file — `node scripts/probe-json-source.mjs <url> --save corpus.json` does
it in the same pass as the probe. The census runs offline over it, and keeping it lets you diff
the next census against this one.

## Step 4 — census the fields

```bash
node scripts/field-census.mjs corpus.json --at cards
```

`--at` names the key holding the records, or a dotted path to it (`data.cards`); omit it when
the root is already an array or an id-keyed object.

## Step 5 — read the census

| Signal | What it means | What to do |
|---|---|---|
| present on 100% of records | required, for this corpus and this date | type it required |
| present on 1–99% | **optional**, whatever the docs imply | type it optional; find out what predicts its presence |
| a small distinct value set | a facet candidate | derive the facet at runtime; never hardcode the list |
| a large distinct value set on a string | free text | render it, do not facet it |
| nullable and sparsely populated | a display state you must design | guard every read; design the empty case |
| several URL hosts on one field | a mixed-origin field | write the allowlist from the measured hosts |
| the same concept in two fields | near-duplicate pair | find out which one is printed and which is derived |

**The presence percentage is the single most valuable output.** It is the difference between a
type that holds over a whole catalogue and one that crashes on card 2000. On Lorcana,
`strength` looks required until the census shows 2710 of 3556 — it exists on Characters only.

**Group the optional fields by what predicts them.** `lore` and `willpower` share a count of
2834 because both belong to Characters and Locations; `moveCost`'s 124 exactly matches the
Location count. Those correlations are the real schema, and they are invisible in a field list.

**Record the counts, not just the field names.** A count is how the next person detects a
truncated download: a sync that fetches 400 cards when the recorded census said 3556 is
broken, and without the recorded number it looks like a successful run.

## Step 6 — write the contract down

Add a reference file to this skill using the same shape as the three censused per-source files
— TCGdex, LorcanaJSON and Riftcodex:

1. **A dated header.** "Last verified YYYY-MM-DD by <how>." The how matters — "by fetching all
   1451 cards and counting" carries different weight from "by reading the docs".
2. **Transport**: base URL, auth, envelope, headers, timeout and retry posture, 404 semantics.
3. **Endpoints**, with what each is actually good for.
4. **The measured shape**, as a commented JSON sample from a real response.
5. **The presence census**, as a table with counts.
6. **Value distributions**, explicitly labelled as documentation rather than contract.
7. **Images**: hosts, URL construction rules, dimension variants, orientation.
8. **The gotchas**, each one naming the failure it causes. A gotcha without its symptom is
   forgettable; "rendering it unrotated looks like a bug in your CSS" is not.
9. **A re-verify block** of copy-pasteable commands.

Then add the source to the freshness ledger in the parent skill, and mention the new reference
file in that skill's source table so it is reachable.

## What a census catches that reading cannot

Four real finds, each of which would have shipped as a bug:

- **A polymorphic field.** Riftcodex's `cardmarket_id` is a string on some sets, an array of
  strings on others, and null on the rest. Any type that assumes one shape crashes on the
  others. No documentation mentioned it.
- **A meaningful empty string.** 104 Lorcana cards carry `color: ""`. Treating empty as missing
  loses the inkless cards; treating it as a value gets them right.
- **Two counts that are not the same count.** TCGdex sets carry both `cardCount.official` (the
  printed denominator) and `cardCount.total` (including secret rares). Picking the wrong one
  renders a wrong collector code on every card in the set.
- **Future records with no data.** LorcanaJSON ships announced-but-unreleased sets with null
  release dates and zero card counts. A UI listing "every set" renders empty shells.

## Re-verification triggers

Run the census again when any of these happens — not on a calendar:

- **A new set released** for that game. New sets introduce new rarities, new keywords, new
  subtypes, and occasionally new fields.
- **A version field moved.** Gate on it in code so this is automatic rather than remembered.
- **Any upstream error you did not expect**, including a 403, a redirect, or a schema
  validation failure. An unexplained error is a contract change until proven otherwise.
- **Before building anything new** on a source you have not touched in a month.
- **A field you depend on starts arriving null.** Census before patching — a single null often
  means the field became conditional for a whole class of records.

When the census disagrees with the recorded contract, the census wins, and the reference file
gets fixed **in the same change** as the code. A note updated later is a note never updated.

## Anti-patterns

| Pattern | Why it fails |
|---|---|
| typing a field required because the docs list it | docs list every field a record *may* have; the census tells you which it *does* |
| hardcoding a rarity, type or domain list | the next set adds one, and your facet silently drops those cards |
| trusting an upstream `pages` or `total` to bound a loop | it is upstream-controlled; one malformed response becomes thousands of requests |
| retrying a 404 | it is a stable answer about that record; retrying spends someone else's bandwidth to hear it again |
| caching a failure | a card added upstream tomorrow stays invisible until the entry expires |
| censusing only the first page | page one is the oldest, most regular set; the awkward records are at the end |
| recording field names without counts | the count is what detects a truncated download later |
| patching around a drifted contract without updating the note | the next reader trusts the note, and this is exactly the failure the skill exists to prevent |

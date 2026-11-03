---
name: tcg-card-data-sources
description: >-
  Find, verify and consume trading-card-game card data from public sources — Pokémon
  through TCGdex, Disney Lorcana through LorcanaJSON, Riftbound through Riftcodex, plus
  checked leads for Magic, Yu-Gi-Oh, One Piece, Digimon and TCGplayer price mirrors. Use
  when a task needs card names, sets, rarities, rules text, card art or prices for any TCG:
  "tcgdex", "lorcanajson", "riftcodex", "scryfall", "ygoprodeck", "tcgcsv", "card images
  API", "TCG card database", "import a card catalogue", "fetch Pokémon card data". Load it
  BEFORE writing fetch code: every field contract here is a dated measurement, not a guarantee.
  Covers card data, art and prices — not gameplay rules.
---

# TCG card data sources

Card-data sources for trading card games are hobby projects, fan APIs and publisher CDNs.
They ship new sets, rename fields, add endpoints, change their auth posture and go down.
None of them owes you a stable contract, and most publish no changelog you can subscribe to.

So this skill is deliberately **not** an authority on what any source returns today. It is a
record of what was measured, when it was measured, and the exact command that re-measures it.

## The one rule

> Every field list, count, header requirement and URL shape in this skill is a **dated
> measurement**. Before writing code against it, re-run the measurement. If the measurement
> disagrees with the note, the measurement wins and the note gets fixed in the same change.

This is not caution for its own sake. Between 2026-08-12 and 2026-11-02, with nobody
touching anything:

| What a note said | What re-measuring found |
|---|---|
| Riftcodex answers `403 Forbidden` without a `User-Agent` | It answers `200`. The requirement was dropped. |
| LorcanaJSON is `formatVersion` 2.3.5, `allCards.json` is 8.79 MB | 2.4.0 and 9.55 MB. |
| TCGdex carries no pricing | Cards now embed Cardmarket and TCGplayer pricing per variant. |
| LorcanaJSON's own docs page states the current format version | The page said 2.3.0 while the data served 2.4.0. **Docs lag data.** |

Three of four moved in under three months. Code written from the stale note would have been
over-defensive in one place, wrong about file size in another, and would have missed a
feature that arrived for free.

## Freshness ledger

| Source | Game | Last verified | Re-verify with |
|---|---|---|---|
| `api.tcgdex.net` / `assets.tcgdex.net` | Pokémon | 2026-11-02 | `node scripts/probe-json-source.mjs https://api.tcgdex.net/v2/en/cards/swsh3-136` |
| `lorcanajson.org` | Disney Lorcana | 2026-11-02 | `node scripts/probe-json-source.mjs https://lorcanajson.org/files/current/en/metadata.json` |
| `api.riftcodex.com` | Riftbound (LoL) | 2026-11-03 | `node scripts/probe-json-source.mjs https://api.riftcodex.com/sets?size=3` |
| `api.scryfall.com` | Magic | 2026-11-02 | `node scripts/probe-json-source.mjs https://api.scryfall.com/bulk-data` |
| `db.ygoprodeck.com` | Yu-Gi-Oh | 2026-11-02 | `node scripts/probe-json-source.mjs "https://db.ygoprodeck.com/api/v7/cardinfo.php?name=Dark%20Magician"` |
| `tcgcsv.com` | prices, 94 categories | 2026-11-02 | `node scripts/probe-json-source.mjs https://tcgcsv.com/tcgplayer/categories` |

When you re-verify, **update the date and the finding in the matching reference file**. A
ledger nobody updates is worse than no ledger, because it looks maintained.

## Pick the source

| Game | Primary source | Reference |
|---|---|---|
| Pokémon | TCGdex — REST + GraphQL + SDKs, 10+ languages, art CDN, embedded Cardmarket and TCGplayer pricing | [references/pokemon-tcgdex.md](references/pokemon-tcgdex.md) |
| Disney Lorcana | LorcanaJSON — static JSON files, no query API | [references/lorcana-lorcanajson.md](references/lorcana-lorcanajson.md) |
| Riftbound (League of Legends) | Riftcodex — paged REST, self-described work in progress | [references/riftbound-riftcodex.md](references/riftbound-riftcodex.md) |
| Magic, Yu-Gi-Oh, One Piece, Digimon, and prices for all of them | checked leads, not verified contracts | [references/other-games-and-pricing.md](references/other-games-and-pricing.md) |
| Anything not listed | measure it yourself | [references/contract-verification.md](references/contract-verification.md) |

For the architecture that consumes any of these — projection, snapshotting, idempotent sync,
degradation — load the `tcg-catalog-ingestion` skill. This skill ends at "you know what the
source returns".

## First contact with any source

Run this before writing a line of integration code. It takes minutes and it is the only way
the field list you code against is real.

1. **Find the canonical docs and read the terms.** Fan APIs usually carry a publisher
   community-policy restriction — LorcanaJSON may not be charged for, Scryfall restricts
   image redistribution. Terms decide the architecture, so read them before designing it.
2. **Probe one record.** `node scripts/probe-json-source.mjs <url>` reports status, content
   type, size, redirect behaviour, whether a `User-Agent` is required, and the top-level
   shape. Never assume a header requirement in either direction.
3. **Pull the whole corpus if it is small enough**, or one full page if it is not.
4. **Census the fields.** `node scripts/field-census.mjs <file.json> --at cards` prints
   per-field presence counts and value vocabularies. This is what turns "the docs list 40
   fields" into "7 of them are present on every record and 11 appear under 5% of the time" —
   which is the difference between a type that holds and a type that crashes on card 2000.
5. **Write the contract down with its date and its counts**, as a reference file in this
   skill. Counts matter: they are how the next person detects a truncated download.
6. **Type the optional fields as optional.** Anything the census found below 100% presence is
   optional, whatever the docs imply.

## Gotchas that cost real time

Each one was paid for once. Details in the per-source reference.

| Source | Trap |
|---|---|
| TCGdex | Card art appends `/<quality>.<ext>` to the image base; set logo and symbol append `.<ext>` **with no quality segment**. Getting it backwards is a 404. |
| TCGdex | The SDK's `setLang` mutates the client instance — a shared singleton races across concurrent requests for different locales. One instance per language. |
| LorcanaJSON | Every image file is portrait, including Location cards, whose art is rotated 90° counter-clockwise inside it. Rendering it unrotated looks like a bug in your CSS. |
| LorcanaJSON | `allCards.json` is ~9.5 MB. It exceeds common per-entry cache limits, so it belongs in a committed snapshot, not a request-path fetch. |
| Riftcodex | `/cards/riftbound/{id}` does **partial** matching and can return several cards. Accept only an exact id match, never the nearest. |
| Riftcodex | Signature card ids contain a literal `*` (`unl-229*-219`) which `encodeURIComponent` leaves alone. Force it to `%2A`. |
| Riftcodex | `text.rich` is HTML. It must never reach `dangerouslySetInnerHTML` — tokenize against an allowlist and fall back to `text.plain`. |
| All | A list endpoint's reported `pages` count is upstream-controlled. Cap your loop independently or one malformed response becomes thousands of requests. |
| All | Measured value vocabularies (rarities, types, domains) are **documentation, not contracts**. Derive facets from the data you fetched; never hardcode the list. |

## Courtesy, limits and attribution

These are free services run by volunteers. Behave accordingly, and treat every number below
as a thing to re-read rather than a thing to recall.

- **Send a `User-Agent` that identifies your project and gives a contact URL**, whether or not
  the source currently enforces it. It is how an operator tells your traffic from an attack.
- **Cap concurrency low.** TCGdex throttled at roughly four to eight concurrent fetches from a
  single IP when measured. Four is a safe default for any unmetered fan API.
- **Prefer bulk endpoints over crawling** where they exist — Scryfall publishes daily bulk
  files precisely so you do not walk its card endpoint, and asks for a delay between requests
  when you must.
- **Retry transport failures, never HTTP statuses.** A 404 is a stable answer; re-asking wastes
  someone's bandwidth to get the same reply.
- **Keep attribution and licensing in the repo**, next to the data you cached. Several sources
  operate under a publisher community policy that forbids charging for access.

## When the contract has changed

1. Do not patch around it silently. A reader who trusts a stale note is the failure mode this
   skill exists to prevent.
2. Re-run the census and diff it against the recorded counts.
3. Fix the reference file: new date, new counts, and a line naming what moved.
4. If a source gained a capability (TCGdex pricing is the live example), say so explicitly —
   a note that only ever records losses trains people to skip it.
5. If a source is gone or degraded past use, record that too, and name the alternative.

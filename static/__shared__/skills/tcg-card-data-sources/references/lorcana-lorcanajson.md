# LorcanaJSON — Disney Lorcana

**Last verified 2026-11-02** against `formatVersion` 2.4.0, `generatedOn`
`2026-10-16T16:43:42`. Every count below was measured by downloading the real file and
counting, not read off the documentation. Re-measure before relying on any of it.

Docs home: <https://lorcanajson.org> — the field explanation is the "Card data fields
explanation" section of that page.

## Contents

- [Licensing — read this first](#licensing--read-this-first)
- [Files and URL patterns](#files-and-url-patterns)
- [There is no query API](#there-is-no-query-api)
- [Top-level shape](#top-level-shape)
- [Set objects, including unreleased ones](#set-objects-including-unreleased-ones)
- [Field presence census](#field-presence-census)
- [Value distributions](#value-distributions)
- [Images, and the Location rotation](#images-and-the-location-rotation)
- [Rules text and its symbols](#rules-text-and-its-symbols)
- [foilTypes](#foiltypes)
- [Fields with no analogue in other TCGs](#fields-with-no-analogue-in-other-tcgs)
- [The formatVersion gate](#the-formatversion-gate)
- [Re-verify](#re-verify)

## Licensing — read this first

LorcanaJSON operates under Ravensburger's Community Code Policy and states: "We are expressly
prohibited from charging you to use or access this content."

That is an architectural constraint, not a footnote. If your product gates Lorcana card data
behind a paywall you are outside the policy the data is published under. Record the
attribution next to wherever you cache the data.

## Files and URL patterns

All under `https://lorcanajson.org/files/current/<lang>/`:

| File | Path | Notes |
|---|---|---|
| metadata | `metadata.json` | ~90 bytes: `{formatVersion, generatedOn, language}` |
| all cards | `allCards.json` | **9,553,983 bytes** when measured, served with `last-modified` and `etag` |
| all cards, zipped | `allCards.json.zip` | far smaller, but inflating it needs a dependency |
| one set | `sets/setdata.<setCode>.json` | e.g. `setdata.1.json`, `setdata.Q1.json` |
| all sets, zipped | `sets/allSets.zip` | |
| one deck | `decks/deckdata.<deckCode>.json` and `.full.json` | |
| all decks | `decks/allDecks.zip`, `allDecks.full.zip` | |
| Cockatrice | `lorcana.cockatrice.xml` and `.zip` | |
| Coconut format | `formatCoconutCards.json` | English only |

**Per-set files are the better default for incremental work** and are easy to miss: a new set
release means fetching one `setdata.<setCode>.json` instead of re-downloading 9.5 MB. Use
`allCards.json` for a first full ingest or a full re-census, and the per-set files thereafter.

Languages published: `en`, `fr`, `de`, `it`. Four, not more — if you need Portuguese or
Spanish Lorcana data, this source does not have it and no amount of URL guessing will produce
it.

`allCards.json` serves `ETag` and `Last-Modified`. Send `If-None-Match` and let a 304 tell you
nothing changed, rather than re-downloading 9.5 MB to find out.

## There is no query API

This is static file hosting. There is no endpoint that answers "give me card 468", no search,
no pagination, no filtering. You download a file and index it yourself.

The practical consequence: 9.5 MB exceeds the per-entry limit of most request-path caches, so
Lorcana data belongs in a **committed, projected snapshot** built offline. Fetching
`allCards.json` inside a request handler is not a slow design, it is a broken one.

## Top-level shape

```jsonc
{
  "metadata": { "formatVersion": "2.4.0", "generatedOn": "2026-10-16T16:43:42", "language": "en" },
  "sets":  { "<setCode>": { /* set object */ } },
  "cards": [ /* card objects */ ]
}
```

Measured: **3556 cards, 19 sets**. Set codes are `"1"`–`"16"` plus `"Q1"`, `"Q2"`, `"Q3"`.

`setCode` is a **string and sometimes non-numeric**. Parsing it as an integer loses the quest
sets entirely, and sorting it as a string puts `"10"` before `"2"`. Sort numerics ascending
first, then the `Q*` sets.

## Set objects, including unreleased ones

```jsonc
{
  "prereleaseDate": "2026-10-02", "releaseDate": "2026-10-02",
  "hasAllCards": false,
  "type": "quest",                 // or "expansion"
  "number": 3,
  "allowedInFormats": { "Core": { "allowed": false }, "Infinity": { "allowed": false } },
  "allowedInTournamentsFromDate": null,
  "name": "Illumineer's Quest: The Great Hunny Rescue",
  "cardCounts": { "base": 37, "nonBase": 0, "promo": 1, "total": 38 }
}
```

Two traps, both measured:

- **`cardCounts` keys are not guaranteed.** Sets `"15"` and `"16"` have no `total`. Read every
  member defensively.
- **The file contains announced-but-unreleased sets.** Set `"16"` (Cosmic Quest) has
  `releaseDate: null`, `hasAllCards: false` and `cardCounts` all zero. A UI that lists "every
  set" will render empty shells unless you filter on `hasAllCards` or a non-null `releaseDate`.
  Decide which, deliberately.

`allowedInFormats` is always present on both sets and cards, shaped
`{ Core: {allowed, allowedUntilDate?, rotationGroup?}, Infinity: {allowed, ...} }`.

## Field presence census

Out of **3556 cards**. Always present (all 3556):

`allowedInFormats`, `allowedInTournamentsFromDate`, `artists`, `artistsText`, `code`, `color`,
`cost`, `externalLinks`, `foilTypes`, `fullIdentifier`, `fullName`, `fullText`,
`fullTextSections`, `id`, `images`, `inkwell`, `name`, `number`, `rarity`, `setCode`,
`simpleName`, `story`, `type`.

Everything else is **optional and must be typed optional**:

| Field | Count | Present when |
|---|---|---|
| `abilities` | 2926 | card has abilities |
| `subtypes` / `subtypesText` | 2920 | card has subtypes |
| `lore`, `willpower` | 2834 | Characters and Locations |
| `version` | 2833 | card has a subtitle |
| `strength` | 2710 | Characters only |
| `flavorText` | 2050 | — |
| `keywordAbilities` | 1044 | — |
| `baseId` | 588 | variant of another printing |
| `effects` | 470 | 469 Action cards plus one Character — not Action-exclusive |
| `varnishType` | 355 | — |
| `promoSourceCategory` | 320 | promos |
| `promoIds` | 249 | — |
| `enchantedId` | 240 | base card with an Enchanted printing |
| `promoGrouping` | 216 | — |
| `colors` | 188 | dual-ink cards only |
| `reprintedAsIds` / `reprintOfId` | 180 each | reprint relationships |
| `clarifications` | 125 | — |
| `moveCost` | 124 | **Locations only** |
| `artistsNormalized` | 117 | — |
| `epicId` | 108 | — |
| `foilEffectColors` | 103 | — |
| `historicData` | 74 | — |
| `promoSource` | 67 | — |
| `names` | 37 | — |
| `iconicId` | 12 | — |
| `maxCopiesInDeck` | 9 | — |
| `errata` | 8 | — |
| `variant` / `variantIds` | 7 each | art variants |

A projection that keeps roughly the 30 rendered fields and drops `abilities` (redundant with
`fullTextSections`), `externalLinks` (competitor storefront ids), `historicData`, `errata`,
`clarifications`, `artists`, `artistsNormalized`, `varnish*`, `foilEffectColors` and the
`names`/`promoIds`/`variantIds` id-bags cuts the file to roughly a third of its size. Omit
absent keys entirely rather than writing nulls — that is most of the saving.

## Value distributions

Measured, and **documentation rather than contract** — derive facets from the data you
actually loaded.

- `type`: `Character` 2710, `Action` 469, `Item` 253, `Location` 124.
- `rarity`: `Common` 1012, `Uncommon` 756, `Rare` 677, `Special` 319, `Super Rare` 258,
  `Enchanted` 240, `Legendary` 174, `Epic` 108, `Iconic` 12.
- `color`: 22 distinct values — 6 single inks (`Amber` 557, `Amethyst` 543, `Ruby` 543,
  `Emerald` 542, `Sapphire` 541, `Steel` 538), 15 dual-ink combinations covering 188 cards,
  and **104 cards with `color: ""`**. The empty string is a real value; treat it as inkless,
  not as missing data.
- `colors` exists only on the 188 dual-ink cards. For a single-ink card the ink is in `color`
  and `colors` is absent, so read both.
- `story`: 64 distinct values (the Disney franchise, e.g. `"Peter Pan"`).

## Images, and the Location rotation

`images` keys measured: `full` 3556, `thumbnail` 3556, `foilMask` 3434, `varnishMask` 355,
`fullFoil` 30, `varnishMask2` 2. Only `full` and `thumbnail` are ordinarily rendered.

Host: `api.lorcana.ravensburger.com` on all 3556. Path shape
`https://api.lorcana.ravensburger.com/images/<lang>/set<N>/<number>_<sha1>.jpg`. The filename
hash is not derivable — you must read the URL from the data.

`full` is 1468×2048 JPG; `thumbnail` is 367×512 JPG.

**Every image file is portrait 1468×2048, including Location cards, which are landscape.** The
Location art sits inside the portrait file rotated 90° counter-clockwise — the printed title
reads bottom-to-top. Render Locations in a landscape frame with a +90° (clockwise) correction.
Diagnosing this from scratch costs an afternoon, because nothing about the file says it is
rotated and the image loads perfectly.

## Rules text and its symbols

Prefer `fullTextSections` (a `string[]`, one entry per ability or effect block) over
`fullText`: it gives you the paragraph split for free. Newlines inside a section are printed
line breaks.

Unicode characters measured in `fullText`, which must be mapped to styled glyphs rather than
left bare — they render inconsistently across fonts at heading sizes:

| Char | Code | Count | Meaning |
|---|---|---|---|
| `⬡` | `U+2B21` | 937 | ink cost |
| `⟳` | `U+27F3` | 456 | exert |
| `◊` | `U+25CA` | 216 | lore |
| `•` | `U+2022` | 92 | subtype separator |
| `⛉` | `U+26C9` | 44 | strength |
| `◉` | `U+25C9` | 5 | willpower |

`keywordAbilities` is a flat `string[]` of keyword names with no reminder text. Measured top
values: `Shift` 341, `Evasive` 198, `Bodyguard` 84, `Ward` 79, `Support` 77, `Rush` 69,
`Singer` 56, `Resist` 56, `Challenger` 52, `Boost` 50, `Sing Together` 37, `Reckless` 28. Read
the list off each card; do not hardcode this vocabulary.

## foilTypes

Always present, a `string[]`. **18 distinct combinations** measured; the 10 most common cover
3486 of 3556 cards: `["None","Silver"]` 2833 (the ordinary case), `["Satin"]` 145, `["None"]`
122, `["Magma"]` 95, `["Silver"]` 90, `["VerticalWave"]` 76, `["Lava"]` 68, `["FreeForm2"]` 23,
`["Tinsel"]` 19, `["None","FreeForm1"]` 15. The remaining 8 combinations cover 70 cards between
them, so do not treat this list as the whole vocabulary.

The useful derived predicate: a card exists **only** as a premium finish when `foilTypes` does
not contain `"None"`.

## Fields with no analogue in other TCGs

- `story` — the Disney franchise the card belongs to. Nothing else in this skill has an
  equivalent axis, and it is a genuinely good facet.
- `fullIdentifier` — `"32/204 • EN • 3"`, exactly as printed. Use it verbatim; recomputing it
  from `number` and a set count gets the quest sets and promos wrong.
- `inkwell` — boolean, whether the card can be put into the inkwell.
- `subtypesText` — the printed `•`-joined form, ready to render.

## The formatVersion gate

Check `metadata.json` before downloading the payload, and **abort on a major-version change**.
Every `2.x` bump observed has been additive — 2.3.5 to 2.4.0 added fields and broke nothing,
which is a track record rather than a promise. A `3.x` would mean the field contract changed
shape and a human must re-read the documentation and redo the census.

Note the asymmetry measured on 2026-11-02: the documentation page stated format version 2.3.0
while the served data was 2.4.0. The data is the authority. Gate on `metadata.json`, never on
what the page says.

## Re-verify

```bash
# the gate
curl -sS -A "yourproject/1.0" https://lorcanajson.org/files/current/en/metadata.json

# size without downloading, plus the cache validators
curl -sS -I -A "yourproject/1.0" https://lorcanajson.org/files/current/en/allCards.json \
  | grep -iE 'content-length|last-modified|etag'

# full re-census, run from the skill root
curl -sS -A "yourproject/1.0" -o /tmp/allCards.json https://lorcanajson.org/files/current/en/allCards.json
node scripts/field-census.mjs /tmp/allCards.json --at cards
```

Then update the date at the top of this file and every count the census contradicts.

# Riftcodex — Riftbound (League of Legends TCG)

**Last verified 2026-11-02** by fetching all 1451 cards and all 8 sets and counting. Every
number below is that measurement, not a contract the operators owe you.

Docs: <https://riftcodex.com/docs/> — endpoint pages under `/docs/endpoints/cards/` and
`/docs/endpoints/sets/`. The documentation describes the API as "an active work in progress",
which is a warning worth taking literally: every consumer must degrade gracefully rather than
assume a field is there.

## Contents

- [Transport](#transport)
- [User-Agent: send one, never depend on it](#user-agent-send-one-never-depend-on-it)
- [Endpoints](#endpoints)
- [The partial-match trap](#the-partial-match-trap)
- [Id formats and the asterisk](#id-formats-and-the-asterisk)
- [Card shape](#card-shape)
- [Attribute presence by type](#attribute-presence-by-type)
- [Value distributions](#value-distributions)
- [Sets, and a polymorphic field](#sets-and-a-polymorphic-field)
- [Images](#images)
- [text.rich — closed vocabulary, still not safe](#textrich--closed-vocabulary-still-not-safe)
- [Paging safely](#paging-safely)
- [Re-verify](#re-verify)

## Transport

- Base URL: `https://api.riftcodex.com`
- No authentication, no API key, no documented rate limit. Be conservative anyway.
- List endpoints return `{ items: T[], total, page, size, pages }`. `size` maxes out at 100.
- `pages` is computed from `total / size`, so `?size=1` reports 1451 pages. It is a derived
  number, not a promise about anything.
- Measured response time is uneven, with slow bursts. A generous per-request timeout (45 s)
  and three attempts with growing backoff cleared every burst observed; a single retry
  back-to-back did not, because a slow burst outlasts both attempts.

## User-Agent: send one, never depend on it

On 2026-08-12 this API answered **`403 Forbidden`** to any request without a `User-Agent`
header. Verified then: `curl` worked, a bare `urllib` call did not.

On 2026-11-02 the same header-less request returned **`200`**. The requirement was dropped,
silently, with no announcement.

Keep sending a descriptive `User-Agent` anyway — `yourproject/1.0 (+https://example.com)`. It
costs nothing, it is how an operator distinguishes your traffic from abuse, and a requirement
that has flipped once can flip back. But do **not** write code that depends on the 403: a
consumer that treats a missing-UA 403 as a sentinel is now dead code, and a consumer that
assumes no header is needed will break if it returns.

This single field is the clearest argument in this whole skill for re-measuring rather than
trusting a note.

## Endpoints

| Endpoint | Use |
|---|---|
| `GET /cards?size=100&page=N` | catalogue sweep — 1451 cards over 15 pages |
| `GET /cards/{objectId}` | exact lookup by the 24-hex Riftcodex id |
| `GET /cards/riftbound/{printedId}` | lookup by printed id — **partial match**, see below |
| `GET /cards/search?query=` | free-text search |
| `GET /cards/name?exact=` / `?fuzzy=` | name lookup |
| `GET /cards/tcgplayer/{id}` | cross-reference by TCGplayer product id |
| `GET /sets?size=100` | 8 sets, one page |
| `GET /sets/{id}`, `/sets/set-id/{set_id}`, `/sets/tcgplayer/{id}`, `/sets/cardmarket/{id}` | set lookups |

List parameters: `sort`, `dir` (`1` ascending, `-1` descending), `set_id`, `new`, `page`,
`size`.

Server-side sort keys measured as accepted: `name`, `collector_number`, `public_code`, `type`,
`supertype`, `rarity`, `domain`, `artist`, `set_id`, `set_label`, `energy`, `might`, `power`.

**Sweeping the whole catalogue beats per-card lookups** past a handful of cards: 15 requests
gets you everything plus the facet vocabularies, which no single-card endpoint can give you.

## The partial-match trap

`GET /cards/riftbound/{id}` performs **partial** matching and can return several cards. It
answers with a bare JSON array, not the `{items,...}` envelope.

Accept only the item whose `riftbound_id` equals the requested value case-insensitively. If
none does, it is a miss. Never take the first element, and never take the nearest — a request
for `unl-215-219` can legitimately come back with several cards whose ids share that prefix,
and silently picking one renders the wrong card on a page that looks entirely correct.

## Id formats and the asterisk

| Form | Example | Lookup path |
|---|---|---|
| printed id | `unl-215-219` (`set-number-total`) | `/cards/riftbound/{id}` |
| alternate art | `unl-116a-219` | same |
| **signature** | `unl-229*-219` | same — the `*` needs `%2A` |
| Riftcodex object id | `69c4407c9288b1e85d94de8a` (24 hex) | `/cards/{id}` |

`encodeURIComponent` **leaves `*` unescaped**. A signature card's id therefore produces a URL
with a literal asterisk in the path, which does not resolve. Force it:

```js
const encodePathSegment = (id) => encodeURIComponent(id).replace(/\*/g, "%2A");
```

A validation pattern covering both shapes, applied after `trim().toLowerCase()` with a length
cap of 32:

```js
/^(?:[0-9a-f]{24}|[a-z]{2,4}-\d{1,4}[a-z*]?-\d{1,4})$/
```

## Card shape

All 14 top-level fields were present on all 1451 cards:

```jsonc
{
  "id": "69c4407c9288b1e85d94de8a",
  "name": "Vi - Piltover Enforcer (Signature)",
  "riftbound_id": "unl-229*-219",
  "tcgplayer_id": "685522",
  "collector_number": 229,                     // integer
  "attributes": { "energy": null, "might": null, "power": null },
  "classification": { "type": "Legend", "supertype": null, "rarity": "Rare",
                      "domain": ["Fury", "Order"] },
  "text": { "rich": "<p>When you conquer…</p>", "plain": "When you conquer…", "flavour": null },
  "set": { "set_id": "UNL", "label": "Unleashed" },
  "media": { "image_url": "https://cmsassets.rgpub.io/sanity/images/dsfx7636/game_data_live/<sha1>-744x1039.png?accountingTag=RB",
             "artist": "Jonathan Santoro",
             "accessibility_text": "Riftbound Legend: Piltover Enforcer. When you conquer…" },
  "tags": ["Vi"],
  "orientation": "portrait",                   // "portrait" | "landscape"
  "metadata": { "clean_name": "Vi Piltover Enforcer Signature",
                "updated_on": "2026-07-10T22:45:08.861364+00:00",
                "alternate_art": false, "overnumbered": false, "signature": true },
  "new": false
}
```

Measured nullability: `classification.supertype` is null on 952 cards, `text.flavour` on 682,
every `attributes` member is `number | null`, and `text.rich` is empty on 20 cards. `media.artist`
was populated on all 1451. `tags` is `[]` on 399 cards and holds up to 4 entries.

Despite 100% top-level presence, narrow defensively. The API is a declared work in progress,
and the right posture is: **drop a card that is missing something unrenderable** (`id`,
`riftbound_id`, `name`, a well-formed `classification`, `media.image_url`, a valid
`orientation`) and **default everything else** (`tags: []`, `new: false`, metadata flags
false, `collector_number: 0`). A future partial record should cost a badge, not a listing.

`accessibility_text` is a genuine gift — publisher-written alt text, per card. Use it.

## Attribute presence by type

This drives which stats a card detail view should render. Show the block only for non-null
values; three of six types never carry any.

| type | of | energy | might | power |
|---|---|---|---|---|
| `Unit` | 771 | 767 | 771 | 388 |
| `Spell` | 267 | 267 | 0 | 153 |
| `Gear` | 132 | 131 | 0 | 41 |
| `Legend` | 180 | 0 | 0 | 0 |
| `Rune` | 30 | 0 | 0 | 0 |
| `Battlefield` | 71 | 0 | 0 | 0 |

## Value distributions

Measured over 1451 cards. **Build facets by scanning the data you fetched** — these are for
sanity-checking a census, not for hardcoding.

- `classification.type`: `Unit` 771, `Spell` 267, `Legend` 180, `Gear` 132, `Battlefield` 71,
  `Rune` 30.
- `classification.supertype`: `null` 952, `Champion` 403, `Signature` 61, `Basic` 30,
  `Token` 5.
- `classification.rarity`: `Rare` 380, `Common` 302, `Uncommon` 276, `Epic` 225, `Promo` 148,
  `Showcase` 120.
- `classification.domain` (multi-valued): `Order` 274, `Mind` 273, `Body` 269, `Calm` 268,
  `Fury` 266, `Chaos` 266, `Colorless` 76.
- `orientation`: `portrait` 1380, `landscape` 71.
- `tags`: region, tribe and champion names. Top values `Ionia` 138, `Noxus` 73, `Yordle` 62,
  `Demacia` 54, `Bilgewater` 54, `Equipment` 52, `Shurima` 50, `Bandle City` 45,
  `Freljord` 43, `Mount Targon` 43, `Piltover` 43, `Zaun` 39.

## Sets, and a polymorphic field

`GET /sets?size=100` returned 8 sets:

| `set_id` | name | `card_count` | `published_on` |
|---|---|---|---|
| `PR` | Riftbound Promotional Cards | 13 | 2025-10-31 |
| `OGS` | Origins: Proving Grounds | 24 | 2025-10-31 |
| `OGN` | Origins | 352 | 2025-10-31 |
| `OPP` | Riftbound Organized Play Promotional Cards | 133 | 2025-10-31 |
| `JDG` | Riftbound Judge Promotional Cards | 3 | 2025-12-01 |
| `SFD` | Spiritforged | 288 | 2026-02-13 |
| `UNL` | Unleashed | 280 | 2026-05-08 |
| `VEN` | Vendetta | 358 | 2026-07-31 |

Set shape: `{ id, name, set_id, card_count, tcgplayer_id, cardmarket_id, published_on }`.

**`cardmarket_id` is polymorphic.** Measured across the 8 sets it is a string (`"6286"`), an
array of strings (`["6322","6483"]`), **and** `null`. A type that assumes any one of those
crashes on the others. This is the exact shape of bug a field census catches and a reading of
the docs does not.

`card_count` on the set is the printed denominator (`219` for UNL appears in the printed ids
while `card_count` is 280, because promos and showcase printings extend past the base set).
Do not assume the two agree.

**There are no set logo or symbol images in this API.** Set identity has to be typographic —
the `set_id` code in a tinted pill. Do not invent image URLs for them.

## Images

Host: `cmsassets.rgpub.io` on all 1451 cards (Riot's Sanity CDN). Shape:
`https://cmsassets.rgpub.io/sanity/images/dsfx7636/game_data_live/<sha1>-<W>x<H>.png?accountingTag=RB`

Measured dimensions: `744x1039` (1336), `1039x744` (44), `1038x744` (26), `1488x2078` (26),
`744x1040` (17), `744x1038` (1), `1040x744` (1).

Two consequences:

- **Never derive orientation from the URL.** Seven dimension variants exist and one of them is
  a 2× portrait. Use the `orientation` field.
- CSS aspect ratios `744/1039` portrait and `1039/744` landscape cover every variant within a
  pixel; a cover fit absorbs the difference.

**Battlefield cards are printed double-headed.** Landscape art contains the rules text twice:
upright at the bottom and rotated 180° along the top edge, because the card lies on the table
between two players. This is correct. Do not rotate, crop or "fix" it.

## text.rich — closed vocabulary, still not safe

`text.rich` is HTML with a tiny, closed tag vocabulary. Measured over all 1451 cards, the only
tags that ever appear: `<p>`/`</p>` (1431 each), `<br />` (899), `<li>`/`</li>` (49 each),
`<ul>`/`</ul>` (18 each). Entities: `&gt;` (153) and `&quot;` (32).

**That closed vocabulary is not permission to render it as HTML.** It must never reach
`dangerouslySetInnerHTML` or any equivalent. Tokenize against the allowlist and fall back to
`text.plain` on anything outside it — the data is third-party, the API is a work in progress,
and the cost of being wrong is cross-site scripting on every card page.

A tokenizer rather than a DOM parser is the right tool: scan `/<\/?([a-z]+)[^>]*>/gi`, reject
the whole string if any capture is outside `p|br|ul|li`, cap the input length, then split into
blocks and inline nodes.

### Inline symbol tokens

19 distinct `:rb_*:` tokens measured. Top: `:rb_might:` 543, `:rb_rune_rainbow:` 286,
`:rb_energy_1:` 226, `:rb_exhaust:` 191, `:rb_energy_2:` 99, `:rb_rune_fury:` 67,
`:rb_rune_body:` 57, `:rb_rune_order:` 53, `:rb_energy_0:` 49, `:rb_energy_3:` 48,
`:rb_rune_calm:` 44, `:rb_rune_mind:` 41, `:rb_rune_chaos:` 36, `:rb_energy_4:` 28.

Match `/:rb_[a-z0-9_]+:/g`. Parse the families — `energy_<N>`, `rune_<domain>` (where the
domain may be `rainbow`), and the bare `might` / `exhaust`. An unknown token must render as a
neutral pill carrying its inner name, never as the raw `:rb_foo:` text.

**Riot publishes no image URLs for these symbols.** Render them as CSS or SVG chips.

### Bracket keywords

53 distinct `[Keyword]` forms measured. Match `/\[[^\]]{1,24}\]/g`. Top: `[Reaction]` 153,
`[&gt;]` 147, `[Action]` 115, `[Empowered]` 98, `[Empower]` 77, `[Hidden]` 71, `[Equip]` 68,
`[Deflect]` 67, `[Ganking]` 64, `[Add]` 58, `[Accelerate]` 47, `[Tank]` 40, `[Temporary]` 34,
`[Assault 2]` 34, `[Deathknell]` 29, `[Ambush]` 27, `[Shield]` 26, `[Repeat]` 26.

Three special cases:

- `[&gt;]` and `[&gt;&gt;]` are activation arrows — render as `▸` / `▸▸`, not as text.
- `[NO TEXT]` means the card has no rules text. Render nothing.
- A numeric suffix is part of the keyword (`[Assault 2]`, `[Level 6]`). Keep it.

Do not hardcode the keyword list. Treat any bracket match as a keyword pill.

## Paging safely

```js
const first = await get("/cards?size=100&page=1");
const lastPage = Math.min(first.pages ?? 1, MAX_PAGES);   // MAX_PAGES = 50
```

The cap is not paranoia. `pages` is upstream-controlled, and an implausible value — from a
malformed response or a changed `size` semantics — turns one sweep into thousands of outbound
requests against a free service. Today's catalogue is 15 pages; a cap of 50 leaves headroom
and bounds the blast radius. Warn when the cap truncates, so the condition is visible rather
than silent.

Fetch remaining pages in small concurrent chunks. Four is the courtesy default: Riftcodex
publishes no limit and none was probed here, so the figure is borrowed from the throttling
measured on TCGdex rather than from this API.

## Re-verify

```bash
# the header question, both ways
curl -sS -o /dev/null -w "with UA    %{http_code}\n" -A "yourproject/1.0" "https://api.riftcodex.com/sets?size=3"
curl -sS -o /dev/null -w "without UA %{http_code}\n" -H "User-Agent:"     "https://api.riftcodex.com/sets?size=3"

# envelope and totals
curl -sS -A "yourproject/1.0" "https://api.riftcodex.com/cards?size=1&page=1" \
  | python3 -c 'import sys,json; d=json.load(sys.stdin); print({k:v for k,v in d.items() if k!="items"})'

# the polymorphic field
curl -sS -A "yourproject/1.0" "https://api.riftcodex.com/sets?size=100" \
  | python3 -c 'import sys,json; [print(s["set_id"], type(s.get("cardmarket_id")).__name__) for s in json.load(sys.stdin)["items"]]'

# full re-census: sweep all pages into one array, then census it (run from the skill root)
for p in $(seq 1 15); do
  curl -sS -A "yourproject/1.0" "https://api.riftcodex.com/cards?size=100&page=$p"
done | python3 -c 'import sys,json; print(json.dumps([c for l in sys.stdin for c in json.loads(l)["items"]]))' > /tmp/riftbound.json
node scripts/field-census.mjs /tmp/riftbound.json
```

Then update the date at the top of this file and every count the census contradicts.

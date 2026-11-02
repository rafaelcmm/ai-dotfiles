# Other games and price sources — leads, not contracts

**Status: these are leads.** Each entry below was reachability-checked on 2026-11-02 — the URL
was called and the status recorded — but **none has been field-censused** the way TCGdex,
LorcanaJSON and Riftcodex have in this skill. Treat every one as "worth starting here", never
as "this is what it returns".

Before building on any of them, run the first-contact protocol from the parent skill: probe
one record, pull a corpus, census the fields, write the contract down with its date.

## Contents

- [How to read the status column](#how-to-read-the-status-column)
- [Magic: The Gathering — Scryfall](#magic-the-gathering--scryfall)
- [Yu-Gi-Oh — YGOPRODeck](#yu-gi-oh--ygoprodeck)
- [Digimon — digimoncard.io](#digimon--digimoncardio)
- [One Piece](#one-piece)
- [Pokémon — the alternative to TCGdex](#pokémon--the-alternative-to-tcgdex)
- [Prices and products — TCGCSV](#prices-and-products--tcgcsv)
- [Multi-game commercial APIs](#multi-game-commercial-apis)
- [Games with no checked source here](#games-with-no-checked-source-here)
- [Choosing between a fan API and a price aggregator](#choosing-between-a-fan-api-and-a-price-aggregator)

## How to read the status column

| Status | Means |
|---|---|
| **reachable** | the exact URL below returned 2xx with plausible JSON on 2026-11-02 |
| **auth required** | returned 401 with a documented path to get a key |
| **unconfirmed** | the URL shape circulating publicly did not resolve; find the current one before using it |

## Magic: The Gathering — Scryfall

- Docs: <https://scryfall.com/docs/api>
- Base: `https://api.scryfall.com`
- Status: **reachable** — `GET /cards/named?exact=Black+Lotus` returned a full card object;
  `GET /bulk-data` returned 7 bulk file descriptors.

The best-engineered source in this list. Notable because the design is worth copying rather
than only consuming:

- **Bulk files exist precisely so you do not crawl.** `GET /bulk-data` lists
  `oracle_cards`, `unique_artwork`, `default_cards`, `all_cards`, `rulings`, `art_tags`,
  `oracle_tags`, each with a `download_uri` and an `updated_at`. All seven were refreshed the
  morning they were checked, so the cadence is daily. Pull the bulk file, not the card
  endpoint.
- `GET /sets` returned 1066 sets — Magic's catalogue is an order of magnitude larger than the
  other games here, which changes the ingestion maths. Projection is not optional.
- Scryfall documents a rate limit and asks for a delay between requests, and asks for an
  identifying `User-Agent` and an explicit `Accept` header. **Read the current numbers off the
  docs page** rather than from any note, including this one. No `x-ratelimit-*` response
  headers are served, so the policy is the only signal.
- Card **images** carry separate redistribution terms from card **data**. Check the docs
  before caching or re-serving art.

## Yu-Gi-Oh — YGOPRODeck

- Docs: <https://ygoprodeck.com/api-guide/>
- Base: `https://db.ygoprodeck.com/api/v7`
- Status: **reachable** — `GET /cardinfo.php?name=Dark%20Magician` returned
  `{"data":[{id, name, typeline, type, humanReadableCardType, ...}]}`.

Notes from the probe: responses wrap in a `data` array even for a single-name lookup, and
`typeline` is an array while `type` is a string — the kind of near-duplicate field pair a
census exists to disambiguate. The endpoint path still carries a `.php` suffix.

## Digimon — digimoncard.io

- Base: `https://digimoncard.io/api-public`
- Status: **reachable** — `GET /search?n=Agumon` returned an array of card objects
  (`{name, type, id, level, play_cost, evolution_cost, evolution_color, ...}`), and
  `GET /getAllCards?sort=name&series=Digimon%20Card%20Game` returned the full list.

Two gotchas already visible: the legacy `/search.php` form **301-redirects** to `/search`, so
follow redirects or use the current path; and `getAllCards` returns a thin projection
(`{name, cardnumber}`) rather than full records, so plan on a second pass for detail.

## One Piece

No free, unauthenticated source resolved on 2026-11-02.

- `optcgapi.com` — **unconfirmed**. `/api/allCards/` and `/api/sets/` both returned 404.
- `apitcg.com` — **auth required**. `GET https://api.apitcg.com/api/one-piece/cards?name=Luffy`
  returned 401 with `{"error":"API key is required, register into: https://apitcg.com/platform"}`.
  Multi-game, so one key covers several titles.

For One Piece today, the TCGCSV route below gives you products, images and prices without a
key, at the cost of card-game semantics (no card text, no colours, no costs).

## Pokémon — the alternative to TCGdex

- Docs: <https://docs.pokemontcg.io>
- Base: `https://api.pokemontcg.io/v2`
- Status: **partially reachable and flaky**. `GET /sets?pageSize=1` returned 200 with a clean
  set object; `GET /cards?pageSize=1` returned **500** on two attempts several minutes apart within the same
probe run.

Use TCGdex as the primary Pokémon source — it is multilingual, has an art CDN, and now carries
pricing. Keep this one as a cross-check for English-only data, and expect to need an API key
for any meaningful rate allowance. Do not design a request path around a source that 500s on
its main endpoint.

## Prices and products — TCGCSV

- Docs: <https://tcgcsv.com/>
- Base: `https://tcgcsv.com`
- Status: **reachable**, unauthenticated.

A free mirror of TCGplayer's catalogue and price data. This is the single most useful entry in
this file, because its **94 categories** cover every game in this skill and most of the ones it
has no card-level source for.

Endpoint shape, all confirmed:

```
GET /tcgplayer/categories                       -> 94 categories
GET /tcgplayer/<categoryId>/groups              -> sets/groups within a category
GET /tcgplayer/<categoryId>/<groupId>/products  -> products with names, images, URLs
GET /tcgplayer/<categoryId>/<groupId>/prices    -> prices per productId and subTypeName
```

Category ids measured:

| id | category | id | category |
|---|---|---|---|
| 1 | Magic | 71 | Lorcana TCG |
| 2 | YuGiOh | 79 | Star Wars Unlimited |
| 3 | Pokemon | 81 | Union Arena |
| 63 | Digimon Card Game | 85 | Pokemon Japan |
| 68 | One Piece Card Game | 86 | Gundam Card Game |
| | | 89 | Riftbound League of Legends Trading Card Game |

Shapes seen:

```jsonc
// products
{ "productId": 712806, "name": "Legacy - Booster Display", "cleanName": "...",
  "imageUrl": "https://tcgplayer-cdn.tcgplayer.com/product/712806_200w.jpg",
  "categoryId": 89, "groupId": 24832, "url": "https://www.tcgplayer.com/product/...",
  "modifiedOn": "<ISO-8601 timestamp>" }

// prices
{ "productId": 712806, "lowPrice": 234.11, "midPrice": 234.35, "highPrice": 349.99,
  "marketPrice": null, "directLowPrice": null, "subTypeName": "Normal" }
```

Three things to know before relying on it:

- **`marketPrice` and `directLowPrice` are nullable** and were null on real rows in the first
  group checked. Any arithmetic needs a null guard, and "no market price" is a display state
  you must design for.
- It is a **product** catalogue, not a card database. A row can be a booster display or a
  sealed box. Joining it to card-level data means matching on the game's own id, usually
  through a `tcgplayer_id` the card source already carries — Riftcodex and TCGdex both do.
- Prices change daily. They do not belong in the same long-lived snapshot as immutable card
  identity. Keep the two on separate refresh cadences.

## Multi-game commercial APIs

Worth knowing when free sources do not cover a game, or when you need one schema across many.
All require accounts, so verify current pricing and terms yourself — none of this was tested
beyond reachability.

- **apitcg.com** — many games behind one key; registration at `apitcg.com/platform`.
- **tcgapi.dev** — advertises 58 games with a free tier bounded at 100 requests per day.
- **tcggraph.com** — advertises 9 games with shared core fields plus per-game stats.
- **justtcg.com** — pricing-focused across Magic, Pokémon, Yu-Gi-Oh, Lorcana, One Piece,
  Digimon and Union Arena.

A unified schema is a real benefit and a real cost: the shared core (name, set, rarity, artist,
image, price) is exactly the part every free source already gives you, while the per-game
specifics you actually need for a good card page are the part that gets flattened away.

## Games with no checked source here

Star Wars Unlimited, Union Arena, Gundam, Flesh and Blood, Grand Archive, Dragon Ball Super,
Weiss Schwarz, Altered, Sorcery. Each appears in TCGCSV's category list, so products and
prices are available without a key; card-level data needs its own search.

When you find one, add it to this file with its probe date and status, and promote it to its
own reference file once you have censused it.

## Choosing between a fan API and a price aggregator

| You need | Go to |
|---|---|
| card text, costs, types, legality, localized names | the game's own fan API |
| high-resolution card art | the game's own fan API or publisher CDN |
| current market prices | TCGCSV, or a card source that embeds pricing |
| sealed product data | TCGCSV |
| one schema across many games | a commercial multi-game API, with the flattening caveat above |
| a game nobody has mapped | the first-contact protocol in the parent skill |

The two kinds compose well: identity and rules text from the fan API, frozen into a snapshot;
prices from the aggregator, refreshed on their own schedule and joined on the TCGplayer id.

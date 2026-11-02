# TCGdex — Pokémon TCG

**Last verified 2026-11-02.** Everything below is a measurement taken on that date against
the live service, not a contract the operators owe you. Re-measure before relying on it.

Docs home: <https://tcgdex.dev> — REST, GraphQL and SDKs (Java, Kotlin, PHP, JavaScript,
TypeScript). Self-described as "The Multilingual Pokemon TCG API", free and open source.

## Contents

- [Transport](#transport)
- [Endpoints worth knowing](#endpoints-worth-knowing)
- [Languages](#languages)
- [Card shape](#card-shape)
- [Pricing, embedded per variant](#pricing-embedded-per-variant)
- [Set shape](#set-shape)
- [Asset URLs — the suffix trap](#asset-urls--the-suffix-trap)
- [Deriving an art URL without the API](#deriving-an-art-url-without-the-api)
- [Vocabulary endpoints](#vocabulary-endpoints)
- [The SDK and its one hazard](#the-sdk-and-its-one-hazard)
- [Courtesy limits](#courtesy-limits)
- [Re-verify](#re-verify)

## Transport

- REST base: `https://api.tcgdex.net/v2/<lang>` — the language is a **path segment**, so it is
  part of every request rather than a header or query parameter.
- GraphQL: `https://api.tcgdex.net/v2/graphql` answered `200` to a `POST` of
  `{"query":"{__typename}"}` when measured.
- Asset CDN: `https://assets.tcgdex.net`.
- No authentication, no API key.
- Unknown card id returns a clean `404` with a JSON error body whose `type` points at
  `https://tcgdex.dev/errors/not-found`. A 404 is an answer about that card — never retry it.
- `GET /v2/` itself returns `404`; there is no self-describing root.
- A `User-Agent` was not enforced when measured. Send one anyway — see the courtesy section.

## Endpoints worth knowing

| Path (below `/v2/<lang>`) | Returns |
|---|---|
| `/cards/<cardId>` | one full card, e.g. `/pt/cards/me04-003` |
| `/sets/<setId>` | set metadata **plus a `cards` array** of `{id, image, localId, name}` |
| `/sets` | every set, with `cardCount` |
| `/series` | every series, `{id, name}` |
| `/rarities` | the rarity vocabulary, localized |

The `/sets/<setId>` payload is the efficient way to enumerate a set: one request yields every
card's id, localId, name and image base. What it does **not** carry is `rarity` — that needs
one `/cards/<id>` request per card. Measured at roughly 0.55 s per card, so a 122-card set is
a minute or two. Unacceptable on a request path; unremarkable in an offline batch, and the
single clearest argument for ingesting into a snapshot.

## Languages

Documented as "more than 10": English, French, Spanish, Italian, Portuguese (Brazilian),
German, Japanese, Chinese (Traditional), Indonesian, Thai, with more marked coming soon.

Codes observed to resolve when probed individually: `en, fr, es, it, pt, pt-br, de, nl, pl,
ru, ja, ko, id, th, zh-cn, zh-tw`. That is wider than the documented list, because the path
segment accepts codes the docs page has not caught up with.

There is **no endpoint that enumerates languages** — `GET /v2/` is a 404 — so the only way to
learn whether a code works is to request a known card id under it and read a 404 as "not this
language". Keep both `pt` and `pt-br` if you serve Brazilian Portuguese: they are distinct
codes and only one of them may hold the card you want.

Language is not cosmetic: the `image` base returned embeds the locale (`/pt/...` versus
`/en/...`), so fetching in the right language is also how you get the right printed artwork.
A card may exist in one language and 404 in another — fall back rather than failing.

## Card shape

Measured from `GET /v2/pt/cards/me04-003`:

```jsonc
{
  "category": "Pokémon",              // localized: "Pokemon" in en, "Pokémon" in pt
  "id": "me04-003",
  "illustrator": "toriyufu",
  "image": "https://assets.tcgdex.net/pt/me/me04/003",   // EXTENSION-LESS base
  "localId": "003",                   // printed collector number, zero-padded, string
  "name": "Beedrill ex",
  "rarity": "Rara Dupla",             // localized
  "set": {
    "cardCount": { "official": 86, "total": 122 },
    "id": "me04",
    "logo": "https://assets.tcgdex.net/pt/me/me04/logo",
    "name": "Caos Ascendente",
    "symbol": "https://assets.tcgdex.net/univ/me/me04/symbol"
  },
  "variants": { "firstEdition": false, "holo": true, "normal": false, "reverse": false, "wPromo": false },
  "variants_detailed": [ /* see pricing below */ ]
}
```

Fields to treat as optional regardless of what any doc implies: `image` (some cards genuinely
have no art), `rarity`, `illustrator`, and every member of `set.cardCount`.

`localId` is a string and may be non-numeric (`"TG01"`, `"GG05"`, `"SV01"`). Sorting a
checklist therefore needs a comparator that puts the numeric run first and the lettered promos
after it, or the order flips between runs and every sync produces a diff.

`official` and `total` in `cardCount` are different numbers and mean different things.
`official` is the denominator printed on the card (`003/86`); `total` includes secret rares
beyond it. Using the wrong one renders a wrong collector code on every card in the set.

## Pricing, embedded per variant

Measured 2026-11-02: each entry of `variants_detailed` carries pricing from both marketplaces.
It was absent when the API was measured on 2026-08-12, so treat its presence as measured rather
than promised.

```jsonc
{
  "type": "Holo",
  "size": "Padrão",
  "variantId": "jr7oetx1mqug9",
  "thirdParty": { "cardmarket": 886395, "tcgplayer": 693453 },
  "pricing": {
    "cardmarket": { "updated": "2026-11-01T22:54:33.737Z", "unit": "EUR",
                    "avg": 0.56, "low": 0.03, "trend": 0.55,
                    "avg1": 0.63, "avg7": 0.6, "avg30": 0.56,
                    "avg-holo": null, "low-holo": null, "trend-holo": 0 },
    "tcgplayer": { "unit": "USD", "updated": "<ISO-8601, as cardmarket above>" }
  }
}
```

Consequences worth acting on:

- You may not need a separate price source for Pokémon at all.
- Prices carry their own `updated` timestamp. Show it, or you are presenting a stale number as
  current.
- Price fields are nullable and sparsely populated per variant. Never arithmetic on them
  without a null check.
- A price is the one field in this payload that genuinely changes daily, so it must **not** be
  frozen into a long-lived snapshot alongside the immutable card identity. Split the two.

## Set shape

`GET /v2/pt/sets/me04` returns a richer `cardCount` than the card-embedded one:
`{"firstEd":0,"holo":62,"normal":69,"official":86,"reverse":76,"total":122}`, plus `name`,
`releaseDate`, `serie` and the `cards` array.

Guards worth coding when you ingest a set: abort if `cardCount.total` is absent, if
`cards.length` is zero, or if `cards.length < cardCount.total`. Each of those states means a
truncated download or a changed contract, and committing one ships an incomplete checklist
that still looks complete.

## Asset URLs — the suffix trap

Two different rules, and mixing them up produces a 404 that reads like a missing card.

| Asset | Base from the API | Full URL | Measured |
|---|---|---|---|
| card art | `.../pt/me/me04/003` | `.../003/high.webp` | `200`, `image/webp`, 90942 bytes |
| card art | `.../pt/me/me04/003` | `.../003.webp` | 404 |
| set logo | `.../pt/me/me04/logo` | `.../logo.png` | `200`, 165727 bytes |
| set logo | `.../pt/me/me04/logo` | `.../logo/high.png` | 404 |

So: **card art appends `/<quality>.<ext>`; logo and symbol append `.<ext>` directly.**

`quality` is `low` or `high`. `ext` is `png`, `webp` or `jpg`. Measured on the same card:
`high.webp` 90,942 bytes, `high.png` 321,358 bytes, `low.webp` 23,618 bytes. Prefer WebP, keep
PNG as the fallback for any consumer that rejects WebP, and use `low` for grid thumbnails.

Not every asset exists in every extension — `univ/me/me04/symbol.webp` returned 404 while the
logo's `.png` resolved. Probe the extension you intend to use rather than assuming parity.

The CDN returned a transient `503` during measurement and `200` on retry seconds later for the
same URL. Treat a 5xx from the asset host as transport, not as a missing image.

## Deriving an art URL without the API

Useful when `api.tcgdex.net` is unreachable but `assets.tcgdex.net` is up — a real outage
shape, observed on 2026-08-31 with TCP refused on both A and AAAA records of the API host
while the CDN served normally.

Art lives at `assets.tcgdex.net/<lang>/<serie>/<setId>/<localId>/high.<ext>`, and a card id is
`<setId>-<localId>`, so every component is recoverable from the id alone once the release
number is stripped off the set id to get the serie: `me05` → `me`, `me02.5` → `me`,
`sv09` → `sv`.

Two conditions make guessing safe rather than reckless:

- **HEAD-verify every derived URL** before using it. A path you got wrong resolves to "no art
  for this card", never to the wrong card's art.
- Split the id on the **last** hyphen — a set id may contain one, a collector number never
  does — and pattern-check each component before interpolating it into a URL. These values
  often arrive from user- or operator-editable configuration.

A 200 at a path built from the card's own id is that card by construction, which is why the
derivation is sound even though it is a guess about layout.

## Vocabulary endpoints

`/rarities` returns the live rarity list, localized — measured in `pt` as `ACE SPEC Raro`,
`Arte Completa de Treinador`, `Brilhante Ultra Rara`, `Comum`, `Hiper rara`,
`Ilustração Rara`, `Ilustração Rara Especial`, `Incomum`, `Mega Hiper Raro`, `None`, `Promo`,
`Rara`, `Rara Dupla`, `Rara Holo`, and more.

Read it; do not hardcode it. New sets introduce new rarities, and `"None"` is a real value a
naive mapping treats as missing. `/series` and `/sets` serve the same purpose for their axes.

## The SDK and its one hazard

`@tcgdex/sdk` wraps the REST API. `new TCGdex('pt-br')` then `await tcgdex.card.get('swsh3-136')`.

**`setLang` mutates the instance.** A shared singleton plus `setLang` races under concurrent
requests for different locales: one request changes the language out from under another and
gets back the wrong localization. Keep one memoized instance per language instead.

The SDK's card object carries methods, so it is not serializable. Project it to a plain object
before caching or storing it, and keep `image` as the extension-less string, building full
URLs at render time.

## Courtesy limits

No published rate limit. Measured behaviour: throttling begins somewhere around four to eight
concurrent fetches from one IP. Six worked for a sustained 684-card batch; four is the safe
default. This is a courtesy ceiling, not a performance target — there is nothing to win by
raising it and a shared free service to lose.

Refuse redirects on these endpoints (`redirect: "manual"`). They answer directly today, so a
3xx means something other than TCGdex is replying, and a followed redirect is an unvalidated
outbound request from whatever machine runs your sync.

## Re-verify

```bash
# one card, full shape
curl -sS -A "yourproject/1.0 (+https://example.com)" \
  https://api.tcgdex.net/v2/pt/cards/me04-003 | python3 -m json.tool | head -60

# set metadata + card list, and the two cardCount denominators
curl -sS -A "yourproject/1.0" https://api.tcgdex.net/v2/pt/sets/me04 \
  | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d["cardCount"], len(d["cards"]))'

# the asset suffix rules, both of them
curl -sS -o /dev/null -w "art  %{http_code} %{size_download}\n" https://assets.tcgdex.net/pt/me/me04/003/high.webp
curl -sS -o /dev/null -w "logo %{http_code} %{size_download}\n" https://assets.tcgdex.net/pt/me/me04/logo.png

# the live rarity vocabulary
curl -sS https://api.tcgdex.net/v2/pt/rarities
```

Then update the date at the top of this file and whatever the output contradicts.

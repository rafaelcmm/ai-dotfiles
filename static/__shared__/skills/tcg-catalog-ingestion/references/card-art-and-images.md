# Card art and images

Card artwork carries problems that card data does not: it is large, it lives on third-party
CDNs, its URL is often constructed rather than given, and the printed card's orientation does
not always match the file's.

## Contents

- [The origin allowlist comes first](#the-origin-allowlist-comes-first)
- [Read the URL when you can; derive it only with verification](#read-the-url-when-you-can-derive-it-only-with-verification)
- [URL construction rules differ per asset type](#url-construction-rules-differ-per-asset-type)
- [Orientation and rotation](#orientation-and-rotation)
- [Rendering a rotated source](#rendering-a-rotated-source)
- [Hotlink, proxy or re-host](#hotlink-proxy-or-re-host)
- [Format and size](#format-and-size)
- [Alt text](#alt-text)
- [Failure states](#failure-states)
- [Checklist](#checklist)

## The origin allowlist comes first

Card art URLs reach your code from three untrustworthy places: a third-party API response, a
committed index generated from one, and editable configuration. If any of them feeds an
outbound `fetch`, that is a server-side request forgery vector, and if any feeds an `<img>` it
is a content-injection vector.

```js
/** Origins whose card art may be fetched. Full host-plus-path prefixes. */
export const ALLOWED_ART_ORIGINS = [
  "https://assets.tcgdex.net/",
  "https://api.lorcana.ravensburger.com/images/",
  "https://cmsassets.rgpub.io/sanity/images/",
];

/** Longest art URL accepted. Bounds the work a pathological value can cause before any parse. */
export const MAX_ART_URL_LENGTH = 512;

/**
 * Reports whether a card-art URL may be fetched or rendered.
 * @param {unknown} url - Candidate URL from any source: an API response, a generated index, or
 *   editable configuration. Non-strings are rejected rather than coerced.
 * @returns {boolean} True only for an exact `https://host/path/` prefix match against
 *   ALLOWED_ART_ORIGINS. A false return is a finding to report, not a value to repair.
 */
export function isAllowedArtUrl(url) {
  if (typeof url !== "string" || url.length > MAX_ART_URL_LENGTH) return false;
  if (/[\s\r\n\t]/.test(url)) return false;
  return ALLOWED_ART_ORIGINS.some((prefix) => url.startsWith(prefix));
}
```

Four details that are all load-bearing:

- **Match a full `https://host/path/` prefix, not a hostname.** Hostname comparison alone
  permits `https://cmsassets.rgpub.io/evil/...` on a CDN that serves arbitrary tenants.
- **Prefix matching catches the lookalike host** that a naive `includes()` check does not:
  `https://assets.tcgdex.net.evil.example/x` fails `startsWith("https://assets.tcgdex.net/")`
  because of the trailing slash. Keep the slash.
- **Reject before doing work.** Length and whitespace checks run before any parsing or
  fetching, so a pathological value costs nothing.
- **Reject embedded whitespace and newlines.** A newline in a URL is request smuggling.

Write the allowlist **from the measured hosts**, not from guesses. A field census that reports
the distinct hosts per URL field hands you the list directly.

Re-apply the check **on read**, not only on write. Data that passed through anywhere editable
is input again, even if you generated it yourself.

Where a security boundary has to be duplicated — a plain `.mjs` sync script cannot import a
typed runtime module — duplicate it with a comment on both copies naming the other, and add a
test asserting the two lists are equal so they cannot drift silently.

## Read the URL when you can; derive it only with verification

Sources differ, and the difference matters:

- **Given, unguessable.** Lorcana image URLs embed a content hash
  (`.../32_15cb…60.jpg`). There is no derivation. You must read the URL from the data.
- **Given, but also derivable.** TCGdex returns an extension-less `image` base and the base is
  reconstructible from the card id. Read it normally; derive only as an outage fallback.

When you must derive, **HEAD-verify every derived URL before using it**. A path you got wrong
then resolves to "no art for this card" and is reported as unresolvable — it can never become
an upload or a render of the *wrong* card's art. That single check is what makes the guess
sound rather than reckless.

A derived path built from the card's own id is that card by construction, which is the other
half of the argument: there is no id whose art could legitimately answer at that path instead.

When parsing an id into path components, split on the **last** separator — a set id may contain
a hyphen, a collector number never does — and pattern-check every component before
interpolating it. These values arrive from editable configuration.

## URL construction rules differ per asset type

Even within one source. Measured on TCGdex:

| Asset | Base from the API | Full URL |
|---|---|---|
| card art | `.../me04/003` | `.../003/high.webp` — quality is a **path segment** |
| set logo | `.../me04/logo` | `.../logo.png` — extension appended **directly** |

Getting the two backwards produces a 404 that reads like a missing card. And not every asset
exists in every extension — `univ/me/me04/symbol.webp` returned 404 while the set logo's `.png`
resolved. Probe the exact asset and extension you intend to use.

Store the **base** and build full URLs at render time. Storing a fully-qualified URL freezes a
quality and format choice into your data and makes changing it a migration.

## Orientation and rotation

Three distinct properties that are easy to conflate:

- **The card's printed orientation** — portrait or landscape.
- **The file's orientation** — which may not match.
- **The rotation needed to display it upright.**

Two measured cases worth knowing because both look like your own bug:

**Lorcana Locations.** Every image file is portrait 1468×2048, *including* Location cards,
which are landscape. The art inside sits rotated 90° counter-clockwise — the printed title
reads bottom-to-top. Render Locations in a landscape frame with a +90° clockwise correction.
Nothing in the data says the file is rotated and the image loads perfectly, so this costs an
afternoon to diagnose from scratch.

**Riftbound Battlefields.** Landscape art contains the rules text **twice**: upright at the
bottom and rotated 180° along the top edge, because the card lies on the table between two
players. **This is correct.** Do not rotate, crop or "fix" it.

**Never derive orientation from image dimensions.** Riftbound serves seven dimension variants
including a 2× portrait; use the source's own `orientation` field. Where a source has no such
field, derive orientation from the card **type** (Location, Battlefield) rather than from the
file.

Carry both as explicit fields on your projected record — an orientation and a rotation — so
every surface renders the same way and the correction lives in one component.

## Rendering a rotated source

A landscape frame holding a portrait source rotated 90°: the pre-rotation box must be as tall
as the frame is wide and as wide as the frame is tall.

```html
<div style="position:relative; overflow:hidden; aspect-ratio:2048/1468">
  <div style="position:absolute; left:50%; top:50%;
              width:calc(100% * 1468 / 2048);
              height:calc(100% * 2048 / 1468);
              transform:translate(-50%,-50%) rotate(90deg);
              transform-origin:center">
    <img src="..." style="width:100%; height:100%; object-fit:cover" alt="...">
  </div>
</div>
```

Compute the two factors from the aspect ratio rather than hardcoding them, so the component
stays correct if a source's ratio changes — the snippet inlines Lorcana's 1468×2048 for
legibility, but a real component derives both from the stored ratio.

Aspect ratios are worth storing per orientation (`744/1039` and `1039/744` for Riftbound; a
±1px variation across files is absorbed by a cover fit). Set the frame's aspect ratio
explicitly so there is no layout shift while the image loads.

## Hotlink, proxy or re-host

| Approach | When | Cost |
|---|---|---|
| **Hotlink** | prototypes, low traffic, terms permit it | the source's CDN pays your bandwidth; the origin must be allowed in your image config and content-security policy; a dead source means dead images |
| **Proxy** | you want same-origin images without storing bytes | one outbound fetch per image on a cache miss; your server is on the image path |
| **Re-host** | production, anything at scale | a sync step and storage; images survive the source going away |

**Re-hosting is the right default for production**, for the same reason a committed snapshot
beats a request-path fetch: it removes a third party from the hot path. Your own storage origin
is usually already allowed by your image config and content-security policy, so re-hosted art
needs no new origin review.

If you proxy: enforce the origin allowlist on the `src` parameter, restrict the **response**
content type to an image allowlist and reject anything else, set a long cache header, and send
a `User-Agent` on the outbound fetch.

Keep the original source URL alongside the re-hosted one. It is the fallback when an upload
has not run or failed, and it is how a mismatch gets diagnosed.

Check the source's terms before re-hosting. Some sources license data and images differently.

## Format and size

WebP is several times smaller than PNG for the same card — measured on one TCGdex card at
90,942 bytes against 321,358 bytes, with its low-quality WebP at 23,618 bytes. Prefer WebP and
keep PNG as the fallback for any consumer that rejects WebP.

Where a source offers quality tiers, use the small one for grids and the large one for detail
views. Store the base and pick the tier at the call site.

A 5xx from an image CDN is transport, not a missing image — one measured URL returned 503 and
then 200 seconds later. Retry; do not record the card as artless.

## Alt text

Some sources publish it. Riftbound ships a publisher-written `accessibility_text` per card;
use it verbatim.

Otherwise compose it deterministically from card name, printed number and set name. Determinism
matters beyond accessibility: if a re-host step uses alt text to detect "this card's art is
already uploaded", a non-deterministic string re-uploads everything on every run. Collapse
whitespace, trim, and cap the length.

## Failure states

- **No art URL at all** — a real case for some promos. Render a placeholder shaped to the
  frame, not a broken image.
- **URL fails the allowlist** — blank it in the projection rather than storing a value the
  reader would reject anyway, and report it.
- **Upload or proxy failed** — fall back to the original source URL. An image degrades to a
  slower image, never to nothing.
- **Source CDN down** — placeholder, and do not let it fail the page.

## Checklist

- [ ] Every art URL passes a full-prefix origin allowlist before any fetch or render.
- [ ] The allowlist was written from measured hosts.
- [ ] The allowlist is re-applied on read, not only on write.
- [ ] Duplicated allowlists have a test asserting they are equal.
- [ ] Derived URLs are HEAD-verified before use.
- [ ] Ids are split on the last separator and pattern-checked before interpolation.
- [ ] Orientation comes from the source or the card type, never from file dimensions.
- [ ] Rotation is an explicit field, and the correction lives in one component.
- [ ] Frames declare an aspect ratio, so there is no layout shift.
- [ ] Bases are stored; full URLs are built at render time.
- [ ] A placeholder exists for every failure state.
- [ ] Re-hosted art keeps its original URL as a fallback.
- [ ] Image terms were checked separately from data terms.

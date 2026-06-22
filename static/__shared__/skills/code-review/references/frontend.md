# Frontend-specific review

Load this lens only when the change touches frontend code (components, hooks, styles, DOM/event
handling). It supplements the general lenses in `review-lenses.md`.

## 1. Is JS doing what HTML or CSS does natively?

Frontend code frequently accumulates JavaScript reimplementing things the platform already
handles — which adds complexity, introduces bugs around edge cases the browser handles for free,
and usually produces worse accessibility and performance. The browser's built-in behaviors are
more accessible, more performant, and more correct than equivalent JS in almost every case.
Prefer them. Ask:
- Is JS detecting outside clicks where a `<dialog>` or `:focus-within` would handle it?
- Is JS managing hover/focus state that CSS pseudo-classes handle natively?
- Is JS building a disclosure widget that `<details>` / `<summary>` would give for free?
- Is JS doing scroll-based effects that `IntersectionObserver` or CSS scroll-driven animations
  handle better?
- Is JS computing a value that a CSS custom property or `calc()` could handle?

## 2. Component behavior should be event-driven

Well-designed components don't reach out and change the world themselves — they **describe what
happened** and let consumers decide how to respond. Prefer specific, event-named props over
generic value-change callbacks:

- Weaker: `<ZipCode onChange={} />` — reports *what value changed*.
- Stronger: `<ZipCode onAutocompleteCity={() => {}} />` — reports *what happened* (the city was
  autocompleted), so the consuming context can decide how to respond: populate a field, validate
  a coverage area, log analytics, or do nothing.

Watch for components accepting generically named callbacks (`onChange`, `onUpdate`) where a more
specific event name would better describe the actual behavior and make consumer code more
self-documenting.

## 3. Async React patterns — are the right tools being used?

Modern React has a rich toolkit for async state that many teams underuse. Code that manages these
concerns manually with `useState` + `useEffect` + loading flags is often doing more work than it
needs to and inviting race-condition and stale-closure bugs that the concurrent APIs handle
correctly by design. When reviewing, ask whether the right tool is in play:
- **`useTransition`** — mark non-urgent state updates to keep the UI responsive during
  transitions.
- **`useOptimistic`** — reflect the expected result of an action immediately, before the server
  responds.
- **`Suspense`** — handle async data-loading boundaries cleanly instead of scattering
  `isLoading` state across components.
- **React Server Components & Actions** — move data logic closer to the server, reducing
  client-side complexity.
- **`Activity`** (formerly `OffscreenComponent`) — render inactive UI without showing it.

These are encouragements, not mandates — flag manual patterns where a concurrent API would remove
a class of bugs, but respect existing conventions and constraints.

## 4. When a core/shared component changes, spot-check ALL usages

If the change modifies a component used in many places — a form input, modal, data table, button
— **the diff review is not sufficient.** Pull up every usage and ask whether the change is
backward-compatible for all of them. Pay special attention to:
- **Prop interface changes** (renamed, removed, or newly required props).
- **Changes to default behavior.**
- **Changes to internal event handling.**
- **Changes to accessibility behavior** — focus management, ARIA roles, keyboard interaction.

These are semantic-contract changes on a shared component — the single most effective way to
introduce widespread, hard-to-detect regressions. Findings here are usually **Blocking** unless
you've confirmed every usage is safe.

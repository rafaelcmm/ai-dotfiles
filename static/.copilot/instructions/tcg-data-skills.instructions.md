---
description: "Always use the TCG data skills first when the request targets trading-card-game card data, its sources, or its ingestion."
---

# TCG Data Skills Rule

Sync note: this rule must stay semantically identical across Claude rules and Copilot instructions; update both files in the same change.

When the request involves trading-card-game card data, always consult the TCG skills before generic API or data-ingestion skills.

## Activation

Activate this rule when the request mentions a trading card game or one of its data sources: TCG, Pokémon TCG, TCGdex, Disney Lorcana, LorcanaJSON, Riftbound, Riftcodex, Magic the Gathering, Scryfall, Yu-Gi-Oh, YGOPRODeck, One Piece Card Game, Digimon Card Game, TCGplayer, TCGCSV, Cardmarket, card singles, card sets, card checklists, card rarities, card art or card prices.

## Activation Scope

Activate only when the request primarily targets trading-card-game data — fetching it, verifying a source's contract, ingesting it, caching it, or rendering card identity and art. Do not activate when a card game is mentioned in passing, or when the subject is gameplay rules rather than card data.

## Mandatory behavior

- Resolve scope first, then load the matching skill:
  - `tcg-card-data-sources` — which source to use, what it returns, and how to verify that it still does
  - `tcg-catalog-ingestion` — what to do with the data: request-path versus snapshot, projection, sync scripts, resilience, card art
- Load `tcg-card-data-sources` first whenever a source's field contract is not already proven in this session. Code written against a remembered contract is usually wrong in a way that only surfaces partway through a catalogue.
- Load both skills when the task spans discovery and ingestion, which is the common case.
- Treat every recorded field list, count, header requirement and URL shape as a dated measurement rather than a contract. Re-run the re-verification command in the skill before relying on it, and fix the reference file in the same change when the measurement disagrees.
- Never hardcode a value vocabulary — rarities, types, domains, inks, keywords. Derive facets from the data actually fetched.
- Never interpolate a card id or an art URL into an outbound request without the origin allowlist the skills specify.
- When no skill covers the game in question, follow the first-contact protocol in `tcg-card-data-sources` and record the measured contract as a new reference file in that skill.

## Agent delegation

- Delegate substantial TCG data work to `tcg-data-engineer` when it is available. That agent owns the pipeline from source selection through committed snapshot.
- Per-game differences are field contracts, not workflows. Keep them in the skill references; do not create a separate agent per game.

## Safety interaction

A request that would publish, redistribute or charge for card data additionally triggers the licensing constraints recorded per source. Several sources operate under a publisher community policy — LorcanaJSON may not be charged for, and some sources license images separately from data. Read the terms recorded in the source's reference file before designing around it.

## Priority

TCG skill guidance overrides generic API and data-ingestion guidance only within trading-card-game data decisions, and never overrides user intent, safety constraints, or higher-priority workflow contracts.

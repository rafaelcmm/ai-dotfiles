---
name: hexagonal-architecture
description: Ports and Adapters (Hexagonal) architecture for isolating domain logic from infrastructure concerns.
allowed-tools: [Read]
---

# Hexagonal Architecture

Hexagonal architecture keeps core business logic independent from frameworks, databases, and external systems.

## Purpose

- Protect domain logic from infrastructure changes
- Improve testability through dependency inversion
- Support swapping adapters with minimal core impact
- Clarify boundaries between business and technical concerns

## When to Reference This Skill

Reference when:

- Designing service/module boundaries
- Refactoring tightly coupled codebases
- Adding external integrations (DB, queue, API)
- Building testable application cores

## Core Concepts

| Concept             | Definition                                     |
| ------------------- | ---------------------------------------------- |
| Domain              | Pure business rules and invariants             |
| Port                | Interface used by or exposed from the domain   |
| Adapter             | Infrastructure implementation of a port        |
| Application Service | Orchestrates use cases across domain and ports |

## Dependency Rule

Dependencies point inward:

- Adapters depend on ports/domain
- Domain depends on nothing external
- Framework code lives at the edges

## Typical Layout

```text
core/
  domain/
  application/
  ports/
adapters/
  inbound/   (http, cli, events)
  outbound/  (db, cache, third-party api)
```

## Implementation Guidelines

- Define ports in the core (`UserRepository`, `PaymentGateway`)
- Keep domain entities framework-agnostic
- Map transport/persistence models in adapters, not core
- Enforce use-case boundaries with application services
- Wire dependencies in composition root

## Testing Strategy

- Unit test domain and use cases without infra
- Contract test adapters against port expectations
- Integration test composition root and runtime wiring

## Quick Checklist

```text
- [ ] Domain has no framework imports
- [ ] Ports are defined by business needs
- [ ] Adapters implement ports, not vice versa
- [ ] Data mapping stays at boundaries
- [ ] Composition root performs all wiring
```

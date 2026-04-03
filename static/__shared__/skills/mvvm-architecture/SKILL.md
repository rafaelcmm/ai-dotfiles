---
name: mvvm-architecture
description: MVVM architecture patterns for separating UI rendering, presentation state, and domain interactions.
allowed-tools: [Read]
---

# MVVM Architecture

Model-View-ViewModel (MVVM) separates visual concerns from state orchestration and business behavior.

## Purpose

- Keep UI components declarative and testable
- Centralize presentation logic in ViewModels
- Isolate domain/use-case interactions from rendering details
- Support predictable state transitions

## When to Reference This Skill

Reference when:

- Building stateful UI flows
- Refactoring fat UI components
- Defining frontend architecture boundaries
- Writing presentation-layer tests

## Layer Responsibilities

| Layer       | Responsibility                                         |
| ----------- | ------------------------------------------------------ |
| `Model`     | Domain entities, use cases, repositories               |
| `ViewModel` | Presentation state, commands, UI-ready transformations |
| `View`      | Rendering, events, binding to ViewModel output         |

## Interaction Flow

1. User interacts with View
2. View delegates action to ViewModel command
3. ViewModel executes use case/repository call
4. ViewModel updates observable state
5. View re-renders from updated state

## ViewModel Design Rules

- Expose immutable view state where possible
- Keep framework-specific APIs out of domain logic
- Represent loading/success/error explicitly
- Use intent-oriented methods (`submitOrder`, `retry`, `refresh`)
- Avoid direct persistence/networking inside the View

## State Modeling Example

```text
state = {
  status: 'idle' | 'loading' | 'success' | 'error',
  data: ViewData | null,
  errorMessage: string | null
}
```

## Common Pitfalls

- Putting UI formatting logic directly in Model
- Making ViewModel a thin pass-through without decisions
- Coupling ViewModel tightly to one UI framework
- Mixing transient UI events with persistent state

## Quick Checklist

```text
- [ ] View contains rendering + event wiring only
- [ ] ViewModel owns presentation decisions
- [ ] Model remains UI-agnostic
- [ ] State handles loading/error/empty explicitly
- [ ] Unit tests cover ViewModel transitions
```

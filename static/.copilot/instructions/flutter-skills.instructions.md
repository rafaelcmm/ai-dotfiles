---
description: "Always use the Dart and Flutter external skills first when the request targets Dart or Flutter code, tooling, tests, or packages."
---

# Flutter Skills Rule

Sync note: this rule must stay semantically identical across Claude rules and Copilot instructions; update both files in the same change.

When the request targets Dart or Flutter code, always consult the Dart and Flutter external skills before generic language or framework skills.

## Activation

Activate this rule when the request mentions Dart, Flutter, pub, pubspec.yaml, `dart analyze`, `dart fix`, `flutter test`, widgets, `WidgetTester`, `build_runner`, mockito, `package:checks`, go_router, `flutter_localizations`, Dart FFI, ffigen, native assets, Dart MCP server, hot reload, or hot restart, or when the files in play are `.dart` sources.

## Activation Scope

Activate only when the request primarily targets Dart or Flutter work. Do not activate when Dart or Flutter is mentioned in passing.

## Mandatory behavior

- Resolve scope first, then load only the most specific skill or minimal set of skills that match the request:
  - Architecture and UI: `flutter-apply-architecture-best-practices`, `flutter-build-responsive-layout`, `flutter-fix-layout-issues`, `flutter-add-widget-preview`
  - Navigation, data and localization: `flutter-setup-declarative-routing`, `flutter-use-http-package`, `flutter-implement-json-serialization`, `flutter-setup-localization`
  - Testing: `dart-add-unit-test`, `flutter-add-widget-test`, `flutter-add-integration-test`, `dart-generate-test-mocks`, `dart-migrate-to-checks-package`, `dart-collect-coverage`
  - Diagnostics and dependencies: `dart-run-static-analysis`, `dart-fix-runtime-errors`, `dart-resolve-package-conflicts`
  - Language features: `dart-use-pattern-matching`, `dart-use-primary-constructors`, `dart-use-path-package`
  - Command-line tools: `dart-build-cli-app`
  - Native interop: `dart-use-ffigen`, `dart-setup-ffi-assets`
  - Documentation: `dart-write-documentation`, `dart-use-doc-examples`
- Prefer the most specific skill for the request.
- If the request spans several areas, combine the relevant skills before falling back to generic cross-language skills.
- `dart-write-documentation` governs Dart doc-comment syntax and placement. `commenting-standards` still governs what every docblock must say; apply both, and the stricter one wins on conflict.
- Several skills (`dart-fix-runtime-errors`, `flutter-fix-layout-issues`, `flutter-add-integration-test`) drive the Dart MCP server. When that server is not connected, say so, fall back to the equivalent CLI command (`dart analyze`, `flutter test`, `flutter run`), and never report a hot reload or runtime check that did not run.
- When editing `.dart` files under `lib/` of a running app, also apply the `flutter-hot-reload` rule.

## Agent delegation

- Delegate substantial Dart or Flutter implementation work to `flutter-specialist` when it is available under `/agents`.

## Priority

Dart and Flutter skill guidance overrides generic framework guidance only within Dart and Flutter decisions, and never overrides user intent, safety constraints, or higher-priority workflow contracts.

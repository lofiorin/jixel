# Contributing to Jixel

Thank you for helping! Code, documentation, translations, bug reports and
sample images that show a problem are all welcome.

*Italiano:* i contributi in italiano sono benvenuti; issue e PR possono essere
scritte in italiano o in inglese.

## Before you start

- Read [docs/en/architecture.md](docs/en/architecture.md) (10 minutes).
- Set up the tools: [docs/en/building.md](docs/en/building.md), then
  `scripts/bootstrap.sh`.
- For anything larger than a small fix, open an issue first to agree on the
  approach.

## Principles

1. **Readable first.** This project is meant to be read by others. Prefer
   plain code over clever code, explain *why* in comments, keep functions
   small, name things for what they mean to a user.
2. **Image logic lives in `jixel-core`.** The CLI, the bridge and the app only
   translate and display.
3. **Honest results.** Never call something lossless unless it is. Every new
   operation that produces an image reports its `Fidelity`.
4. **Tests for behaviour.** A new feature comes with a test showing it works
   (round trips, size/quality guarantees). A bug fix comes with a test that
   failed before.
5. **Everything user-facing is translated** (English and Italian at least).

## Workflow

1. Fork and branch from `main` (`feat/…`, `fix/…`, `docs/…`).
2. Make the change, with tests and docs.
3. Run the checks in [building.md](docs/en/building.md#checks-before-a-pull-request).
4. Open a pull request describing *what* and *why*; screenshots for UI.

Commit messages: short imperative summary prefixed by area
(`core:`, `cli:`, `app:`, `docs:`, `ci:`), e.g. `core: keep ICC profile when cropping`.

## Style

- **Rust**: `cargo fmt`, `cargo clippy -- -D warnings`. Public items have doc
  comments. Errors go through `jixel_core::Error` with messages a user can act
  on.
- **Dart**: `dart format --line-length 120`, `flutter analyze` clean. Widgets
  small and named; no state-management packages.
- **Docs**: Markdown, English in `docs/en`, Italian in `docs/it`. Update both
  when you change behaviour (or open an issue asking for the translation).

## Adding things

**A new engine operation**
1. Implement it in `jixel-core` with tests.
2. Expose it in the CLI (`core/crates/jixel-cli/src/main.rs`) and document it in
   `docs/en/cli.md`.
3. Add a `match` arm in `app/rust/src/lib.rs` (document its arguments above
   it), then a method in `app/lib/engine/engine.dart`.

**A UI string**
Add a row to `app/tool/strings.py` (English and Italian), then
`python3 tool/strings.py && flutter gen-l10n && dart format --line-length 120 lib/l10n`.

**A language**
Add a column to every row in `app/tool/strings.py`, add the locale to the
`LOCALES` list in that file and to the language picker in
`app/lib/ui/screens/settings.dart`, and translate the docs if you can.

## Reporting bugs

Include: platform and version, Jixel version (Settings → About), steps, what
you expected, what happened, and — if possible — the image (or one that shows
the same problem). For security issues see [SECURITY.md](SECURITY.md).

## Licence

By contributing you agree that your contributions are licensed under the
Apache License 2.0.

# Localization

## Current status

- **Application source code and UI strings**: English only. Neither the CLI
  (`applications/cli`) nor the static Web application
  (`applications/web/index.html`, `app.js`) contain any translated strings
  or localization framework today.
- **Development documentation** (`README.md`, `ARCHITECTURE.md`,
  `PROJECT_GUIDELINES.md`, `CONTRIBUTING.md`, everything under
  `documentation/`): English only, per `CONTRIBUTING.md` ("Development
  documentation must be written in English").
- **User documentation** (`documentation/user_guides/`): currently English
  only. `PROJECT_GUIDELINES.md` states user documentation should eventually
  cover English and French, with Spanish planned after that; this has not
  been started.

## Scope for now

There is no localization library, resource catalog, or translation workflow
in this repository. `libraries/localization` mentioned in earlier
architecture notes is not implemented and is out of scope until a concrete
UI localization requirement and API are defined (see
`documentation/architecture/dependency_rules.md`).

## Before implementing UI localization

1. Decide which layer owns translated strings: the Rust core (for shared
   validation/error messages surfaced to every platform) versus each
   application (for UI-only text). Do not add translated strings to
   `ui24_core`, `audio_processing`, or `session_generator` without a
   documented reason — these crates are domain logic, not presentation.
2. Choose a resource format (for example flat JSON per locale) and a loading
   mechanism compatible with the offline-first, no-backend constraint used
   by the Web application.
3. Update this document with the chosen design before writing code, per the
   "Documentation First" principle in `PROJECT_GUIDELINES.md`.

## Translating user-facing documentation

When French user documentation is added:

- Keep it under `documentation/user_guides/`, alongside the English version,
  with a clear filename suffix (for example `user_manual.fr.md`).
- Do not translate developer-facing documentation
  (`documentation/developer_guides/`, `documentation/architecture/`); it
  stays in English.

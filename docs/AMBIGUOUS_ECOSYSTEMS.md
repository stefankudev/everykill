# Ambiguous Ecosystem Detection

## The Problem

Several dependency folder names are shared across multiple ecosystems. When everykill scans a directory it matches folder names against patterns in `ecosystems/*.json`. Because the scan only sees the folder name — not anything about the surrounding project — it attributes the first matching ecosystem to a folder, which is often wrong.

### Concrete Example

Running `everykill` in a Rust project's root:

```
target/             ← matched as "Clojure" (loaded first alphabetically)
```

`target/` is listed as a `local` pattern in four ecosystem files: `clojure.json`, `java.json`, `rust.json`, and `scala.json`. Whichever file loads first wins, which is effectively random from the user's perspective.

---

## Full Conflict Map

### `target/`

| Ecosystem | Marker files that confirm it |
|-----------|------------------------------|
| Rust | `Cargo.toml`, `Cargo.lock` |
| Java (Maven) | `pom.xml` |
| Clojure | `project.clj`, `deps.edn`, `build.boot` |
| Scala | `build.sbt`, `build.sc` |

### `build/`

| Ecosystem | Marker files |
|-----------|--------------|
| Android | `build.gradle` + `AndroidManifest.xml` in parent or sibling |
| Java (Gradle) | `build.gradle` or `build.gradle.kts` |
| Kotlin | `build.gradle.kts` |
| Flutter | `pubspec.yaml` |
| Remix | `remix.config.js`, `package.json` with `"remix"` dep |

### `.gradle/`

| Ecosystem | Marker files |
|-----------|--------------|
| Android | `AndroidManifest.xml`, `build.gradle` + `android {}` block |
| Java (Gradle) | `build.gradle` (no Android manifest) |
| Kotlin | `build.gradle.kts` |

### `vendor/`

| Ecosystem | Marker files |
|-----------|--------------|
| Go | `go.mod`, `go.sum` |
| PHP | `composer.json`, `composer.lock` |
| Ruby | `Gemfile`, `Gemfile.lock` (but Ruby uses `vendor/bundle/` specifically) |
| Vendored Dependencies | catch-all if no other markers found |

### `_build/`

| Ecosystem | Marker files |
|-----------|--------------|
| Elixir | `mix.exs` |
| Erlang | `rebar.config`, `rebar3` |
| OCaml | `dune-project`, `*.opam` |
| Sphinx | `conf.py` with `sphinx` import |

### `deps/`

| Ecosystem | Marker files |
|-----------|--------------|
| Elixir | `mix.exs` |
| Erlang | `rebar.config` |
| Julia | `Project.toml`, `Manifest.toml` |

### `node_modules/`

| Ecosystem | Marker files |
|-----------|--------------|
| Node.js (plain) | `package.json` without framework-specific deps |
| React | `package.json` with `"react"` dependency |
| Vue | `package.json` with `"vue"` dependency |
| Angular | `angular.json` |
| Next.js | `next.config.js` / `next.config.ts` |
| Nuxt | `nuxt.config.js` / `nuxt.config.ts` |
| Svelte | `svelte.config.js` |
| Vite | `vite.config.js` / `vite.config.ts` |
| Webpack | `webpack.config.js` |
| Bun | `bun.lockb` |
| Remix | `remix.config.js` |

`node_modules/` is a special case: all JS/TS frameworks share it, and distinguishing between them is low value — they all use the same folder and the safe action (deleting it) is the same regardless. The framework label is cosmetic here.

### `dist/`

| Ecosystem | Marker files |
|-----------|--------------|
| Vite | `vite.config.*` |
| Webpack | `webpack.config.js` |

---

## Proposed Solution: Marker-File Confidence Scoring

### Core idea

For each discovered folder, instead of returning the first ecosystem that matches the folder name, look at the **parent directory** (the project root) for well-known marker files. Each ecosystem defines a set of marker files; the ecosystem whose markers are present wins. If multiple ecosystems' markers are present, return all of them as a combined label.

### Data model changes

Add an optional `markers` array to the ecosystem JSON schema:

```json
{
  "name": "Rust",
  "local": ["target/"],
  "global": ["~/.cargo/registry/", "~/.cargo/git/"],
  "markers": ["Cargo.toml"]
}
```

```json
{
  "name": "Clojure",
  "local": ["target/", ".cpcache/"],
  "global": ["~/.clojure/"],
  "markers": ["project.clj", "deps.edn", "build.boot"]
}
```

`markers` is optional — ecosystems with unambiguous folder names (e.g. `node_modules/` for plain Node.js, `.dart_tool/` for Dart) don't need it.

### Detection algorithm

```
For each discovered folder F at path P:
  candidate_ecosystems = all ecosystems whose local patterns match basename(P)

  if len(candidate_ecosystems) == 1:
      → use that ecosystem, confidence = Certain

  else:
      project_root = parent directory of P
      confirmed = []
      for each candidate C:
          if any of C.markers exist in project_root:
              confirmed.append(C)

      if len(confirmed) == 1:
          → use confirmed[0], confidence = Confirmed
      elif len(confirmed) > 1:
          → use "Ecosystem1 / Ecosystem2", confidence = Ambiguous
      else:
          → use "Unknown (Ecosystem1 / Ecosystem2)", confidence = Undetected
```

### Display in TUI

The ecosystem column in the list widget already shows a string. Changes needed:

| Confidence | Display | Example |
|------------|---------|---------|
| Certain | Name as-is | `Rust` |
| Confirmed | Name as-is | `Rust` |
| Ambiguous | Slash-separated | `Rust / Java` |
| Undetected | Question mark prefix | `? target/` |

Ambiguous and Undetected rows could be styled with a dim colour to communicate lower certainty without adding noise.

### Marker file registry (proposed additions to each JSON)

| Ecosystem | Markers |
|-----------|---------|
| Rust | `Cargo.toml` |
| Java (Maven) | `pom.xml` |
| Clojure | `project.clj`, `deps.edn`, `build.boot` |
| Scala | `build.sbt`, `build.sc` |
| Android | `AndroidManifest.xml` |
| Java (Gradle) | `build.gradle`, `build.gradle.kts`, `settings.gradle` |
| Kotlin | `build.gradle.kts`, `settings.gradle.kts` |
| Flutter | `pubspec.yaml` |
| Go | `go.mod` |
| PHP | `composer.json` |
| Ruby | `Gemfile` |
| Elixir | `mix.exs` |
| Erlang | `rebar.config` |
| OCaml | `dune-project` |
| Sphinx | `conf.py` |
| Julia | `Project.toml` |
| Vite | `vite.config.js`, `vite.config.ts` |
| Webpack | `webpack.config.js` |
| Angular | `angular.json` |
| Next.js | `next.config.js`, `next.config.ts` |
| Nuxt | `nuxt.config.js`, `nuxt.config.ts` |
| Svelte | `svelte.config.js` |

---

## Implementation Plan

### Phase 1 — JSON schema (non-breaking)

1. Add `"markers": []` to every ecosystem JSON that has ambiguous folder names.
2. Update `Ecosystem` struct in `src/config/ecosystem.rs` to deserialize `markers` (default empty vec — backward compatible).
3. No behavior change yet.

### Phase 2 — Confidence scoring in scanner

1. In `src/scanner/dir.rs`, after finding candidate ecosystems for a folder, call a new `resolve_ecosystem(candidates, project_root) -> ResolvedEcosystem` function.
2. `ResolvedEcosystem` carries a display name and a `Confidence` enum.
3. Update `DiscoveredFolder` to store `confidence: Confidence` alongside `ecosystem: String`.

### Phase 3 — TUI display

1. Update the list widget's ecosystem cell to render `Ambiguous` rows in dim style.
2. No other UI changes needed — the existing string-based ecosystem column handles slash-joined names naturally.

### Phase 4 — Plain-text mode

1. Update `run_plain()` output to include confidence indicator (e.g. `[?]` prefix) for ambiguous entries.

---

## What This Does NOT Solve

- **Nested projects**: A monorepo may have a `target/` that is genuinely used by multiple ecosystems in different subdirectories. Marker-file lookup only checks the immediate parent.
- **Non-standard layouts**: Some projects put marker files in non-standard locations. Marker lookup will fall back to Undetected in these cases, which is safe (it shows the ambiguity rather than guessing wrong).
- **`node_modules/` framework attribution**: Distinguishing React vs Vue vs Angular within `node_modules/` is cosmetic since the delete action is identical. This is intentionally left as "Node.js" unless a strong framework marker is present (e.g. `angular.json`).

---

## Priority

The highest-value fix is the `target/` conflict (Rust vs Java vs Clojure vs Scala) since it affects the most common use case — Rust developers running everykill in their own projects. This should be implemented in Phase 1–2 first, before the full marker registry is complete.

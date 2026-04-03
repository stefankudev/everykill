# Ecosystems Reference

Complete list of supported ecosystems in `everykill`. Each ecosystem is defined in `ecosystems/<name>.json`.

## Schema

```json
{
  "name": "Display Name",
  "local": ["path/", "other/"],
  "global": ["~/.cache/path"]
}
```

- `local` - Per-project dependency folders (scanned from current directory)
- `global` - User-level caches (typically not deleted by default)

Path information is stored in the JSON files only.

## Supported Ecosystems

| File | Name |
|------|------|
| `angular.json` | Angular |
| `ansible.json` | Ansible |
| `android.json` | Android |
| `bazel.json` | Bazel |
| `buck.json` | Buck |
| `bun.json` | Bun |
| `cache.json` | Cache |
| `carthage.json` | Carthage |
| `clojure.json` | Clojure |
| `cmake.json` | CMake |
| `cocoapods.json` | CocoaPods |
| `crystal.json` | Crystal |
| `csharp.json` | C# / .NET |
| `dart.json` | Dart |
| `deno.json` | Deno |
| `elixir.json` | Elixir |
| `erlang.json` | Erlang |
| `fsharp.json` | F# |
| `flutter.json` | Flutter |
| `gatsby.json` | Gatsby |
| `godot.json` | Godot |
| `go.json` | Go |
| `haskell.json` | Haskell |
| `helm.json` | Helm |
| `hugo.json` | Hugo |
| `java-gradle.json` | Java (Gradle) |
| `java.json` | Java (Maven) |
| `jekyll.json` | Jekyll |
| `julia.json` | Julia |
| `kotlin.json` | Kotlin |
| `kustomize.json` | Kustomize |
| `lua.json` | Lua |
| `matlab.json` | MATLAB |
| `meson.json` | Meson |
| `nextjs.json` | Next.js |
| `nim.json` | Nim |
| `nix.json` | Nix |
| `nodejs.json` | Node.js |
| `nuxt.json` | Nuxt |
| `ocaml.json` | OCaml |
| `perl.json` | Perl |
| `php.json` | PHP |
| `pulumi.json` | Pulumi |
| `python.json` | Python |
| `r.json` | R |
| `react.json` | React |
| `remix.json` | Remix |
| `ruby.json` | Ruby |
| `rust.json` | Rust |
| `scala.json` | Scala |
| `sphinx.json` | Sphinx |
| `svelte.json` | Svelte |
| `swift.json` | Swift |
| `terraform.json` | Terraform |
| `unity.json` | Unity |
| `unreal.json` | Unreal Engine |
| `v.json` | V |
| `vendored.json` | Vendored Dependencies |
| `vite.json` | Vite |
| `vue.json` | Vue |
| `webpack.json` | Webpack |
| `zig.json` | Zig |

## Adding a New Ecosystem

1. Create `ecosystems/<name>.json` with the schema above
2. Add `name`, `local`, and `global` arrays
3. Use `kebab-case` for the filename
4. Commit with: `git commit -m "feat(ecosystems): add <name> support"`

## Notes

- Frontend frameworks (React, Vue, etc.) primarily use `node_modules/` which is shared with Node.js ecosystem
- Android shares `.gradle/` and `build/` paths with Java Gradle
- Flutter shares `.dart_tool/` with Dart

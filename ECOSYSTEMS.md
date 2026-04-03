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

## Supported Ecosystems

| File | Name | Local Paths | Global Paths |
|------|------|-------------|--------------|
| `nodejs.json` | Node.js | `node_modules/` | `~/.npm/` |
| `bun.json` | Bun | `node_modules/`, `.bun-install/` | `~/.bun/install-cache/` |
| `deno.json` | Deno | - | `~/.deno/` |
| `python.json` | Python | `venv/`, `.venv/`, `env/`, `__pycache__/` | `~/.local/lib/python*/site-packages/` |
| `ruby.json` | Ruby | `vendor/bundle/`, `.bundle/` | `~/.gem/ruby/*/cache/` |
| `php.json` | PHP | `vendor/` | `~/.composer/` |
| `perl.json` | Perl | `local/lib/`, `blib/` | `~/.cpanm/` |
| `lua.json` | Lua | `lua_modules/`, `luarocks/` | `~/.luarocks/` |
| `rust.json` | Rust | `target/` | `~/.cargo/registry/`, `~/.cargo/git/` |
| `go.json` | Go | `vendor/` | `~/go/pkg/mod/` |
| `java.json` | Java (Maven) | `target/` | `~/.m2/repository/` |
| `java-gradle.json` | Java (Gradle) | `.gradle/`, `build/` | `~/.gradle/caches/` |
| `kotlin.json` | Kotlin | `.gradle/`, `build/` | `~/.gradle/` |
| `scala.json` | Scala | `target/`, `.ivy2/` | `~/.ivy2/cache/`, `~/.sbt/boot/` |
| `csharp.json` | C# / .NET | `bin/`, `obj/` | `~/.nuget/packages/` |
| `haskell.json` | Haskell | `dist-newstyle/`, `.stack-work/` | `~/.stack/`, `~/.cabal/` |
| `swift.json` | Swift | `.build/` | `~/Library/Caches/org.swift.swiftpm/` |
| `dart.json` | Dart | `.dart_tool/`, `packages/` | `~/.pub-cache/` |
| `flutter.json` | Flutter | `.dart_tool/`, `.pub-cache/`, `build/` | `~/.pub-cache/` |
| `zig.json` | Zig | `zig-cache/`, `zig-out/` | - |
| `v.json` | V | `vlib/` | `~/.vmodules/` |
| `elixir.json` | Elixir | `_build/`, `deps/` | `~/.hex/` |
| `erlang.json` | Erlang | `_build/`, `deps/` | `~/.erlang.mk/` |
| `clojure.json` | Clojure | `target/`, `.cpcache/` | `~/.clojure/` |
| `ocaml.json` | OCaml | `_opam/`, `_build/` | `~/.opam/` |
| `fsharp.json` | F# | `bin/`, `obj/` | `~/.nuget/packages/` |
| `nim.json` | Nim | `nimcache/` | `~/.nimble/pkgs/` |
| `crystal.json` | Crystal | `lib/`, `shards/` | `~/.cache/shards/` |
| `r.json` | R | `renv/library/`, `.Rproj.user/` | `~/R/library/` |
| `julia.json` | Julia | `deps/` | `~/.julia/packages/`, `~/.julia/artifacts/` |
| `matlab.json` | MATLAB | `codegen/`, `slprj/` | - |
| `react.json` | React | `node_modules/` | - |
| `vue.json` | Vue | `node_modules/` | - |
| `angular.json` | Angular | `node_modules/` | - |
| `svelte.json` | Svelte | `node_modules/` | - |
| `nextjs.json` | Next.js | `.next/`, `node_modules/` | - |
| `nuxt.json` | Nuxt | `.nuxt/`, `node_modules/` | - |
| `remix.json` | Remix | `node_modules/`, `build/` | - |
| `gatsby.json` | Gatsby | `.cache/`, `public/` | - |
| `vite.json` | Vite | `node_modules/`, `dist/` | - |
| `webpack.json` | Webpack | `node_modules/`, `dist/` | - |
| `hugo.json` | Hugo | `resources/_gen/`, `public/` | - |
| `jekyll.json` | Jekyll | `_site/` | - |
| `sphinx.json` | Sphinx | `_build/` | - |
| `pulumi.json` | Pulumi | `.pulumi/` | `~/.pulumi/` |
| `ansible.json` | Ansible | `roles/`, `collections/` | `~/.ansible/collections/` |
| `terraform.json` | Terraform | `.terraform/` | `~/.terraform.d/plugin-cache/` |
| `helm.json` | Helm | `charts/` | `~/.cache/helm/` |
| `kustomize.json` | Kustomize | `kustomize/` | - |
| `nix.json` | Nix | `result/` | `~/.nix-profile/` |
| `unity.json` | Unity | `Library/`, `Packages/`, `obj/`, `Temp/` | - |
| `unreal.json` | Unreal Engine | `Binaries/`, `Intermediate/`, `DerivedDataCache/` | - |
| `godot.json` | Godot | `.import/` | - |
| `bazel.json` | Bazel | `bazel-out/`, `bazel-bin/`, `bazel-testlogs/` | `~/.cache/bazel/` |
| `buck.json` | Buck | `buck-out/`, `gen/` | `~/.buckcache/` |
| `cmake.json` | CMake | `CMakeFiles/`, `cmake-files/`, `_deps/` | - |
| `meson.json` | Meson | `meson-out/` | - |
| `carthage.json` | Carthage | `Carthage/Build/` | `~/Library/Caches/carthage/` |
| `cocoapods.json` | CocoaPods | `Pods/` | `~/Library/Caches/CocoaPods/` |
| `android.json` | Android | `.gradle/`, `build/`, `app/build/` | `~/.android/` |
| `vendored.json` | Vendored Dependencies | `vendor/`, `third_party/` | - |
| `cache.json` | Cache | `.cache/`, `tmp/`, `temp/` | `~/.cache/` |

## Adding a New Ecosystem

1. Create `ecosystems/<name>.json` with the schema above
2. Add `name`, `local`, and `global` arrays
3. Use `kebab-case` for the filename
4. Commit with: `git commit -m "feat(ecosystems): add <name> support"`

## Notes

- Frontend frameworks (React, Vue, etc.) primarily use `node_modules/` which is shared with Node.js ecosystem
- Android shares `.gradle/` and `build/` paths with Java Gradle
- Flutter shares `.dart_tool/` with Dart
- Some ecosystems have empty `global` arrays when caches are rarely used

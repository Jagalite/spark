#!/usr/bin/env bash
# Source-only scaffolding helper. Does not build, fetch dependencies or publish.
set -euo pipefail

if [[ $# -ne 2 ]]; then
  printf 'Usage: bash fixtures/extension-starter/copy.sh DESTINATION package-name\n' >&2
  exit 2
fi
destination=$1
name=$2
if [[ ! $name =~ ^[a-z][a-z0-9]*(-[a-z0-9]+)*$ ]]; then
  printf 'Use a lowercase package name such as my-lantern (letters, digits, hyphens).\n' >&2
  exit 2
fi
# Do not create invalid Rust import identifiers or shadow existing dependencies.
case "$name" in
  as|async|await|break|const|continue|crate|dyn|else|enum|extern|false|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|self|static|struct|super|trait|true|type|unsafe|use|where|while|abstract|become|box|do|final|gen|macro|macro-rules|override|priv|try|typeof|unsized|virtual|yield|union|raw|safe|std|core|alloc|proc-macro|test|spark-runtime|ratatui-core)
    printf 'Choose a package name that is not a Rust keyword or an existing runtime dependency.\n' >&2
    exit 2
    ;;
esac
if [[ -e $destination || -L $destination ]]; then
  printf 'Destination already exists; nothing was copied: %s\n' "$destination" >&2
  exit 2
fi

starter=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
spark=$(cd -- "$starter/../.." && pwd)
if [[ ! -f $spark/crates/runtime/Cargo.toml ]]; then
  printf 'Run the helper from its original Spark checkout.\n' >&2
  exit 2
fi
if [[ $spark == *$'\n'* || $spark == *$'\r'* || $spark == *$'\t'* ]]; then
  printf 'Checkout paths containing control characters are unsupported.\n' >&2
  exit 2
fi
crate_name=${name//-/_}
# TOML basic strings need these escapes, even in paths containing spaces.
runtime_path=${spark//\\/\\\\}/crates/runtime
runtime_path=${runtime_path//\"/\\\"}

mkdir -- "$destination"
cp -R -- "$starter/src" "$starter/tests" "$destination/"
cp -- "$starter/LICENSE" "$starter/README.md" "$starter/preview.txt" \
  "$starter/rust-toolchain.toml" "$starter/.gitignore" "$destination/"
cat > "$destination/Cargo.toml" <<EOF
[package]
name = "$name"
version = "0.0.0"
edition = "2024"
rust-version = "1.90"
publish = false
license = "MIT"

# Remain a standalone workspace; do not add a package to Spark's workspace.
[workspace]

[dependencies]
spark-runtime = { version = "=0.1.0", path = "$runtime_path" }
ratatui-core = { version = "=0.1.2", default-features = false, features = ["std", "underline-color"] }

[lints.rust]
unsafe_code = "forbid"
EOF
sed "s/^name = \"spark-extension-starter\"$/name = \"$name\"/" \
  "$starter/Cargo.lock" > "$destination/Cargo.lock"
for file in "$destination/src/main.rs" "$destination/tests/public_contract.rs"; do
  sed "s/spark_extension_starter/$crate_name/g" "$file" > "$file.renamed"
  mv -- "$file.renamed" "$file"
done
printf 'Created %s (package %s)\n' "$destination" "$name"
printf 'Runtime path is pinned to this Spark checkout: %s/crates/runtime\n' "$spark"
printf 'Next: cd into the new directory and run cargo test --locked, then cargo run --locked\n'

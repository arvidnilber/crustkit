#!/usr/bin/env bash
set -euo pipefail

project_dir="${1:-$PWD}"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
script_crustkit_path="$(cd "$script_dir/.." && pwd)"

load_env_file() {
  local env_file="$1"
  [[ -f "$env_file" ]] || return 0

  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "$line" || "$line" =~ ^[[:space:]]*# ]] && continue
    [[ "$line" == *=* ]] || continue

    local key="${line%%=*}"
    local value="${line#*=}"
    key="$(printf '%s' "$key" | xargs)"
    value="$(printf '%s' "$value" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^"//' -e 's/"$//' -e "s/^'//" -e "s/'$//")"

    if [[ "$key" == "CRUSTKIT_PATH" && -z "${CRUSTKIT_PATH:-}" ]]; then
      CRUSTKIT_PATH="$value"
      export CRUSTKIT_PATH
    fi
  done < "$env_file"
}

load_env_file "$project_dir/.env"
load_env_file "$project_dir/.env.local"

crustkit_path="${CRUSTKIT_PATH:-$script_crustkit_path}"
crate_path="$crustkit_path/crates/crustkit"

if [[ ! -f "$crate_path/Cargo.toml" ]]; then
  printf 'crustkit crate not found at %s\n' "$crate_path" >&2
  printf 'set CRUSTKIT_PATH in %s/.env or %s/.env.local\n' "$project_dir" "$project_dir" >&2
  exit 1
fi

mkdir -p "$project_dir/.cargo"
cat > "$project_dir/.cargo/config.toml" <<EOF
[patch.crates-io]
crustkit = { path = "$crate_path" }
EOF

printf 'wrote %s\n' "$project_dir/.cargo/config.toml"
printf 'using local crustkit at %s\n' "$crate_path"

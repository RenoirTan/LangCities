#!/bin/bash

set -euo pipefail

DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

source "$DIR/lib.sh"

get_entity_dir() {
  # $1: service name
  local _service=${1,,}
  if [ $_service == 'auth' ]; then
    echo "/app/crates/langcities-auth/src/entity"
  elif [ $_service == 'dc' ]; then
    echo "/app/crates/langcities-dc/src/entity"
  else
    echo "unknown service: $1"
    return 1
  fi
}

main() {
  # $1: auth/dc
  # ensure_sea_orm_cli
  if [ $# -ne 1 ]; then
    echo "Usage: ./sea-codegen.sh <auth/dc>"
    return 1
  fi
  local _db_url="$(get_db_url $1)"
  local _entity_dir="$(get_entity_dir $1)"
  sea-orm-cli generate entity \
    -u $_db_url \
    -o $_entity_dir \
    --entity-format dense \
    --ignore-tables tower_sessions
}

main $@

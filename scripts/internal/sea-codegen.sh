#!/bin/bash

set -euo pipefail

ensure_sea_orm_cli() {
  if ! command -v sea-orm-cli; then
    echo "could not find sea-orm-cli, installing using cargo..."
    cargo install sea-orm-cli@^2.0
    if ! command -v sea-orm-cli; then
      echo "failed to verify installation, aborting..."
      return 1
    fi
  fi
}

get_first_good() {
  local _good=""
  while [ $# -ge 1 ]; do
    if [ ! -z "$1" ]; then
      if [ -z "$_good" ]; then
        _good="$1"
      else
        return 1
      fi
    fi
    shift
  done
  if [ -z "$_good" ]; then
    return 1
  else
    echo "$_good"
  fi
}

get_db_url() {
  # $1: service name
  local _service=${1,,}
  local _url=''
  if [ $_service == 'auth' ]; then
    _url="$(get_first_good ${LCAUTH_DB_URL:-} ${LCAUTH_DATABASE_URL:-})"
  elif [ $_service == 'dc' ]; then
    _url="$(get_first_good ${LCDC_DB_URL:-} ${LCDC_DATABASE_URL:-})"
  else
    echo "unknown service: $1"
    return 1
  fi
  echo "$_url"
}

get_entity_dir() {
  # $1: service name
  local _repo_root="$(dirname $0)/.."
  local _service=${1,,}
  if [ $_service == 'auth' ]; then
    echo "$_repo_root/crates/langcities-auth/src/entity"
  elif [ $_service == 'dc' ]; then
    echo "$_repo_root/crates/langcities-dc/src/entity"
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

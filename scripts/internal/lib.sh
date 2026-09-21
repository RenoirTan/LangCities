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

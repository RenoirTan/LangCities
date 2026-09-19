# LangCities

LangCities is a constructed language management system that aims to eventually encompass a full suite of tools to support conlang creators.

## Build

```bash
cargo build --workspace --features sqlite,postgres,mysql
```

### SeaORM Codegen

Run the following commands to generate entity files for langcities' microservices.

```bash
./scripts/db-codegen.sh sqlite dev auth
./scripts/db-codegen.sh sqlite dev dc
```

Internally, the script indirectly runs something like this:

```bash
sea-orm-cli generate entity -u $LCAUTH_DB_URL -o crates/langcities-auth/src/entity --entity-format dense
```

LangCities uses SeaORM to manage the data stored in databases. It uses the new dense entity format available in SeaORM v2 in order to make some cool new features available for me. Therefore, specifying `--entity-format dense` is mandatory. Otherwise, you may find that certain features like find by unique key will become unavailable and trigger a compile error.

## Docker

### dev

```bash
mkdir -p data/cargo/{target,git,registry}
./scripts/compose.sh build
./scripts/compose.sh up --remove-orphans
```

# LangCities

LangCities is a constructed language management system that aims to eventually encompass a full suite of tools to support conlang creators.

## Build

```bash
cargo build --workspace --features sqlite,postgres,mysql
```

### SeaORM Codegen

LangCities uses SeaORM to manage the data stored in databases. It uses the new dense entity format available in SeaORM v2 in order to make some cool new features available for me. Therefore, specifying `--entity-format dense` is mandatory. Otherwise, you may find that certain features like find by unique key will become unavailable and trigger a compile error.

```bash
sea-orm-cli generate entity -u $LCAUTH_DB_URL -o crates/langcities-auth/src/entity --entity-format dense
```

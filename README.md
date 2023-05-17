rs-schema-registry
---

Run before compiling

```
cargo install sqlx-cli
cargo sqlx migrate run
```


Test curl commands

```
curl -v  -X POST -d '{"schema": "{\"type\": \"string\"}"}' -H "Content-Type: application/json" localhost:8888/subjects/test
curl -v  -X POST -d '{"schema": "{\"type\": \"string\"}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions
```
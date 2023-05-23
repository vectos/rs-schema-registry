rs-schema-registry
---

The main purpose of Schema Registry is to enforce data compatibility and enable schema evolution in a decoupled manner. When data is produced or consumed by various services, it is important to have a shared understanding of the structure and format of the data. Schema Registry provides a way to define, register, and version schemas, ensuring that all data producers and consumers adhere to the agreed-upon schema.

### Scope

The scope of this project is to explore Rust/Axum/SQLX. Therefore it comes close to the Confluent spec but has limitations like:

- Not all endpoints are implemented
- It only supports _Avro_
- It is barely tested, only by hand

### Run before compiling

```
cargo install sqlx-cli
cargo sqlx migrate run --database-url postgres://postgres:postgres@localhost:5432/postgres
cargo sqlx prepare  --database-url postgres://postgres:postgres@localhost:5432/postgres
```

This will migrate the database before compiling, as the `sqlx::query!` and `sqlx::query_as!` will test the queries _while_ compiling.

### Test curl commands

```
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test

-- insert new schema (1) - success
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions

-- insert new schema (2) - failure
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"},{\"name\":\"Wage\",\"type\":\"int\"}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions

-- insert new schema (2) - success
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"},{\"name\":\"Wage\",\"default\":1,\"type\":[\"int\", \"null\"]}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions

-- check compatibility
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"},{\"name\":\"Wage\",\"default\":1,\"type\":\"int\"}]}"}' -H "Content-Type: application/json" localhost:8888/compatibility/subjects/test/versions/1

-- set compatibility globally
curl -X PUT -H "Content-Type: application/json" -d '{"compatibility": "BACKWARD"}' http://localhost:8888/config

-- set compatibilityy per subject
curl -X PUT -H "Content-Type: application/json" -d '{"compatibility": "BACKWARD_TRANSITIVE"}' http://localhost:8888/config/test

```

### Reference

- [Docker hub](https://hub.docker.com/r/markdj/rs-schema-registry/tags)
- [Confluent API](https://docs.confluent.io/platform/current/schema-registry/develop/api.html)
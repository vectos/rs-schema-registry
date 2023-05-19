rs-schema-registry
---

Run before compiling

```
cargo install sqlx-cli
cargo sqlx migrate run --database-url postgres://postgres:postgres@localhost:5432/postgres
```


Test curl commands

```
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"},{\"name\":\"Wage\",\"type\":\"int\"}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"},{\"name\":\"Wage\",\"default\":1,\"type\":[\"int\", \"null\"]}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions

curl -X PUT -H "Content-Type: application/json" -d '{"compatibility": "BACKWARD"}' http://localhost:8888/config
curl -X PUT -H "Content-Type: application/json" -d '{"compatibility": "BACKWARD_TRANSITIVE"}' http://localhost:8888/config/test

```
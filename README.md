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

-- insert new schema (1) - success
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions

-- insert new schema (2) - failure
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"},{\"name\":\"Wage\",\"type\":\"int\"}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions

-- insert new schema (2) - success
curl -v  -X POST -d '{"schema": "{\"type\":\"record\",\"namespace\":\"Tutorialspoint\",\"name\":\"Employee\",\"fields\":[{\"name\":\"Name\",\"type\":\"string\"},{\"name\":\"Age\",\"type\":\"int\"},{\"name\":\"Wage\",\"default\":1,\"type\":[\"int\", \"null\"]}]}"}' -H "Content-Type: application/json" localhost:8888/subjects/test/versions

-- check compatibility
curl -v  -X POST -d '{"type":"record","namespace":"Tutorialspoint","name":"Employee","fields":[{"name":"Name","type":"string"},{"name":"Age","type":"int"},{"name":"Wage","type":"int"}]}' -H "Content-Type: application/json" localhost:8888/compatibility/subjects/test/versions/1

-- set compatibility globally
curl -X PUT -H "Content-Type: application/json" -d '{"compatibility": "BACKWARD"}' http://localhost:8888/config

-- set compatibilityy per subject
curl -X PUT -H "Content-Type: application/json" -d '{"compatibility": "BACKWARD_TRANSITIVE"}' http://localhost:8888/config/test

```

## TODO

- [ ] DELETE endpoints
- [ ] Test setup
- [ ] Segregate service / data layer
- [ ] Is the check compatibility also transitive? In other words should we fetch all version X <= version and check for transitive property
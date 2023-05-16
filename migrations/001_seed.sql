CREATE SEQUENCE subjects_id_seq;
CREATE TABLE subjects (
  id BIGINT PRIMARY KEY DEFAULT nextval('subjects_id_seq'::regclass),
  name TEXT NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

CREATE UNIQUE INDEX index_subjects_on_name ON subjects(name);

CREATE SEQUENCE schemas_id_seq;
CREATE TABLE schemas (
  id BIGINT PRIMARY KEY DEFAULT nextval('schemas_id_seq'::regclass),
  fingerprint CHARACTER VARYING NOT NULL,
  json TEXT NOT NULL,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  fingerprint2 CHARACTER VARYING
);

CREATE UNIQUE INDEX index_schemas_on_fingerprint ON schemas(fingerprint);
CREATE UNIQUE INDEX index_schemas_on_fingerprint2 ON schemas(fingerprint2);

CREATE SEQUENCE schema_versions_id_seq;
CREATE TABLE schema_versions (
  id BIGINT PRIMARY KEY DEFAULT nextval('schema_versions_id_seq'::regclass),
  version INTEGER DEFAULT 1,
  subject_id BIGINT NOT NULL references subjects(id),
  schema_id BIGINT NOT NULL references schemas(id)
);

CREATE UNIQUE INDEX index_schema_versions_on_subject_id_and_version ON schema_versions(subject_id, version);

CREATE SEQUENCE configs_id_seq;
CREATE TABLE configs (
  id BIGINT PRIMARY KEY DEFAULT nextval('configs_id_seq'::regclass),
  compatibility CHARACTER VARYING,
  created_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  updated_at TIMESTAMP WITHOUT TIME ZONE NOT NULL,
  subject_id BIGINT NOT NULL references subjects(id)
);

CREATE UNIQUE INDEX index_configs_on_subject_id ON configs(subject_id);
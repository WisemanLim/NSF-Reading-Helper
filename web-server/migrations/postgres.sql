-- NSF Reading Helper — PostgreSQL schema.
-- Mirrors the SQLite schema in src-tauri/src/core/db.rs but uses native types
-- and pg_trgm + fuzzystrmatch for fuzzy search instead of FTS5 + custom
-- Levenshtein function.

create extension if not exists pg_trgm;
create extension if not exists fuzzystrmatch;

create table if not exists domains (
    id          bigserial primary key,
    code        varchar(50) unique not null,
    name_ko     varchar(100) not null,
    name_en     varchar(100),
    description text,
    created_at  bigint default (extract(epoch from now()) * 1000)::bigint
);

create table if not exists symbols (
    id                bigserial primary key,
    unicode_char      varchar(16),
    latex_command     varchar(128),
    mathml_tag        varchar(128),
    canonical_name_ko varchar(200) not null,
    canonical_name_en varchar(200),
    reading_ko        varchar(200),
    reading_en        varchar(200),
    short_meaning_ko  text not null,
    short_meaning_en  text,
    symbol_type       varchar(50) not null,
    difficulty_level  smallint default 1,
    is_operator       boolean default false,
    is_relation       boolean default false,
    is_delimiter      boolean default false,
    is_greek          boolean default false,
    is_constant       boolean default false,
    is_function       boolean default false,
    notes             text,
    created_at        bigint default (extract(epoch from now()) * 1000)::bigint,
    updated_at        bigint default (extract(epoch from now()) * 1000)::bigint
);

create table if not exists symbol_aliases (
    id         bigserial primary key,
    symbol_id  bigint not null references symbols(id) on delete cascade,
    alias_text varchar(200) not null,
    alias_type varchar(50) not null,
    locale     varchar(10) default 'ko',
    unique (symbol_id, alias_text, alias_type, locale)
);

create table if not exists symbol_domain_map (
    id                    bigserial primary key,
    symbol_id             bigint not null references symbols(id) on delete cascade,
    domain_id             bigint not null references domains(id) on delete cascade,
    priority              smallint default 1,
    meaning_in_domain_ko  text,
    example_usage         text,
    unique (symbol_id, domain_id)
);

create table if not exists symbol_examples (
    id                       bigserial primary key,
    symbol_id                bigint not null references symbols(id) on delete cascade,
    domain_id                bigint references domains(id) on delete set null,
    example_latex            text not null,
    example_reading_ko       text,
    example_explanation_ko   text,
    difficulty_level         smallint default 1,
    created_at               bigint default (extract(epoch from now()) * 1000)::bigint
);

create table if not exists expressions (
    id                bigserial primary key,
    input_format      varchar(20) not null,
    raw_input         text not null,
    normalized_latex  text,
    normalized_mathml text,
    rendered_ok       boolean default false,
    speech_text_ko    text,
    speech_text_en    text,
    created_at        bigint default (extract(epoch from now()) * 1000)::bigint
);

create table if not exists favorites (
    id         bigserial primary key,
    symbol_id  bigint not null references symbols(id) on delete cascade,
    created_at bigint default (extract(epoch from now()) * 1000)::bigint,
    unique (symbol_id)
);

create table if not exists notes (
    id            bigserial primary key,
    expression_id bigint references expressions(id) on delete set null,
    symbol_id     bigint references symbols(id) on delete set null,
    body_md       text not null,
    tags          text default '[]',
    created_at    bigint default (extract(epoch from now()) * 1000)::bigint,
    updated_at    bigint default (extract(epoch from now()) * 1000)::bigint
);

create table if not exists app_settings (
    key   text primary key,
    value text not null
);

create index if not exists idx_symbols_unicode on symbols(unicode_char);
create index if not exists idx_symbols_latex on symbols(latex_command);
create index if not exists idx_symbols_type on symbols(symbol_type);
create index if not exists idx_aliases_text on symbol_aliases(alias_text);

-- Trigram indexes for fuzzy match (replaces SQLite FTS5 trigram tokenizer).
create index if not exists idx_symbols_ko_trgm
    on symbols using gin (canonical_name_ko gin_trgm_ops);
create index if not exists idx_symbols_en_trgm
    on symbols using gin (canonical_name_en gin_trgm_ops);
create index if not exists idx_aliases_trgm
    on symbol_aliases using gin (alias_text gin_trgm_ops);

-- fuzzystrmatch.levenshtein is provided by the extension — no custom function needed.

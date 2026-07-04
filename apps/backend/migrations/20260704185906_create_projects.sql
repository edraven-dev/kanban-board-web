create table projects (
    id         uuid primary key,
    name       text not null,
    position   integer not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

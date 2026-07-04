create table cards (
    id          uuid primary key,
    column_id   uuid not null references columns (id) on delete cascade,
    title       text not null,
    description text not null default '',
    position    integer not null,
    created_at  timestamptz not null default now(),
    updated_at  timestamptz not null default now()
);

create index cards_column_position_idx on cards (column_id, position);

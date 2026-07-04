create table columns (
    id         uuid primary key,
    board_id   uuid not null references boards (id) on delete cascade,
    name       text not null,
    position   integer not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index columns_board_position_idx on columns (board_id, position);

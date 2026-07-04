create table boards (
    id         uuid primary key,
    project_id uuid not null references projects (id) on delete cascade,
    name       text not null,
    position   integer not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index boards_project_position_idx on boards (project_id, position);

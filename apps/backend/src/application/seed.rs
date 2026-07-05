use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::application::board_service::MAX_BOARDS_PER_PROJECT;
use crate::domain::board::Board;
use crate::domain::card::Card;
use crate::domain::column::Column;
use crate::domain::description::Description;
use crate::domain::ids::{BoardId, CardId, ColumnId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::ports::{BoardRepository, ProjectRepository, RepoResult};
use crate::domain::position::Position;
use crate::domain::project::Project;
use crate::domain::title::Title;

/// Stable id for the demo project so re-seeding replaces it instead of duplicating.
const DEMO_PROJECT_ID: Uuid = Uuid::from_u128(0x0de0_0000_0000_4000_8000_0000_0000_0001);

/// Row counts written by [`seed`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedSummary {
    pub projects: usize,
    pub boards: usize,
    pub columns: usize,
    pub cards: usize,
}

type ColumnSpec<'a> = (&'a str, &'a [(&'a str, &'a str)]);

/// Populates a demo project (two boards, six columns, ten cards with staggered
/// `created_at`s) for local development. Idempotent: any prior demo project is
/// removed by its fixed id first, so re-running converges to the same state.
pub async fn seed(
    projects: &dyn ProjectRepository,
    boards: &dyn BoardRepository,
) -> RepoResult<SeedSummary> {
    let project_id = ProjectId::from_uuid(DEMO_PROJECT_ID);
    projects.delete(project_id).await?;

    let project = Project::from_parts(
        project_id,
        name("Demo Project"),
        pos(0),
        Utc::now(),
        Utc::now(),
    );
    projects.insert(&project).await?;

    let demo = demo_boards(project_id);
    let columns = demo.iter().map(|b| b.columns().len()).sum();
    let cards = demo
        .iter()
        .flat_map(Board::columns)
        .map(|c| c.cards().len())
        .sum();
    for board in &demo {
        boards
            .insert_within_limit(board, MAX_BOARDS_PER_PROJECT)
            .await?;
        boards.save(board).await?;
    }

    Ok(SeedSummary {
        projects: 1,
        boards: demo.len(),
        columns,
        cards,
    })
}

fn demo_boards(project_id: ProjectId) -> Vec<Board> {
    let roadmap: &[ColumnSpec] = &[
        (
            "Backlog",
            &[
                ("Dark mode", "Respect the OS theme with a manual toggle."),
                ("Bulk card actions", ""),
                ("Keyboard shortcuts", ""),
            ],
        ),
        (
            "In Progress",
            &[
                ("Drag-and-drop polish", "Smooth cross-column moves."),
                ("Board sharing", ""),
            ],
        ),
        (
            "Done",
            &[("Card details modal", ""), ("Column reordering", "")],
        ),
    ];
    let bugs: &[ColumnSpec] = &[
        ("Reported", &[("Cards flicker on move", "Only in Safari.")]),
        (
            "Investigating",
            &[("Slow board load", "Boards over 50 cards take >2s.")],
        ),
        ("Resolved", &[("Login redirect loop", "")]),
    ];

    let now = Utc::now();
    vec![
        board(project_id, "Product Roadmap", 0, roadmap, now),
        board(project_id, "Bug Tracker", 1, bugs, now),
    ]
}

fn board(
    project_id: ProjectId,
    name_: &str,
    position: i32,
    columns_spec: &[ColumnSpec],
    now: DateTime<Utc>,
) -> Board {
    let board_id = BoardId::new();
    // A rolling index spreads card timestamps into the past across the whole board.
    let mut age = 0usize;
    let columns = columns_spec
        .iter()
        .enumerate()
        .map(|(index, (col_name, cards))| {
            column(board_id, col_name, index as i32, cards, now, &mut age)
        })
        .collect();
    Board::from_parts(
        board_id,
        project_id,
        name(name_),
        pos(position),
        now,
        now,
        columns,
    )
}

fn column(
    board_id: BoardId,
    name_: &str,
    position: i32,
    cards_spec: &[(&str, &str)],
    now: DateTime<Utc>,
    age: &mut usize,
) -> Column {
    let column_id = ColumnId::new();
    let cards = cards_spec
        .iter()
        .enumerate()
        .map(|(index, (card_title, card_desc))| {
            let created = now - card_age(*age);
            *age += 1;
            Card::from_parts(
                CardId::new(),
                column_id,
                title(card_title),
                desc(card_desc),
                pos(index as i32),
                created,
                created,
            )
        })
        .collect();
    Column::from_parts(
        column_id,
        board_id,
        name(name_),
        pos(position),
        now,
        now,
        cards,
    )
}

fn card_age(index: usize) -> Duration {
    let offsets = [
        Duration::minutes(6),
        Duration::minutes(52),
        Duration::hours(4),
        Duration::hours(27),
        Duration::days(3),
        Duration::days(8),
    ];
    offsets[index % offsets.len()]
}

fn name(value: &str) -> EntityName {
    EntityName::new(value).expect("valid seed name")
}

fn title(value: &str) -> Title {
    Title::new(value).expect("valid seed title")
}

fn desc(value: &str) -> Description {
    Description::new(value).expect("valid seed description")
}

fn pos(value: i32) -> Position {
    Position::new(value).expect("valid seed position")
}

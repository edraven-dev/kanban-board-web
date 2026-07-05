use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::application::board_service::MAX_BOARDS_PER_PROJECT;
use crate::domain::board::Board;
use crate::domain::description::Description;
use crate::domain::ids::{ColumnId, ProjectId};
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

/// Populates a demo project (two boards, six columns, ten cards with staggered
/// `created_at`s) for local development. Idempotent: any prior demo project is
/// removed by its fixed id first, so re-running converges to the same state.
pub async fn seed(
    projects: &dyn ProjectRepository,
    boards: &dyn BoardRepository,
) -> RepoResult<SeedSummary> {
    let project_id = ProjectId::from_uuid(DEMO_PROJECT_ID);
    projects.delete(project_id).await?;

    let mut project = Project::new(name("Demo Project"), pos(0));
    project.id = project_id;
    projects.insert(&project).await?;

    let demo = demo_boards(project_id);
    let columns = demo.iter().map(|b| b.columns.len()).sum();
    let cards = demo
        .iter()
        .flat_map(|b| &b.columns)
        .map(|c| c.cards.len())
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
    let mut roadmap = Board::new(project_id, name("Product Roadmap"), pos(0));
    let backlog = add_column(&mut roadmap, "Backlog");
    let doing = add_column(&mut roadmap, "In Progress");
    let done = add_column(&mut roadmap, "Done");
    add_card(
        &mut roadmap,
        backlog,
        "Dark mode",
        "Respect the OS theme with a manual toggle.",
    );
    add_card(&mut roadmap, backlog, "Bulk card actions", "");
    add_card(&mut roadmap, backlog, "Keyboard shortcuts", "");
    add_card(
        &mut roadmap,
        doing,
        "Drag-and-drop polish",
        "Smooth cross-column moves.",
    );
    add_card(&mut roadmap, doing, "Board sharing", "");
    add_card(&mut roadmap, done, "Card details modal", "");
    add_card(&mut roadmap, done, "Column reordering", "");

    let mut bugs = Board::new(project_id, name("Bug Tracker"), pos(1));
    let reported = add_column(&mut bugs, "Reported");
    let investigating = add_column(&mut bugs, "Investigating");
    let resolved = add_column(&mut bugs, "Resolved");
    add_card(
        &mut bugs,
        reported,
        "Cards flicker on move",
        "Only in Safari.",
    );
    add_card(
        &mut bugs,
        investigating,
        "Slow board load",
        "Boards over 50 cards take >2s.",
    );
    add_card(&mut bugs, resolved, "Login redirect loop", "");

    let now = Utc::now();
    let mut boards = vec![roadmap, bugs];
    for board in &mut boards {
        stagger_created_at(board, now);
    }
    boards
}

// Spreads card timestamps into the past so the UI shows a range of "created N ago".
fn stagger_created_at(board: &mut Board, now: DateTime<Utc>) {
    let offsets = [
        Duration::minutes(6),
        Duration::minutes(52),
        Duration::hours(4),
        Duration::hours(27),
        Duration::days(3),
        Duration::days(8),
    ];
    for (index, card) in board
        .columns
        .iter_mut()
        .flat_map(|c| c.cards.iter_mut())
        .enumerate()
    {
        let at = now - offsets[index % offsets.len()];
        card.created_at = at;
        card.updated_at = at;
    }
}

fn add_column(board: &mut Board, name_: &str) -> ColumnId {
    board
        .add_column(name(name_))
        .expect("within column limit")
        .id
}

fn add_card(board: &mut Board, column: ColumnId, title_: &str, description: &str) {
    board
        .add_card(column, title(title_), desc(description))
        .expect("seeded column exists");
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

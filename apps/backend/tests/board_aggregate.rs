//! Persistence tests for the Board aggregate repository.

mod common;

use backend::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::domain::board::Board;
use backend::domain::description::Description;
use backend::domain::ids::{BoardId, CardId, ColumnId};
use backend::domain::name::EntityName;
use backend::domain::ports::{BoardRepository, ProjectRepository};
use backend::domain::position::Position;
use backend::domain::project::Project;
use backend::domain::title::Title;
use common::db_test;
use uuid::Uuid;

/// Seeds a board with columns `To Do` {A, B} and `Done` {}.
async fn seeded(pool: &sqlx::PgPool) -> (PgBoardRepo, BoardId, ColumnId, ColumnId, CardId, CardId) {
    let project = Project::new(EntityName::new("P").unwrap(), Position::new(0).unwrap());
    PgProjectRepo::new(pool.clone())
        .insert(&project)
        .await
        .unwrap();
    let board = Board::new(
        project.id,
        EntityName::new("B").unwrap(),
        Position::new(0).unwrap(),
    );
    let repo = PgBoardRepo::new(pool.clone());
    repo.insert_within_limit(&board, 99).await.unwrap();

    let mut aggregate = repo.load(board.id).await.unwrap().unwrap();
    let todo = aggregate
        .add_column(EntityName::new("To Do").unwrap())
        .unwrap()
        .id;
    let done = aggregate
        .add_column(EntityName::new("Done").unwrap())
        .unwrap()
        .id;
    let a = aggregate
        .add_card(todo, Title::new("A").unwrap(), Description::default())
        .unwrap()
        .id;
    let b = aggregate
        .add_card(todo, Title::new("B").unwrap(), Description::default())
        .unwrap()
        .id;
    repo.save(&aggregate).await.unwrap();
    (repo, board.id, todo, done, a, b)
}

db_test! {
    async fn save_round_trips_mutations_and_prunes_removed_cards(pool: PgPool) {
        let (repo, board, todo, done, a, b) = seeded(&pool).await;

        let mut aggregate = repo.load(board).await.unwrap().unwrap();
        aggregate.rename_column(todo, EntityName::new("Doing").unwrap()).unwrap();
        aggregate.move_card(a, done, 0).unwrap();
        aggregate.remove_card(b).unwrap();
        repo.save(&aggregate).await.unwrap();

        let reloaded = repo.load(board).await.unwrap().unwrap();
        let doing = reloaded.column(todo).unwrap();
        assert_eq!(doing.name.as_str(), "Doing");
        assert!(doing.cards.is_empty());
        let done = reloaded.column(done).unwrap();
        let titles: Vec<&str> = done.cards.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(titles, ["A"]);
        assert_eq!(done.cards[0].column_id, done.id);
    }
}

db_test! {
    async fn save_prunes_a_removed_column_and_its_cards(pool: PgPool) {
        let (repo, board, todo, _done, a, _b) = seeded(&pool).await;

        let mut aggregate = repo.load(board).await.unwrap().unwrap();
        aggregate.remove_column(todo).unwrap();
        repo.save(&aggregate).await.unwrap();

        let reloaded = repo.load(board).await.unwrap().unwrap();
        assert_eq!(reloaded.columns.len(), 1);
        assert!(reloaded.column(todo).is_none());
        assert!(repo.load_by_card(a).await.unwrap().is_none());
    }
}

db_test! {
    async fn load_by_column_and_card_resolve_the_owning_board(pool: PgPool) {
        let (repo, board, todo, _done, a, _b) = seeded(&pool).await;

        assert_eq!(repo.load_by_column(todo).await.unwrap().unwrap().id, board);
        assert_eq!(repo.load_by_card(a).await.unwrap().unwrap().id, board);
        assert!(repo.load_by_column(ColumnId::new()).await.unwrap().is_none());
        assert!(repo.load_by_card(CardId::new()).await.unwrap().is_none());
    }
}

db_test! {
    async fn deleting_a_board_cascades_to_its_columns_and_cards(pool: PgPool) {
        let (repo, board, todo, _done, a, _b) = seeded(&pool).await;

        repo.delete(board).await.unwrap();
        assert!(repo.load(board).await.unwrap().is_none());
        assert!(repo.load_by_column(todo).await.unwrap().is_none());
        assert!(repo.load_by_card(a).await.unwrap().is_none());
    }
}

db_test! {
    async fn a_column_row_violating_the_name_rule_fails_to_map(pool: PgPool) {
        let (repo, board, _todo, _done, _a, _b) = seeded(&pool).await;
        sqlx::query("INSERT INTO columns (id, board_id, name, position) VALUES ($1, $2, '', 9)")
            .bind(Uuid::now_v7())
            .bind(board.as_uuid())
            .execute(&pool)
            .await
            .unwrap();
        assert!(repo.load(board).await.is_err());
    }
}

db_test! {
    async fn a_card_row_violating_the_title_rule_fails_to_map(pool: PgPool) {
        let (repo, board, todo, _done, _a, _b) = seeded(&pool).await;
        sqlx::query("INSERT INTO cards (id, column_id, title, position) VALUES ($1, $2, '', 9)")
            .bind(Uuid::now_v7())
            .bind(todo.as_uuid())
            .execute(&pool)
            .await
            .unwrap();
        assert!(repo.load(board).await.is_err());
    }
}

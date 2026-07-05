//! Integration test for the dev seed: expected counts and idempotency.

mod common;

use backend::adapters::outbound::persistence::pg_board_repo::PgBoardRepo;
use backend::adapters::outbound::persistence::pg_project_repo::PgProjectRepo;
use backend::application::seed::{SeedSummary, seed};
use backend::domain::ports::{BoardRepository, ProjectRepository};
use common::db_test;

async fn db_counts(projects: &PgProjectRepo, boards: &PgBoardRepo) -> (usize, usize, usize, usize) {
    let project_rows = projects.list().await.unwrap();
    let mut board_count = 0;
    let mut columns = 0;
    let mut cards = 0;
    for project in &project_rows {
        let summaries = boards.list_by_project(project.id()).await.unwrap();
        board_count += summaries.len();
        for summary in &summaries {
            let board = boards.load(summary.id()).await.unwrap().unwrap();
            columns += board.columns().len();
            cards += board
                .columns()
                .iter()
                .map(|c| c.cards().len())
                .sum::<usize>();
        }
    }
    (project_rows.len(), board_count, columns, cards)
}

db_test! {
    async fn seed_writes_expected_counts_and_is_idempotent(pool: PgPool) {
        let projects = PgProjectRepo::new(pool.clone());
        let boards = PgBoardRepo::new(pool.clone());

        let expected = SeedSummary { projects: 1, boards: 2, columns: 6, cards: 10 };

        let first = seed(&projects, &boards).await.unwrap();
        assert_eq!(first, expected);
        assert_eq!(db_counts(&projects, &boards).await, (1, 2, 6, 10));

        // Cards are staggered into the past to showcase relative times.
        let demo = &boards
            .list_by_project(projects.list().await.unwrap()[0].id())
            .await
            .unwrap()[0];
        let board = boards.load(demo.id()).await.unwrap().unwrap();
        let timestamps: Vec<_> = board
            .columns()
            .iter()
            .flat_map(|c| c.cards())
            .map(|card| card.created_at())
            .collect();
        let distinct = timestamps.iter().collect::<std::collections::HashSet<_>>();
        assert!(distinct.len() > 1, "expected varied created_at values");

        // Re-running must replace, not duplicate.
        let second = seed(&projects, &boards).await.unwrap();
        assert_eq!(second, expected);
        assert_eq!(db_counts(&projects, &boards).await, (1, 2, 6, 10));
    }
}

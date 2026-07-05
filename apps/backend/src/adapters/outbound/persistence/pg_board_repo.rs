use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::adapters::outbound::persistence::records::{BoardRecord, CardRecord, ColumnRecord};
use crate::domain::board::{Board, BoardSummary};
use crate::domain::card::Card;
use crate::domain::column::Column;
use crate::domain::ids::{BoardId, CardId, ColumnId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::ports::{BoardRepository, LimitedInsert, RepoResult, RepositoryError};

pub struct PgBoardRepo {
    pool: PgPool,
}

impl PgBoardRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn assemble(&self, id: BoardId) -> RepoResult<Option<Board>> {
        let Some(record) = sqlx::query_as!(
            BoardRecord,
            r#"SELECT id, project_id, name, position, created_at, updated_at
               FROM boards WHERE id = $1"#,
            id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(to_repo_error)?
        else {
            return Ok(None);
        };
        let summary = BoardSummary::try_from(record)?;

        let column_records = sqlx::query_as!(
            ColumnRecord,
            r#"SELECT id, board_id, name, position, created_at, updated_at
               FROM columns WHERE board_id = $1 ORDER BY position, created_at"#,
            id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(to_repo_error)?;
        let mut columns: Vec<Column> = column_records
            .into_iter()
            .map(Column::try_from)
            .collect::<RepoResult<_>>()?;

        let column_ids: Vec<Uuid> = columns.iter().map(|c| c.id.as_uuid()).collect();
        let card_records = sqlx::query_as!(
            CardRecord,
            r#"SELECT id, column_id, title, description, position, created_at, updated_at
               FROM cards WHERE column_id = ANY($1) ORDER BY position, created_at"#,
            &column_ids,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(to_repo_error)?;

        let mut by_column: HashMap<ColumnId, Vec<Card>> = HashMap::new();
        for record in card_records {
            let card = Card::try_from(record)?;
            by_column.entry(card.column_id).or_default().push(card);
        }
        for column in &mut columns {
            column.cards = by_column.remove(&column.id).unwrap_or_default();
        }

        Ok(Some(Board {
            id: summary.id,
            project_id: summary.project_id,
            name: summary.name,
            position: summary.position,
            created_at: summary.created_at,
            updated_at: summary.updated_at,
            columns,
        }))
    }
}

fn to_repo_error(error: sqlx::Error) -> RepositoryError {
    RepositoryError::new(error.to_string())
}

#[async_trait]
impl BoardRepository for PgBoardRepo {
    async fn list_by_project(&self, project_id: ProjectId) -> RepoResult<Vec<BoardSummary>> {
        let records = sqlx::query_as!(
            BoardRecord,
            r#"SELECT id, project_id, name, position, created_at, updated_at
               FROM boards WHERE project_id = $1 ORDER BY position, created_at"#,
            project_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(to_repo_error)?;
        records.into_iter().map(BoardSummary::try_from).collect()
    }

    async fn summary(&self, id: BoardId) -> RepoResult<Option<BoardSummary>> {
        let record = sqlx::query_as!(
            BoardRecord,
            r#"SELECT id, project_id, name, position, created_at, updated_at
               FROM boards WHERE id = $1"#,
            id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(to_repo_error)?;
        record.map(BoardSummary::try_from).transpose()
    }

    async fn load(&self, id: BoardId) -> RepoResult<Option<Board>> {
        self.assemble(id).await
    }

    async fn load_by_column(&self, column_id: ColumnId) -> RepoResult<Option<Board>> {
        let board_id = sqlx::query_scalar!(
            r#"SELECT board_id FROM columns WHERE id = $1"#,
            column_id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(to_repo_error)?;
        match board_id {
            Some(board_id) => self.assemble(BoardId::from_uuid(board_id)).await,
            None => Ok(None),
        }
    }

    async fn load_by_card(&self, card_id: CardId) -> RepoResult<Option<Board>> {
        let board_id = sqlx::query_scalar!(
            r#"SELECT col.board_id
               FROM cards c JOIN columns col ON col.id = c.column_id
               WHERE c.id = $1"#,
            card_id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(to_repo_error)?;
        match board_id {
            Some(board_id) => self.assemble(BoardId::from_uuid(board_id)).await,
            None => Ok(None),
        }
    }

    async fn insert_within_limit(&self, board: &Board, max: i64) -> RepoResult<LimitedInsert> {
        let mut tx = self.pool.begin().await.map_err(to_repo_error)?;

        // Per-project lock so count+insert is atomic without reading the `projects` table.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
            .bind(board.project_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(to_repo_error)?;

        let count = sqlx::query_scalar!(
            r#"SELECT count(*) AS "count!" FROM boards WHERE project_id = $1"#,
            board.project_id.as_uuid(),
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(to_repo_error)?;
        if count >= max {
            return Ok(LimitedInsert::LimitReached);
        }

        sqlx::query!(
            r#"INSERT INTO boards (id, project_id, name, position, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
            board.id.as_uuid(),
            board.project_id.as_uuid(),
            board.name.as_str(),
            board.position.value(),
            board.created_at,
            board.updated_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(to_repo_error)?;

        tx.commit().await.map_err(to_repo_error)?;
        Ok(LimitedInsert::Created)
    }

    async fn save(&self, board: &Board) -> RepoResult<()> {
        let mut tx = self.pool.begin().await.map_err(to_repo_error)?;

        sqlx::query!(
            r#"UPDATE boards SET name = $2, position = $3, updated_at = $4 WHERE id = $1"#,
            board.id.as_uuid(),
            board.name.as_str(),
            board.position.value(),
            board.updated_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(to_repo_error)?;

        let column_ids: Vec<Uuid> = board.columns.iter().map(|c| c.id.as_uuid()).collect();
        sqlx::query!(
            r#"DELETE FROM columns WHERE board_id = $1 AND NOT (id = ANY($2))"#,
            board.id.as_uuid(),
            &column_ids,
        )
        .execute(&mut *tx)
        .await
        .map_err(to_repo_error)?;

        for column in &board.columns {
            sqlx::query!(
                r#"INSERT INTO columns (id, board_id, name, position, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, $5, $6)
                   ON CONFLICT (id) DO UPDATE
                     SET name = EXCLUDED.name,
                         position = EXCLUDED.position,
                         updated_at = EXCLUDED.updated_at"#,
                column.id.as_uuid(),
                board.id.as_uuid(),
                column.name.as_str(),
                column.position.value(),
                column.created_at,
                column.updated_at,
            )
            .execute(&mut *tx)
            .await
            .map_err(to_repo_error)?;
        }

        let card_ids: Vec<Uuid> = board
            .columns
            .iter()
            .flat_map(|c| c.cards.iter())
            .map(|c| c.id.as_uuid())
            .collect();
        sqlx::query!(
            r#"DELETE FROM cards WHERE column_id = ANY($1) AND NOT (id = ANY($2))"#,
            &column_ids,
            &card_ids,
        )
        .execute(&mut *tx)
        .await
        .map_err(to_repo_error)?;

        for column in &board.columns {
            for card in &column.cards {
                sqlx::query!(
                    r#"INSERT INTO cards
                         (id, column_id, title, description, position, created_at, updated_at)
                       VALUES ($1, $2, $3, $4, $5, $6, $7)
                       ON CONFLICT (id) DO UPDATE
                         SET column_id = EXCLUDED.column_id,
                             title = EXCLUDED.title,
                             description = EXCLUDED.description,
                             position = EXCLUDED.position,
                             updated_at = EXCLUDED.updated_at"#,
                    card.id.as_uuid(),
                    card.column_id.as_uuid(),
                    card.title.as_str(),
                    card.description.as_str(),
                    card.position.value(),
                    card.created_at,
                    card.updated_at,
                )
                .execute(&mut *tx)
                .await
                .map_err(to_repo_error)?;
            }
        }

        tx.commit().await.map_err(to_repo_error)?;
        Ok(())
    }

    async fn update(&self, id: BoardId, name: EntityName) -> RepoResult<()> {
        sqlx::query!(
            r#"UPDATE boards SET name = $2, updated_at = now() WHERE id = $1"#,
            id.as_uuid(),
            name.as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(to_repo_error)?;
        Ok(())
    }

    async fn delete(&self, id: BoardId) -> RepoResult<()> {
        sqlx::query!(r#"DELETE FROM boards WHERE id = $1"#, id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(to_repo_error)?;
        Ok(())
    }

    async fn reorder(&self, project_id: ProjectId, ordered_ids: &[BoardId]) -> RepoResult<()> {
        let mut tx = self.pool.begin().await.map_err(to_repo_error)?;
        for (index, id) in ordered_ids.iter().enumerate() {
            sqlx::query!(
                r#"UPDATE boards SET position = $3, updated_at = now()
                   WHERE id = $1 AND project_id = $2"#,
                id.as_uuid(),
                project_id.as_uuid(),
                index as i32,
            )
            .execute(&mut *tx)
            .await
            .map_err(to_repo_error)?;
        }
        tx.commit().await.map_err(to_repo_error)?;
        Ok(())
    }
}

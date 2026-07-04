use async_trait::async_trait;
use sqlx::PgPool;

use crate::adapters::outbound::persistence::records::ColumnRecord;
use crate::domain::column::Column;
use crate::domain::ids::{BoardId, ColumnId};
use crate::domain::name::EntityName;
use crate::domain::ports::{ColumnRepository, LimitedInsert, RepoResult, RepositoryError};

pub struct PgColumnRepo {
    pool: PgPool,
}

impl PgColumnRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn to_repo_error(error: sqlx::Error) -> RepositoryError {
    RepositoryError::new(error.to_string())
}

#[async_trait]
impl ColumnRepository for PgColumnRepo {
    async fn list_by_board(&self, board_id: BoardId) -> RepoResult<Vec<Column>> {
        let records = sqlx::query_as!(
            ColumnRecord,
            r#"SELECT id, board_id, name, position, created_at, updated_at
               FROM columns
               WHERE board_id = $1
               ORDER BY position, created_at"#,
            board_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(to_repo_error)?;
        records.into_iter().map(Column::try_from).collect()
    }

    async fn get(&self, id: ColumnId) -> RepoResult<Option<Column>> {
        let record = sqlx::query_as!(
            ColumnRecord,
            r#"SELECT id, board_id, name, position, created_at, updated_at
               FROM columns
               WHERE id = $1"#,
            id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(to_repo_error)?;
        record.map(Column::try_from).transpose()
    }

    async fn insert_within_limit(&self, column: &Column, max: i64) -> RepoResult<LimitedInsert> {
        let mut tx = self.pool.begin().await.map_err(to_repo_error)?;

        let parent = sqlx::query_scalar!(
            r#"SELECT id FROM boards WHERE id = $1 FOR UPDATE"#,
            column.board_id.as_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(to_repo_error)?;
        if parent.is_none() {
            return Ok(LimitedInsert::ParentMissing);
        }

        let count = sqlx::query_scalar!(
            r#"SELECT count(*) AS "count!" FROM columns WHERE board_id = $1"#,
            column.board_id.as_uuid(),
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(to_repo_error)?;
        if count >= max {
            return Ok(LimitedInsert::LimitReached);
        }

        sqlx::query!(
            r#"INSERT INTO columns (id, board_id, name, position, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6)"#,
            column.id.as_uuid(),
            column.board_id.as_uuid(),
            column.name.as_str(),
            column.position.value(),
            column.created_at,
            column.updated_at,
        )
        .execute(&mut *tx)
        .await
        .map_err(to_repo_error)?;

        tx.commit().await.map_err(to_repo_error)?;
        Ok(LimitedInsert::Created)
    }

    async fn update(&self, id: ColumnId, name: EntityName) -> RepoResult<()> {
        sqlx::query!(
            r#"UPDATE columns SET name = $2, updated_at = now() WHERE id = $1"#,
            id.as_uuid(),
            name.as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(to_repo_error)?;
        Ok(())
    }

    async fn delete(&self, id: ColumnId) -> RepoResult<()> {
        sqlx::query!(r#"DELETE FROM columns WHERE id = $1"#, id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(to_repo_error)?;
        Ok(())
    }

    async fn reorder(&self, board_id: BoardId, ordered_ids: &[ColumnId]) -> RepoResult<()> {
        let mut tx = self.pool.begin().await.map_err(to_repo_error)?;
        for (index, id) in ordered_ids.iter().enumerate() {
            sqlx::query!(
                r#"UPDATE columns SET position = $3, updated_at = now()
                   WHERE id = $1 AND board_id = $2"#,
                id.as_uuid(),
                board_id.as_uuid(),
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

use async_trait::async_trait;
use sqlx::PgPool;

use crate::adapters::outbound::persistence::records::BoardRecord;
use crate::domain::board::Board;
use crate::domain::ids::{BoardId, ProjectId};
use crate::domain::name::EntityName;
use crate::domain::ports::{BoardRepository, LimitedInsert, RepoResult, RepositoryError};

pub struct PgBoardRepo {
    pool: PgPool,
}

impl PgBoardRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn to_repo_error(error: sqlx::Error) -> RepositoryError {
    RepositoryError::new(error.to_string())
}

#[async_trait]
impl BoardRepository for PgBoardRepo {
    async fn list_by_project(&self, project_id: ProjectId) -> RepoResult<Vec<Board>> {
        let records = sqlx::query_as!(
            BoardRecord,
            r#"SELECT id, project_id, name, position, created_at, updated_at
               FROM boards
               WHERE project_id = $1
               ORDER BY position, created_at"#,
            project_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(to_repo_error)?;
        records.into_iter().map(Board::try_from).collect()
    }

    async fn get(&self, id: BoardId) -> RepoResult<Option<Board>> {
        let record = sqlx::query_as!(
            BoardRecord,
            r#"SELECT id, project_id, name, position, created_at, updated_at
               FROM boards
               WHERE id = $1"#,
            id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(to_repo_error)?;
        record.map(Board::try_from).transpose()
    }

    async fn insert_within_limit(&self, board: &Board, max: i64) -> RepoResult<LimitedInsert> {
        let mut tx = self.pool.begin().await.map_err(to_repo_error)?;

        let parent = sqlx::query_scalar!(
            r#"SELECT id FROM projects WHERE id = $1 FOR UPDATE"#,
            board.project_id.as_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(to_repo_error)?;
        if parent.is_none() {
            return Ok(LimitedInsert::ParentMissing);
        }

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

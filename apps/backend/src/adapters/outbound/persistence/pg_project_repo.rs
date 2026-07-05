use async_trait::async_trait;
use sqlx::PgPool;

use crate::adapters::outbound::persistence::rows::ProjectRow;
use crate::domain::ids::ProjectId;
use crate::domain::name::EntityName;
use crate::domain::ports::{ProjectRepository, RepoResult};
use crate::domain::project::Project;

pub struct PgProjectRepo {
    pool: PgPool,
}

impl PgProjectRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for PgProjectRepo {
    async fn list(&self) -> RepoResult<Vec<Project>> {
        let rows = sqlx::query_as!(
            ProjectRow,
            r#"SELECT id, name, position, created_at, updated_at
               FROM projects
               ORDER BY position, created_at"#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(Project::try_from).collect()
    }

    async fn get(&self, id: ProjectId) -> RepoResult<Option<Project>> {
        let row = sqlx::query_as!(
            ProjectRow,
            r#"SELECT id, name, position, created_at, updated_at
               FROM projects
               WHERE id = $1"#,
            id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await?;
        row.map(Project::try_from).transpose()
    }

    async fn insert(&self, project: &Project) -> RepoResult<()> {
        sqlx::query!(
            r#"INSERT INTO projects (id, name, position, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5)"#,
            project.id().as_uuid(),
            project.name().as_str(),
            project.position().value(),
            project.created_at(),
            project.updated_at(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, id: ProjectId, name: EntityName) -> RepoResult<()> {
        sqlx::query!(
            r#"UPDATE projects SET name = $2, updated_at = now() WHERE id = $1"#,
            id.as_uuid(),
            name.as_str(),
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: ProjectId) -> RepoResult<()> {
        sqlx::query!(r#"DELETE FROM projects WHERE id = $1"#, id.as_uuid())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn reorder(&self, ordered_ids: &[ProjectId]) -> RepoResult<()> {
        let mut tx = self.pool.begin().await?;
        for (index, id) in ordered_ids.iter().enumerate() {
            sqlx::query!(
                r#"UPDATE projects SET position = $2, updated_at = now() WHERE id = $1"#,
                id.as_uuid(),
                index as i32,
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(())
    }
}

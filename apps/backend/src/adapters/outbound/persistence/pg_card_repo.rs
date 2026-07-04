use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::adapters::outbound::persistence::records::CardRecord;
use crate::domain::card::Card;
use crate::domain::description::Description;
use crate::domain::ids::{CardId, ColumnId};
use crate::domain::ports::{
    CardRepository, InsertOutcome, MoveOutcome, RepoResult, RepositoryError,
};
use crate::domain::title::Title;

pub struct PgCardRepo {
    pool: PgPool,
}

impl PgCardRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn to_repo_error(error: sqlx::Error) -> RepositoryError {
    RepositoryError::new(error.to_string())
}

async fn write_positions(
    tx: &mut sqlx::PgConnection,
    ordered: &[Uuid],
) -> Result<(), sqlx::Error> {
    for (index, id) in ordered.iter().enumerate() {
        sqlx::query!(
            r#"UPDATE cards SET position = $2, updated_at = now() WHERE id = $1"#,
            id,
            index as i32,
        )
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

#[async_trait]
impl CardRepository for PgCardRepo {
    async fn list_by_column(&self, column_id: ColumnId) -> RepoResult<Vec<Card>> {
        let records = sqlx::query_as!(
            CardRecord,
            r#"SELECT id, column_id, title, description, position, created_at, updated_at
               FROM cards
               WHERE column_id = $1
               ORDER BY position, created_at"#,
            column_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(to_repo_error)?;
        records.into_iter().map(Card::try_from).collect()
    }

    async fn get(&self, id: CardId) -> RepoResult<Option<Card>> {
        let record = sqlx::query_as!(
            CardRecord,
            r#"SELECT id, column_id, title, description, position, created_at, updated_at
               FROM cards
               WHERE id = $1"#,
            id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(to_repo_error)?;
        record.map(Card::try_from).transpose()
    }

    async fn insert(&self, card: &Card) -> RepoResult<InsertOutcome> {
        let mut tx = self.pool.begin().await.map_err(to_repo_error)?;

        let parent = sqlx::query_scalar!(
            r#"SELECT id FROM columns WHERE id = $1 FOR UPDATE"#,
            card.column_id.as_uuid(),
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(to_repo_error)?;
        if parent.is_none() {
            return Ok(InsertOutcome::ParentMissing);
        }

        sqlx::query!(
            r#"INSERT INTO cards (id, column_id, title, description, position, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
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

        tx.commit().await.map_err(to_repo_error)?;
        Ok(InsertOutcome::Inserted)
    }

    async fn update(&self, id: CardId, title: Title, description: Description) -> RepoResult<()> {
        sqlx::query!(
            r#"UPDATE cards SET title = $2, description = $3, updated_at = now() WHERE id = $1"#,
            id.as_uuid(),
            title.as_str(),
            description.as_str(),
        )
        .execute(&self.pool)
        .await
        .map_err(to_repo_error)?;
        Ok(())
    }

    async fn delete(&self, id: CardId) -> RepoResult<()> {
        sqlx::query!(r#"DELETE FROM cards WHERE id = $1"#, id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(to_repo_error)?;
        Ok(())
    }

    async fn move_card(
        &self,
        id: CardId,
        target_column: ColumnId,
        position: i32,
    ) -> RepoResult<MoveOutcome> {
        let card_id = id.as_uuid();
        let target_id = target_column.as_uuid();
        let mut tx = self.pool.begin().await.map_err(to_repo_error)?;

        let source = sqlx::query_scalar!(
            r#"SELECT column_id FROM cards WHERE id = $1 FOR UPDATE"#,
            card_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(to_repo_error)?;
        let source = match source {
            Some(source) => source,
            None => return Ok(MoveOutcome::CardMissing),
        };

        // The target column must exist.
        let target = sqlx::query_scalar!(
            r#"SELECT id FROM columns WHERE id = $1 FOR UPDATE"#,
            target_id,
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(to_repo_error)?;
        if target.is_none() {
            return Ok(MoveOutcome::TargetMissing);
        }

        if source == target_id {
            let mut ids: Vec<Uuid> = sqlx::query_scalar!(
                r#"SELECT id FROM cards WHERE column_id = $1 ORDER BY position, created_at"#,
                source,
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(to_repo_error)?;
            ids.retain(|x| *x != card_id);
            let pos = position.clamp(0, ids.len() as i32) as usize;
            ids.insert(pos, card_id);
            write_positions(&mut tx, &ids).await.map_err(to_repo_error)?;
        } else {
            let mut source_ids: Vec<Uuid> = sqlx::query_scalar!(
                r#"SELECT id FROM cards WHERE column_id = $1 ORDER BY position, created_at"#,
                source,
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(to_repo_error)?;
            source_ids.retain(|x| *x != card_id);
            write_positions(&mut tx, &source_ids).await.map_err(to_repo_error)?;

            let mut target_ids: Vec<Uuid> = sqlx::query_scalar!(
                r#"SELECT id FROM cards WHERE column_id = $1 ORDER BY position, created_at"#,
                target_id,
            )
            .fetch_all(&mut *tx)
            .await
            .map_err(to_repo_error)?;
            let pos = position.clamp(0, target_ids.len() as i32) as usize;
            target_ids.insert(pos, card_id);

            sqlx::query!(
                r#"UPDATE cards SET column_id = $2 WHERE id = $1"#,
                card_id,
                target_id,
            )
            .execute(&mut *tx)
            .await
            .map_err(to_repo_error)?;
            write_positions(&mut tx, &target_ids).await.map_err(to_repo_error)?;
        }

        tx.commit().await.map_err(to_repo_error)?;
        Ok(MoveOutcome::Moved)
    }
}

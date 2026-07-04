use std::sync::Arc;

use crate::application::error::ApplicationError;
use crate::domain::card::Card;
use crate::domain::description::Description;
use crate::domain::ids::{CardId, ColumnId};
use crate::domain::ports::{CardRepository, InsertOutcome, MoveOutcome};
use crate::domain::position::Position;
use crate::domain::title::Title;

#[derive(Clone)]
pub struct CardService {
    cards: Arc<dyn CardRepository>,
}

impl CardService {
    pub fn new(cards: Arc<dyn CardRepository>) -> Self {
        Self { cards }
    }

    pub async fn list(&self, column_id: ColumnId) -> Result<Vec<Card>, ApplicationError> {
        Ok(self.cards.list_by_column(column_id).await?)
    }

    pub async fn get(&self, id: CardId) -> Result<Card, ApplicationError> {
        self.cards.get(id).await?.ok_or(ApplicationError::NotFound)
    }

    pub async fn create(
        &self,
        column_id: ColumnId,
        title: &str,
        description: Option<&str>,
    ) -> Result<Card, ApplicationError> {
        let title = Title::new(title)?;
        let description = match description {
            Some(text) => Description::new(text)?,
            None => Description::default(),
        };
        let next = self
            .cards
            .list_by_column(column_id)
            .await?
            .iter()
            .map(|c| c.position.value())
            .max()
            .map_or(0, |max| max + 1);
        let card = Card::new(column_id, title, description, Position::new(next)?);
        match self.cards.insert(&card).await? {
            InsertOutcome::Inserted => Ok(card),
            InsertOutcome::ParentMissing => Err(ApplicationError::NotFound),
        }
    }

    pub async fn update(
        &self,
        id: CardId,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<Card, ApplicationError> {
        let current = self.get(id).await?;
        let title = match title {
            Some(text) => Title::new(text)?,
            None => current.title,
        };
        let description = match description {
            Some(text) => Description::new(text)?,
            None => current.description,
        };
        self.cards.update(id, title, description).await?;
        self.get(id).await
    }

    pub async fn delete(&self, id: CardId) -> Result<(), ApplicationError> {
        if self.cards.get(id).await?.is_none() {
            return Err(ApplicationError::NotFound);
        }
        self.cards.delete(id).await?;
        Ok(())
    }

    pub async fn move_card(
        &self,
        id: CardId,
        target_column: ColumnId,
        position: i32,
    ) -> Result<(), ApplicationError> {
        Position::new(position)?;
        match self.cards.move_card(id, target_column, position).await? {
            MoveOutcome::Moved => Ok(()),
            MoveOutcome::CardMissing | MoveOutcome::TargetMissing => {
                Err(ApplicationError::NotFound)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::DomainError;
    use crate::domain::ports::{RepoResult, RepositoryError};
    use async_trait::async_trait;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeCardRepo {
        rows: Mutex<Vec<Card>>,
        fail: bool,
    }

    impl FakeCardRepo {
        fn new() -> Arc<Self> {
            Arc::new(Self::default())
        }

        fn failing() -> Arc<Self> {
            Arc::new(Self {
                rows: Mutex::new(Vec::new()),
                fail: true,
            })
        }
    }

    fn column_ids(rows: &[Card], column: ColumnId) -> Vec<CardId> {
        let mut cards: Vec<&Card> = rows.iter().filter(|c| c.column_id == column).collect();
        cards.sort_by_key(|c| c.position.value());
        cards.into_iter().map(|c| c.id).collect()
    }

    fn reseq(rows: &mut [Card], ordered: &[CardId]) {
        for (index, id) in ordered.iter().enumerate() {
            if let Some(c) = rows.iter_mut().find(|c| c.id == *id) {
                c.position = Position::new(index as i32).unwrap();
            }
        }
    }

    #[async_trait]
    impl CardRepository for FakeCardRepo {
        async fn list_by_column(&self, column_id: ColumnId) -> RepoResult<Vec<Card>> {
            if self.fail {
                return Err(RepositoryError::new("boom"));
            }
            let ids = column_ids(&self.rows.lock().unwrap(), column_id);
            let rows = self.rows.lock().unwrap();
            Ok(ids
                .iter()
                .map(|id| rows.iter().find(|c| c.id == *id).unwrap().clone())
                .collect())
        }

        async fn get(&self, id: CardId) -> RepoResult<Option<Card>> {
            Ok(self.rows.lock().unwrap().iter().find(|c| c.id == id).cloned())
        }

        async fn insert(&self, card: &Card) -> RepoResult<InsertOutcome> {
            self.rows.lock().unwrap().push(card.clone());
            Ok(InsertOutcome::Inserted)
        }

        async fn update(&self, id: CardId, title: Title, description: Description) -> RepoResult<()> {
            if let Some(c) = self.rows.lock().unwrap().iter_mut().find(|c| c.id == id) {
                c.title = title;
                c.description = description;
            }
            Ok(())
        }

        async fn delete(&self, id: CardId) -> RepoResult<()> {
            self.rows.lock().unwrap().retain(|c| c.id != id);
            Ok(())
        }

        async fn move_card(
            &self,
            id: CardId,
            target_column: ColumnId,
            position: i32,
        ) -> RepoResult<MoveOutcome> {
            let mut rows = self.rows.lock().unwrap();
            let source = match rows.iter().find(|c| c.id == id) {
                Some(c) => c.column_id,
                None => return Ok(MoveOutcome::CardMissing),
            };
            if source == target_column {
                let mut ids = column_ids(&rows, source);
                ids.retain(|x| *x != id);
                let pos = position.clamp(0, ids.len() as i32) as usize;
                ids.insert(pos, id);
                reseq(&mut rows, &ids);
            } else {
                let mut src = column_ids(&rows, source);
                src.retain(|x| *x != id);
                reseq(&mut rows, &src);
                let mut tgt = column_ids(&rows, target_column);
                let pos = position.clamp(0, tgt.len() as i32) as usize;
                tgt.insert(pos, id);
                if let Some(c) = rows.iter_mut().find(|c| c.id == id) {
                    c.column_id = target_column;
                }
                reseq(&mut rows, &tgt);
            }
            Ok(MoveOutcome::Moved)
        }
    }

    async fn seed(service: &CardService, column: ColumnId, names: &[&str]) -> Vec<CardId> {
        let mut ids = Vec::new();
        for name in names {
            ids.push(service.create(column, name, None).await.unwrap().id);
        }
        ids
    }

    async fn titles(service: &CardService, column: ColumnId) -> Vec<String> {
        service
            .list(column)
            .await
            .unwrap()
            .iter()
            .map(|c| c.title.as_str().to_owned())
            .collect()
    }

    #[tokio::test]
    async fn create_appends_and_defaults_description_to_empty() {
        let service = CardService::new(FakeCardRepo::new());
        let column = ColumnId::new();

        let first = service.create(column, "First", None).await.unwrap();
        let second = service.create(column, "Second", Some("body")).await.unwrap();

        assert_eq!(first.position.value(), 0);
        assert_eq!(first.description.as_str(), "");
        assert_eq!(second.position.value(), 1);
        assert_eq!(second.description.as_str(), "body");
    }

    #[tokio::test]
    async fn create_rejects_a_blank_title() {
        let service = CardService::new(FakeCardRepo::new());
        let err = service.create(ColumnId::new(), "  ", None).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::EmptyTitle)
        ));
    }

    #[tokio::test]
    async fn create_rejects_an_over_long_description() {
        let service = CardService::new(FakeCardRepo::new());
        let long = "x".repeat(10_001);
        let err = service.create(ColumnId::new(), "ok", Some(&long)).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::DescriptionTooLong { .. })
        ));
    }

    #[tokio::test]
    async fn create_surfaces_repository_errors() {
        let service = CardService::new(FakeCardRepo::failing());
        let err = service.create(ColumnId::new(), "x", None).await.unwrap_err();
        assert!(matches!(err, ApplicationError::Repository(_)));
    }

    #[tokio::test]
    async fn update_changes_only_the_provided_fields() {
        let service = CardService::new(FakeCardRepo::new());
        let column = ColumnId::new();
        let card = service.create(column, "Title", Some("desc")).await.unwrap();

        let updated = service.update(card.id, None, Some("new desc")).await.unwrap();
        assert_eq!(updated.title.as_str(), "Title");
        assert_eq!(updated.description.as_str(), "new desc");

        let updated = service.update(card.id, Some("New Title"), None).await.unwrap();
        assert_eq!(updated.title.as_str(), "New Title");
        assert_eq!(updated.description.as_str(), "new desc");
    }

    #[tokio::test]
    async fn update_rejects_a_blank_title() {
        let service = CardService::new(FakeCardRepo::new());
        let card = service.create(ColumnId::new(), "Title", None).await.unwrap();
        let err = service.update(card.id, Some(""), None).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::EmptyTitle)
        ));
    }

    #[tokio::test]
    async fn get_and_update_and_delete_of_a_missing_card_are_not_found() {
        let service = CardService::new(FakeCardRepo::new());
        let id = CardId::new();
        assert!(matches!(service.get(id).await.unwrap_err(), ApplicationError::NotFound));
        assert!(matches!(
            service.update(id, Some("x"), None).await.unwrap_err(),
            ApplicationError::NotFound
        ));
        assert!(matches!(service.delete(id).await.unwrap_err(), ApplicationError::NotFound));
    }

    #[tokio::test]
    async fn delete_removes_a_card() {
        let service = CardService::new(FakeCardRepo::new());
        let column = ColumnId::new();
        let card = service.create(column, "Bye", None).await.unwrap();
        service.delete(card.id).await.unwrap();
        assert!(service.list(column).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn move_rejects_a_negative_position() {
        let service = CardService::new(FakeCardRepo::new());
        let column = ColumnId::new();
        let card = service.create(column, "A", None).await.unwrap();
        let err = service.move_card(card.id, column, -1).await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::NegativePosition(-1))
        ));
    }

    #[tokio::test]
    async fn move_of_a_missing_card_is_not_found() {
        let service = CardService::new(FakeCardRepo::new());
        let err = service.move_card(CardId::new(), ColumnId::new(), 0).await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound));
    }

    #[tokio::test]
    async fn move_same_column_downwards() {
        let service = CardService::new(FakeCardRepo::new());
        let col = ColumnId::new();
        let ids = seed(&service, col, &["A", "B", "C"]).await;

        service.move_card(ids[0], col, 2).await.unwrap();
        assert_eq!(titles(&service, col).await, ["B", "C", "A"]);
    }

    #[tokio::test]
    async fn move_same_column_upwards() {
        let service = CardService::new(FakeCardRepo::new());
        let col = ColumnId::new();
        let ids = seed(&service, col, &["A", "B", "C"]).await;

        service.move_card(ids[2], col, 0).await.unwrap();
        assert_eq!(titles(&service, col).await, ["C", "A", "B"]);
    }

    #[tokio::test]
    async fn move_cross_column_to_start_middle_and_end() {
        let service = CardService::new(FakeCardRepo::new());
        let src = ColumnId::new();
        let dst = ColumnId::new();

        // to start
        let a = seed(&service, src, &["A"]).await[0];
        seed(&service, dst, &["X", "Y"]).await;
        service.move_card(a, dst, 0).await.unwrap();
        assert_eq!(titles(&service, dst).await, ["A", "X", "Y"]);
        assert!(titles(&service, src).await.is_empty());

        // to middle
        let b = seed(&service, src, &["B"]).await[0];
        service.move_card(b, dst, 1).await.unwrap();
        assert_eq!(titles(&service, dst).await, ["A", "B", "X", "Y"]);

        // to end (position past the end clamps to append)
        let c = seed(&service, src, &["C"]).await[0];
        service.move_card(c, dst, 99).await.unwrap();
        assert_eq!(titles(&service, dst).await, ["A", "B", "X", "Y", "C"]);
    }
}

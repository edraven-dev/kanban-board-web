use std::sync::Arc;

use crate::application::error::ApplicationError;
use crate::application::reorder::ensure_permutation;
use crate::domain::ids::ProjectId;
use crate::domain::name::EntityName;
use crate::domain::ports::ProjectRepository;
use crate::domain::position::Position;
use crate::domain::project::Project;

#[derive(Clone)]
pub struct ProjectService {
    projects: Arc<dyn ProjectRepository>,
}

impl ProjectService {
    pub fn new(projects: Arc<dyn ProjectRepository>) -> Self {
        Self { projects }
    }

    pub async fn list(&self) -> Result<Vec<Project>, ApplicationError> {
        Ok(self.projects.list().await?)
    }

    pub async fn create(&self, name: &str) -> Result<Project, ApplicationError> {
        let name = EntityName::new(name)?;
        let next = self
            .projects
            .list()
            .await?
            .iter()
            .map(|p| p.position.value())
            .max()
            .map_or(0, |max| max + 1);
        let project = Project::new(name, Position::new(next)?);
        self.projects.insert(&project).await?;
        Ok(project)
    }

    pub async fn update(&self, id: ProjectId, name: &str) -> Result<Project, ApplicationError> {
        let name = EntityName::new(name)?;
        self.projects.update(id, name).await?;
        self.projects
            .get(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn delete(&self, id: ProjectId) -> Result<(), ApplicationError> {
        if self.projects.get(id).await?.is_none() {
            return Err(ApplicationError::NotFound);
        }
        self.projects.delete(id).await?;
        Ok(())
    }

    pub async fn reorder(&self, ordered_ids: Vec<ProjectId>) -> Result<(), ApplicationError> {
        let existing: Vec<ProjectId> = self
            .projects
            .list()
            .await?
            .into_iter()
            .map(|p| p.id)
            .collect();
        ensure_permutation(&existing, &ordered_ids)?;
        self.projects.reorder(&ordered_ids).await?;
        Ok(())
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
    struct FakeProjectRepo {
        rows: Mutex<Vec<Project>>,
        fail: bool,
    }

    impl FakeProjectRepo {
        fn with(rows: Vec<Project>) -> Arc<Self> {
            Arc::new(Self {
                rows: Mutex::new(rows),
                fail: false,
            })
        }

        fn failing() -> Arc<Self> {
            Arc::new(Self {
                rows: Mutex::new(Vec::new()),
                fail: true,
            })
        }
    }

    fn project(name: &str, position: i32) -> Project {
        Project::new(
            EntityName::new(name).unwrap(),
            Position::new(position).unwrap(),
        )
    }

    #[async_trait]
    impl ProjectRepository for FakeProjectRepo {
        async fn list(&self) -> RepoResult<Vec<Project>> {
            if self.fail {
                return Err(RepositoryError::new("boom"));
            }
            let mut rows = self.rows.lock().unwrap().clone();
            rows.sort_by_key(|p| p.position.value());
            Ok(rows)
        }

        async fn get(&self, id: ProjectId) -> RepoResult<Option<Project>> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned())
        }

        async fn insert(&self, project: &Project) -> RepoResult<()> {
            self.rows.lock().unwrap().push(project.clone());
            Ok(())
        }

        async fn update(&self, id: ProjectId, name: EntityName) -> RepoResult<()> {
            if let Some(p) = self.rows.lock().unwrap().iter_mut().find(|p| p.id == id) {
                p.name = name;
            }
            Ok(())
        }

        async fn delete(&self, id: ProjectId) -> RepoResult<()> {
            self.rows.lock().unwrap().retain(|p| p.id != id);
            Ok(())
        }

        async fn reorder(&self, ordered_ids: &[ProjectId]) -> RepoResult<()> {
            let mut rows = self.rows.lock().unwrap();
            for (index, id) in ordered_ids.iter().enumerate() {
                if let Some(p) = rows.iter_mut().find(|p| p.id == *id) {
                    p.position = Position::new(index as i32).unwrap();
                }
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn create_appends_at_the_next_position() {
        let repo = FakeProjectRepo::with(vec![project("A", 0), project("B", 1)]);
        let service = ProjectService::new(repo.clone());

        let created = service.create("  Roadmap  ").await.unwrap();

        assert_eq!(created.name.as_str(), "Roadmap");
        assert_eq!(created.position.value(), 2);
        assert_eq!(service.list().await.unwrap().len(), 3);
    }

    #[tokio::test]
    async fn create_of_the_first_project_starts_at_zero() {
        let service = ProjectService::new(FakeProjectRepo::with(vec![]));
        let created = service.create("First").await.unwrap();
        assert_eq!(created.position.value(), 0);
    }

    #[tokio::test]
    async fn create_rejects_a_blank_name() {
        let service = ProjectService::new(FakeProjectRepo::with(vec![]));
        let err = service.create("   ").await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::EmptyName)
        ));
    }

    #[tokio::test]
    async fn create_surfaces_repository_errors() {
        let service = ProjectService::new(FakeProjectRepo::failing());
        let err = service.create("x").await.unwrap_err();
        assert!(matches!(err, ApplicationError::Repository(_)));
    }

    #[tokio::test]
    async fn update_changes_an_existing_project() {
        let existing = project("Old", 0);
        let id = existing.id;
        let service = ProjectService::new(FakeProjectRepo::with(vec![existing]));

        let updated = service.update(id, "New").await.unwrap();
        assert_eq!(updated.name.as_str(), "New");
    }

    #[tokio::test]
    async fn update_of_a_missing_project_is_not_found() {
        let service = ProjectService::new(FakeProjectRepo::with(vec![]));
        let err = service.update(ProjectId::new(), "New").await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound));
    }

    #[tokio::test]
    async fn update_rejects_a_blank_name() {
        let existing = project("Old", 0);
        let id = existing.id;
        let service = ProjectService::new(FakeProjectRepo::with(vec![existing]));
        let err = service.update(id, "").await.unwrap_err();
        assert!(matches!(
            err,
            ApplicationError::Domain(DomainError::EmptyName)
        ));
    }

    #[tokio::test]
    async fn delete_removes_an_existing_project() {
        let existing = project("Gone", 0);
        let id = existing.id;
        let service = ProjectService::new(FakeProjectRepo::with(vec![existing]));

        service.delete(id).await.unwrap();
        assert!(service.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn delete_of_a_missing_project_is_not_found() {
        let service = ProjectService::new(FakeProjectRepo::with(vec![]));
        let err = service.delete(ProjectId::new()).await.unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound));
    }

    #[tokio::test]
    async fn reorder_rewrites_positions_for_a_valid_permutation() {
        let a = project("A", 0);
        let b = project("B", 1);
        let c = project("C", 2);
        let (ia, ib, ic) = (a.id, b.id, c.id);
        let service = ProjectService::new(FakeProjectRepo::with(vec![a, b, c]));

        service.reorder(vec![ic, ia, ib]).await.unwrap();

        let ordered: Vec<ProjectId> = service.list().await.unwrap().iter().map(|p| p.id).collect();
        assert_eq!(ordered, vec![ic, ia, ib]);
    }

    #[tokio::test]
    async fn reorder_rejects_a_non_permutation() {
        let a = project("A", 0);
        let b = project("B", 1);
        let ia = a.id;
        let service = ProjectService::new(FakeProjectRepo::with(vec![a, b]));

        let err = service
            .reorder(vec![ia, ProjectId::new()])
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Unprocessable(_)));
    }

    #[tokio::test]
    async fn reorder_rejects_duplicate_ids() {
        let a = project("A", 0);
        let b = project("B", 1);
        let (ia, ib) = (a.id, b.id);
        let service = ProjectService::new(FakeProjectRepo::with(vec![a, b]));

        let err = service.reorder(vec![ia, ia, ib]).await.unwrap_err();
        assert!(matches!(err, ApplicationError::Unprocessable(_)));
    }
}

use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::ids::ProjectId;
use crate::domain::ports::{ProjectApi, ProjectRepository, RepoResult};

#[derive(Clone)]
pub struct ProjectApiGateway {
    projects: Arc<dyn ProjectRepository>,
}

impl ProjectApiGateway {
    pub fn new(projects: Arc<dyn ProjectRepository>) -> Self {
        Self { projects }
    }
}

#[async_trait]
impl ProjectApi for ProjectApiGateway {
    async fn exists(&self, id: ProjectId) -> RepoResult<bool> {
        Ok(self.projects.get(id).await?.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::name::EntityName;
    use crate::domain::position::Position;
    use crate::domain::project::Project;
    use std::sync::Mutex;

    struct FakeProjectRepo {
        rows: Mutex<Vec<Project>>,
    }

    #[async_trait]
    impl ProjectRepository for FakeProjectRepo {
        async fn list(&self) -> RepoResult<Vec<Project>> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn get(&self, id: ProjectId) -> RepoResult<Option<Project>> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id() == id)
                .cloned())
        }
        async fn insert(&self, _project: &Project) -> RepoResult<()> {
            Ok(())
        }
        async fn update(&self, _id: ProjectId, _name: EntityName) -> RepoResult<()> {
            Ok(())
        }
        async fn delete(&self, _id: ProjectId) -> RepoResult<()> {
            Ok(())
        }
        async fn reorder(&self, _ordered_ids: &[ProjectId]) -> RepoResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn reports_whether_a_project_exists() {
        let project = Project::new(EntityName::new("P").unwrap(), Position::new(0).unwrap());
        let id = project.id();
        let gateway = ProjectApiGateway::new(Arc::new(FakeProjectRepo {
            rows: Mutex::new(vec![project]),
        }));

        assert!(gateway.exists(id).await.unwrap());
        assert!(!gateway.exists(ProjectId::new()).await.unwrap());
    }
}

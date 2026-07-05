use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::ids::ProjectId;
use crate::domain::ports::{ProjectDirectory, ProjectRepository, RepoResult};

/// Backs [`ProjectDirectory`] with the Project repo; across a service split, an API call.
#[derive(Clone)]
pub struct ProjectDirectoryService {
    projects: Arc<dyn ProjectRepository>,
}

impl ProjectDirectoryService {
    pub fn new(projects: Arc<dyn ProjectRepository>) -> Self {
        Self { projects }
    }
}

#[async_trait]
impl ProjectDirectory for ProjectDirectoryService {
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
                .find(|p| p.id == id)
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
        let id = project.id;
        let directory = ProjectDirectoryService::new(Arc::new(FakeProjectRepo {
            rows: Mutex::new(vec![project]),
        }));

        assert!(directory.exists(id).await.unwrap());
        assert!(!directory.exists(ProjectId::new()).await.unwrap());
    }
}

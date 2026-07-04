use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::application::error::ApplicationError;

fn status_and_code(error: &ApplicationError) -> (StatusCode, &'static str) {
    match error {
        ApplicationError::Domain(_) => (StatusCode::BAD_REQUEST, "validation"),
        ApplicationError::NotFound => (StatusCode::NOT_FOUND, "not_found"),
        ApplicationError::LimitExceeded => (StatusCode::CONFLICT, "limit_exceeded"),
        ApplicationError::Unprocessable(_) => (StatusCode::UNPROCESSABLE_ENTITY, "unprocessable"),
        ApplicationError::Repository(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
    }
}

impl IntoResponse for ApplicationError {
    fn into_response(self) -> Response {
        let (status, code) = status_and_code(&self);
        let body = Json(json!({ "error": { "code": code, "message": self.to_string() } }));
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::error::DomainError;
    use crate::domain::ports::RepositoryError;
    use axum::body::to_bytes;

    #[test]
    fn each_variant_maps_to_the_expected_status_and_code() {
        let cases = [
            (
                ApplicationError::Domain(DomainError::EmptyName),
                StatusCode::BAD_REQUEST,
                "validation",
            ),
            (ApplicationError::NotFound, StatusCode::NOT_FOUND, "not_found"),
            (
                ApplicationError::LimitExceeded,
                StatusCode::CONFLICT,
                "limit_exceeded",
            ),
            (
                ApplicationError::Unprocessable("bad".into()),
                StatusCode::UNPROCESSABLE_ENTITY,
                "unprocessable",
            ),
            (
                ApplicationError::Repository(RepositoryError::new("boom")),
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
            ),
        ];
        for (error, status, code) in cases {
            assert_eq!(status_and_code(&error), (status, code));
        }
    }

    #[tokio::test]
    async fn into_response_writes_the_problem_body() {
        let response = ApplicationError::NotFound.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["error"]["code"], "not_found");
        assert_eq!(body["error"]["message"], "not found");
    }
}

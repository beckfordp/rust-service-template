// SPDX-License-Identifier: MIT

//! `GET /examples/{id}`: the shape of a route — the path parsed into the
//! domain, the store through `AppState`, and one error enum whose
//! `IntoResponse` maps each case to its status. Replaced by the Scala
//! service's first route.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use service_basics::text_error::TextError;
use time::format_description::well_known::Rfc3339;

use crate::domain::{Example, InvalidId};
use crate::server::AppState;
use crate::store::{Examples, StoreError};

/// An example, as JSON.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExampleResponse {
    /// Its id.
    pub id: String,
    /// Its name.
    pub name: String,
    /// When it was made (RFC 3339).
    pub created_at: String,
}

impl From<Example> for ExampleResponse {
    fn from(example: Example) -> Self {
        Self {
            id: example.id.to_string(),
            name: example.name,
            created_at: example
                .created_at
                .format(&Rfc3339)
                .unwrap_or_else(|_| example.created_at.to_string()),
        }
    }
}

/// Why `GET /examples/{id}` didn't answer 200.
#[derive(Debug, thiserror::Error)]
pub enum ExampleError {
    /// The id isn't a UUID.
    #[error(transparent)]
    Id(#[from] InvalidId),
    /// No example has it.
    #[error("Not found")]
    NotFound,
    /// The store failed.
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl IntoResponse for ExampleError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::Id(error) => (StatusCode::BAD_REQUEST, error.to_string()),
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Store(error) => {
                eprintln!("{{project-name}}: GET /examples failed: {error}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_owned(),
                )
            }
        };
        TextError { status, message }.into_response()
    }
}

/// `GET /examples/{id}`.
///
/// # Errors
/// [`ExampleError`]: a malformed id (400), an unknown one (404), a store
/// failure (500).
pub async fn get_example<S: Examples>(
    State(state): State<AppState<S>>,
    Path(id): Path<String>,
) -> Result<Json<ExampleResponse>, ExampleError> {
    let example = state
        .store
        .get(id.parse()?)
        .await?
        .ok_or(ExampleError::NotFound)?;
    Ok(Json(example.into()))
}

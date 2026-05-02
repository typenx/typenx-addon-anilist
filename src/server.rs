use crate::{api::AniListClient, types::*};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    client: AniListClient,
    manifest: Arc<AddonManifest>,
}

pub fn app(client: AniListClient) -> Router {
    let state = AppState {
        client,
        manifest: Arc::new(manifest()),
    };
    Router::new()
        .route("/health", get(health))
        .route("/manifest", get(manifest_route))
        .route("/catalog", post(catalog))
        .route("/search", post(search))
        .route("/anime/{id}", get(anime))
        .route("/manga/{id}", get(manga))
        .route("/recommendations", post(recommendations))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn serve(client: AniListClient, port: u16) -> anyhow::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Typenx addon listening on http://127.0.0.1:{port}");
    axum::serve(listener, app(client)).await?;
    Ok(())
}

async fn health() -> Json<AddonHealth> {
    Json(AddonHealth {
        ok: true,
        message: None,
    })
}

async fn manifest_route(State(state): State<AppState>) -> Json<AddonManifest> {
    Json((*state.manifest).clone())
}

async fn catalog(
    State(state): State<AppState>,
    Json(request): Json<CatalogRequest>,
) -> Result<Json<CatalogResponse>, AddonError> {
    Ok(Json(state.client.catalog(request).await?))
}

async fn search(
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<CatalogResponse>, AddonError> {
    Ok(Json(state.client.search(request).await?))
}

async fn anime(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AnimeMetadata>, AddonError> {
    Ok(Json(state.client.anime(&id).await?))
}

async fn manga(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AnimeMetadata>, AddonError> {
    Ok(Json(state.client.manga(&id).await?))
}

async fn recommendations(
    State(state): State<AppState>,
    Json(request): Json<RecommendationRequest>,
) -> Result<Json<RecommendationResponse>, AddonError> {
    Ok(Json(state.client.recommendations(request).await?))
}

async fn not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "message": "Not found" })),
    )
}

#[derive(Debug)]
struct AddonError(anyhow::Error);

impl<E> From<E> for AddonError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self(error.into())
    }
}

impl IntoResponse for AddonError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "message": self.0.to_string() })),
        )
            .into_response()
    }
}

pub fn manifest() -> AddonManifest {
    AddonManifest {
        id: "typenx-addon-anilist".into(),
        name: "AniList".into(),
        version: "0.1.0".into(),
        description: Some("Official Typenx metadata addon backed by AniList.".into()),
        icon: Some("https://anilist.co/img/icons/android-chrome-512x512.png".into()),
        resources: vec![
            AddonResource::Catalog,
            AddonResource::Search,
            AddonResource::AnimeMeta,
            AddonResource::Recommendations,
        ],
        catalogs: vec![
            catalog_def("popular", "Popular Anime", ContentType::Anime),
            catalog_def("trending", "Trending Anime", ContentType::Anime),
            catalog_def("airing", "Airing Anime", ContentType::Anime),
            catalog_def("manga-popular", "Popular Manga", ContentType::Manga),
            catalog_def("manga-trending", "Trending Manga", ContentType::Manga),
            catalog_def("manga-rated", "Top Rated Manga", ContentType::Manga),
        ],
    }
}

fn catalog_def(id: &str, name: &str, content_type: ContentType) -> CatalogDefinition {
    CatalogDefinition {
        id: id.into(),
        name: name.into(),
        content_type,
        filters: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use tower::ServiceExt;

    #[tokio::test]
    async fn manifest_route_smoke_test() {
        let response = app(AniListClient::new())
            .oneshot(
                axum::http::Request::builder()
                    .uri("/manifest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

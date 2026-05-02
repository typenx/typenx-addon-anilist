use crate::{mapping, recommendations, types::*};
use anyhow::{Context, bail};
use reqwest::Client;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Value, json};

const API_URL: &str = "https://graphql.anilist.co";
const MEDIA_FIELDS: &str = r#"
  id idMal type title { romaji english native } synonyms description(asHtml: false)
  format status season seasonYear episodes chapters volumes duration source(version: 3)
  countryOfOrigin startDate { year month day } endDate { year month day } genres
  tags { name rank isMediaSpoiler } averageScore meanScore popularity
  rankings { rank type allTime context } updatedAt siteUrl trailer { id site }
  externalLinks { site url } coverImage { extraLarge large medium } bannerImage
  studios { nodes { name isAnimationStudio } }
  staff(perPage: 12, sort: RELEVANCE) { edges { role node { name { full } } } }
  streamingEpisodes { title thumbnail site }
  airingSchedule(notYetAired: false, perPage: 25) { nodes { episode airingAt } }
"#;

fn catalog_query() -> String {
    format!(
        "query TypenxCatalog($page: Int!, $perPage: Int!, $type: MediaType!, $sort: [MediaSort], $status: MediaStatus) {{ Page(page: $page, perPage: $perPage) {{ media(type: $type, isAdult: false, sort: $sort, status: $status) {{ {MEDIA_FIELDS} }} }} }}"
    )
}

fn search_query() -> String {
    format!(
        "query TypenxSearch($page: Int!, $perPage: Int!, $type: MediaType!, $search: String!) {{ Page(page: $page, perPage: $perPage) {{ media(type: $type, isAdult: false, search: $search, sort: SEARCH_MATCH) {{ {MEDIA_FIELDS} }} }} }}"
    )
}

fn media_query() -> String {
    format!(
        "query TypenxMedia($id: Int!, $type: MediaType!) {{ Media(id: $id, type: $type) {{ {MEDIA_FIELDS} }} }}"
    )
}

#[derive(Clone)]
pub struct AniListClient {
    http: Client,
}

impl AniListClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }

    pub async fn catalog(&self, request: CatalogRequest) -> anyhow::Result<CatalogResponse> {
        let limit = clamp_limit(request.limit);
        if let Some(query) = request.query.filter(|query| !query.is_empty()) {
            return self
                .search(SearchRequest {
                    addon_id: request.addon_id,
                    query,
                    content_type: request.content_type,
                    limit: Some(limit),
                })
                .await;
        }

        self.catalog_page(
            &request.catalog_id,
            request.content_type.as_ref(),
            request.skip,
            limit,
        )
        .await
    }

    pub async fn search(&self, request: SearchRequest) -> anyhow::Result<CatalogResponse> {
        let limit = clamp_limit(request.limit);
        let query = request.query.trim();
        if query.is_empty() {
            return self
                .catalog_page("popular", request.content_type.as_ref(), None, limit)
                .await;
        }

        let data: PageResponse = self
            .graphql(
                &search_query(),
                json!({
                    "page": 1,
                    "perPage": limit,
                    "type": media_type_of(request.content_type.as_ref(), ""),
                    "search": query,
                }),
            )
            .await?;
        Ok(CatalogResponse {
            items: data.page.media.iter().map(mapping::to_preview).collect(),
        })
    }

    pub async fn anime(&self, id: &str) -> anyhow::Result<AnimeMetadata> {
        self.media(id, "ANIME", "anime").await
    }

    pub async fn manga(&self, id: &str) -> anyhow::Result<AnimeMetadata> {
        self.media(id, "MANGA", "manga").await
    }

    pub async fn recommendations(
        &self,
        request: RecommendationRequest,
    ) -> anyhow::Result<RecommendationResponse> {
        recommendations::recommend_anime(request, self).await
    }

    async fn media(
        &self,
        id: &str,
        media_type: &str,
        label: &str,
    ) -> anyhow::Result<AnimeMetadata> {
        let numeric_id: i64 = id
            .parse()
            .ok()
            .filter(|value| *value > 0)
            .with_context(|| format!("Invalid AniList {label} id: {id}"))?;
        let data: MediaResponse = self
            .graphql(
                &media_query(),
                json!({ "id": numeric_id, "type": media_type }),
            )
            .await?;
        let media = data
            .media
            .with_context(|| format!("AniList {label} not found: {id}"))?;
        Ok(mapping::to_metadata(&media))
    }

    async fn catalog_page(
        &self,
        catalog_id: &str,
        content_type: Option<&ContentType>,
        skip: Option<u32>,
        limit: u32,
    ) -> anyhow::Result<CatalogResponse> {
        let mut variables = json!({
            "page": skip_to_page(skip, limit),
            "perPage": limit,
            "type": media_type_of(content_type, catalog_id),
        });
        let catalog_vars = catalog_variables(catalog_id);
        variables
            .as_object_mut()
            .expect("json object")
            .extend(catalog_vars);

        let data: PageResponse = self.graphql(&catalog_query(), variables).await?;
        Ok(CatalogResponse {
            items: data.page.media.iter().map(mapping::to_preview).collect(),
        })
    }

    async fn graphql<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: Value,
    ) -> anyhow::Result<T> {
        let response = self
            .http
            .post(API_URL)
            .header("accept", "application/json")
            .json(&json!({ "query": query, "variables": variables }))
            .send()
            .await
            .context("failed to call AniList")?;
        let status = response.status();
        let body: GraphQlResponse<T> = response
            .json()
            .await
            .context("AniList returned an empty response")?;

        if !status.is_success() {
            bail!("AniList returned {}", status.as_u16());
        }
        if let Some(errors) = body.errors.filter(|errors| !errors.is_empty()) {
            bail!(
                "{}",
                errors
                    .into_iter()
                    .filter_map(|error| error.message)
                    .collect::<Vec<_>>()
                    .join("; ")
            );
        }
        body.data.context("AniList response did not include data")
    }
}

impl Default for AniListClient {
    fn default() -> Self {
        Self::new()
    }
}

fn catalog_variables(catalog_id: &str) -> serde_json::Map<String, Value> {
    let mut variables = serde_json::Map::new();
    match catalog_id {
        "trending" | "manga-trending" => variables.insert("sort".into(), json!(["TRENDING_DESC"])),
        "airing" => {
            variables.insert("sort".into(), json!(["POPULARITY_DESC"]));
            variables.insert("status".into(), json!("RELEASING"))
        }
        "manga-rated" => variables.insert("sort".into(), json!(["SCORE_DESC"])),
        _ => variables.insert("sort".into(), json!(["POPULARITY_DESC"])),
    };
    variables
}

fn media_type_of(content_type: Option<&ContentType>, catalog_id: &str) -> &'static str {
    if matches!(content_type, Some(ContentType::Manga))
        || matches!(content_type, Some(ContentType::Manhwa))
        || matches!(content_type, Some(ContentType::Manhua))
        || matches!(content_type, Some(ContentType::LightNovel))
        || catalog_id.starts_with("manga-")
    {
        "MANGA"
    } else {
        "ANIME"
    }
}

fn clamp_limit(limit: Option<u32>) -> u32 {
    limit.unwrap_or(24).clamp(1, 50)
}

fn skip_to_page(skip: Option<u32>, limit: u32) -> u32 {
    skip.unwrap_or(0) / limit + 1
}

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PageResponse {
    #[serde(rename = "Page")]
    page: PageData,
}

#[derive(Debug, Deserialize)]
struct PageData {
    media: Vec<mapping::AniListMedia>,
}

#[derive(Debug, Deserialize)]
struct MediaResponse {
    #[serde(rename = "Media")]
    media: Option<mapping::AniListMedia>,
}

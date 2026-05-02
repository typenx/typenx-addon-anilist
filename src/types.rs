use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Anime,
    Manga,
    Manhwa,
    Manhua,
    LightNovel,
    Movie,
    Ova,
    Ona,
    Special,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AddonResource {
    Catalog,
    Search,
    AnimeMeta,
    EpisodeMeta,
    VideoSources,
    MangaPages,
    Recommendations,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddonManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub resources: Vec<AddonResource>,
    pub catalogs: Vec<CatalogDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogDefinition {
    pub id: String,
    pub name: String,
    pub content_type: ContentType,
    pub filters: Vec<CatalogFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogFilter {
    pub id: String,
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AddonHealth {
    pub ok: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct CatalogRequest {
    pub addon_id: Option<String>,
    pub catalog_id: String,
    pub content_type: Option<ContentType>,
    pub skip: Option<u32>,
    pub limit: Option<u32>,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    pub addon_id: Option<String>,
    pub query: String,
    pub content_type: Option<ContentType>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogResponse {
    pub items: Vec<AnimePreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeasonEntry {
    pub id: String,
    pub title: String,
    pub season_number: Option<u32>,
    pub year: Option<i32>,
    pub episode_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimePreview {
    pub id: String,
    pub title: String,
    pub poster: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub synopsis: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    pub year: Option<i32>,
    pub content_type: ContentType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub genres: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub season_entries: Option<Vec<SeasonEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimeMetadata {
    pub id: String,
    pub title: String,
    pub original_title: Option<String>,
    pub alternative_titles: Vec<String>,
    pub synopsis: Option<String>,
    pub description: Option<String>,
    pub poster: Option<String>,
    pub banner: Option<String>,
    pub year: Option<i32>,
    pub season: Option<String>,
    pub season_year: Option<i32>,
    pub status: Option<String>,
    pub content_type: ContentType,
    pub source: Option<String>,
    pub duration_minutes: Option<u32>,
    pub episode_count: Option<u32>,
    pub score: Option<f64>,
    pub rank: Option<u32>,
    pub popularity: Option<u32>,
    pub rating: Option<String>,
    pub genres: Vec<String>,
    pub tags: Vec<String>,
    pub authors: Vec<String>,
    pub studios: Vec<String>,
    pub staff: Vec<StaffCredit>,
    pub country_of_origin: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub site_url: Option<String>,
    pub trailer_url: Option<String>,
    pub external_links: Vec<ExternalLink>,
    pub episodes: Vec<EpisodeMetadata>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StaffCredit {
    pub name: String,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalLink {
    pub site: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EpisodeMetadata {
    pub id: String,
    pub anime_id: String,
    pub season_number: Option<u32>,
    pub number: u32,
    pub title: Option<String>,
    pub synopsis: Option<String>,
    pub thumbnail: Option<String>,
    pub duration_minutes: Option<u32>,
    pub source: Option<String>,
    pub aired_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecommendationSeed {
    pub anime_id: String,
    pub score: Option<f64>,
    pub weight: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecommendationRequest {
    pub addon_id: Option<String>,
    #[serde(default)]
    pub liked: Vec<RecommendationSeed>,
    #[serde(default)]
    pub disliked: Vec<RecommendationSeed>,
    pub limit: Option<u32>,
    pub candidate_limit: Option<u32>,
    pub include_reasons: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendationPreview {
    #[serde(flatten)]
    pub preview: AnimePreview,
    pub recommendation_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasons: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendationResponse {
    pub items: Vec<RecommendationPreview>,
}

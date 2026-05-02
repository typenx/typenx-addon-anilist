use crate::{api::AniListClient, types::*};
use std::collections::{HashMap, HashSet};

struct ScoredSeed {
    anime: AnimeMetadata,
    weight: f64,
}

struct Profile {
    weights: HashMap<String, f64>,
    preferred_year: Option<f64>,
}

pub async fn recommend_anime(
    request: RecommendationRequest,
    client: &AniListClient,
) -> anyhow::Result<RecommendationResponse> {
    let limit = clamp(request.limit.unwrap_or(24) as f64, 1.0, 50.0) as usize;
    let candidate_limit = clamp(
        request.candidate_limit.unwrap_or((limit * 5) as u32) as f64,
        limit as f64,
        200.0,
    ) as u32;
    let liked = hydrate_seeds(&request.liked, client, 1.0).await?;
    let disliked = hydrate_seeds(&request.disliked, client, -1.0).await?;

    if liked.is_empty() {
        return Ok(RecommendationResponse { items: vec![] });
    }

    let profile = build_profile(&liked, &disliked);
    let seen_ids: HashSet<String> = liked
        .iter()
        .chain(disliked.iter())
        .map(|seed| seed.anime.id.clone())
        .collect();
    let candidates = fetch_candidates(client, candidate_limit).await?;
    let mut scored: Vec<_> = candidates
        .into_iter()
        .filter(|candidate| !seen_ids.contains(&candidate.id))
        .map(|candidate| {
            score_candidate(candidate, &profile, request.include_reasons.unwrap_or(true))
        })
        .collect();
    scored.sort_by(|a, b| b.recommendation_score.total_cmp(&a.recommendation_score));

    Ok(RecommendationResponse {
        items: diversify(scored, limit),
    })
}

async fn hydrate_seeds(
    seeds: &[RecommendationSeed],
    client: &AniListClient,
    polarity: f64,
) -> anyhow::Result<Vec<ScoredSeed>> {
    let mut hydrated = Vec::new();
    for seed in seeds.iter().filter(|seed| !seed.anime_id.is_empty()) {
        hydrated.push(ScoredSeed {
            anime: client.anime(&seed.anime_id).await?,
            weight: seed_weight(seed) * polarity,
        });
    }
    Ok(hydrated)
}

async fn fetch_candidates(client: &AniListClient, limit: u32) -> anyhow::Result<Vec<AnimePreview>> {
    let half = limit.div_ceil(2);
    let trending = client.catalog(CatalogRequest {
        catalog_id: "trending".into(),
        limit: Some(half),
        ..Default::default()
    });
    let popular = client.catalog(CatalogRequest {
        catalog_id: "popular".into(),
        limit: Some(half),
        ..Default::default()
    });
    let (trending, popular) = tokio::try_join!(trending, popular)?;
    Ok(unique_by_id(
        trending.items.into_iter().chain(popular.items).collect(),
    ))
}

pub fn seed_weight(seed: &RecommendationSeed) -> f64 {
    if let Some(weight) = seed.weight {
        return clamp(weight, 0.1, 3.0);
    }
    if let Some(score) = seed.score {
        return clamp((score - 5.0).abs() / 2.5, 0.2, 2.0);
    }
    1.0
}

fn build_profile(liked: &[ScoredSeed], disliked: &[ScoredSeed]) -> Profile {
    let mut weights = HashMap::new();
    let mut years = Vec::new();

    for seed in liked.iter().chain(disliked.iter()) {
        for feature in features_of_metadata(&seed.anime) {
            *weights.entry(feature).or_insert(0.0) += seed.weight;
        }
        if seed.weight > 0.0 {
            if let Some(year) = seed.anime.year {
                years.push(year as f64);
            }
        }
    }

    Profile {
        weights,
        preferred_year: mean(&years),
    }
}

pub fn score_preview_for_test(
    candidate: AnimePreview,
    seed: AnimeMetadata,
) -> RecommendationPreview {
    let profile = build_profile(
        &[ScoredSeed {
            anime: seed,
            weight: 1.0,
        }],
        &[],
    );
    score_candidate(candidate, &profile, true)
}

fn score_candidate(
    candidate: AnimePreview,
    profile: &Profile,
    include_reasons: bool,
) -> RecommendationPreview {
    let features = features_of_preview(&candidate);
    let affinity: f64 = features
        .iter()
        .map(|feature| profile.weights.get(feature).copied().unwrap_or(0.0))
        .sum();
    let normalized_affinity = if features.is_empty() {
        0.0
    } else {
        affinity / (features.len() as f64).sqrt()
    };
    let quality = candidate.score.unwrap_or(0.0) / 10.0;
    let recency = match (profile.preferred_year, candidate.year) {
        (Some(year), Some(candidate_year)) => {
            (1.0 - ((candidate_year as f64 - year).abs() / 20.0)).max(0.0)
        }
        _ => 0.25,
    };
    let score = normalized_affinity * 0.72 + quality * 0.18 + recency * 0.1;
    let reasons = include_reasons.then(|| reasons_for(&candidate, &features, profile));

    RecommendationPreview {
        preview: candidate,
        recommendation_score: (score * 10_000.0).round() / 10_000.0,
        reasons: reasons.filter(|reasons| !reasons.is_empty()),
    }
}

fn diversify(items: Vec<RecommendationPreview>, limit: usize) -> Vec<RecommendationPreview> {
    let mut selected: Vec<RecommendationPreview> = Vec::new();
    for item in items {
        let item_genres: HashSet<_> = item.preview.genres.iter().collect();
        let overlaps = selected
            .iter()
            .filter(|selected_item| {
                selected_item
                    .preview
                    .genres
                    .iter()
                    .any(|genre| item_genres.contains(genre))
            })
            .count();
        if overlaps < 4 || selected.len() < limit.div_ceil(3) {
            selected.push(item);
        }
        if selected.len() >= limit {
            break;
        }
    }
    selected
}

fn features_of_metadata(anime: &AnimeMetadata) -> Vec<String> {
    let mut features = features_base(&anime.genres, &anime.content_type, anime.year);
    features.extend(
        anime
            .tags
            .iter()
            .map(|tag| format!("tag:{}", normalize(tag))),
    );
    features
}

fn features_of_preview(anime: &AnimePreview) -> Vec<String> {
    features_base(&anime.genres, &anime.content_type, anime.year)
}

fn features_base(genres: &[String], content_type: &ContentType, year: Option<i32>) -> Vec<String> {
    let mut features: Vec<_> = genres
        .iter()
        .map(|genre| format!("genre:{}", normalize(genre)))
        .collect();
    features.push(format!("type:{}", content_type_name(content_type)));
    if let Some(year) = year {
        features.push(format!("era:{}", (year / 5) * 5));
    }
    features
}

fn reasons_for(candidate: &AnimePreview, features: &[String], profile: &Profile) -> Vec<String> {
    let mut matches: Vec<_> = features
        .iter()
        .filter(|feature| profile.weights.get(*feature).copied().unwrap_or(0.0) > 0.0)
        .take(3)
        .map(|feature| {
            feature
                .trim_start_matches("genre:")
                .trim_start_matches("tag:")
                .trim_start_matches("type:")
                .trim_start_matches("era:")
                .replace('-', " ")
        })
        .collect();
    if candidate.score.is_some_and(|score| score >= 8.0) {
        matches.push("strong community score".into());
    }
    matches.truncate(4);
    matches
}

fn normalize(value: &str) -> String {
    let mut output = String::new();
    let mut last_dash = false;
    for ch in value.trim().to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
            last_dash = false;
        } else if !last_dash && !output.is_empty() {
            output.push('-');
            last_dash = true;
        }
    }
    output.trim_matches('-').to_string()
}

fn content_type_name(content_type: &ContentType) -> &'static str {
    match content_type {
        ContentType::Anime => "anime",
        ContentType::Manga => "manga",
        ContentType::Manhwa => "manhwa",
        ContentType::Manhua => "manhua",
        ContentType::LightNovel => "light_novel",
        ContentType::Movie => "movie",
        ContentType::Ova => "ova",
        ContentType::Ona => "ona",
        ContentType::Special => "special",
    }
}

fn unique_by_id(items: Vec<AnimePreview>) -> Vec<AnimePreview> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.id.clone()))
        .collect()
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preview(id: &str, genres: &[&str], year: i32, score: f64) -> AnimePreview {
        AnimePreview {
            id: id.into(),
            title: id.into(),
            poster: None,
            banner: None,
            synopsis: None,
            score: Some(score),
            year: Some(year),
            content_type: ContentType::Anime,
            genres: genres.iter().map(|value| value.to_string()).collect(),
            season_entries: None,
        }
    }

    #[test]
    fn seed_weight_matches_typescript_clamps() {
        assert_eq!(
            seed_weight(&RecommendationSeed {
                anime_id: "1".into(),
                score: None,
                weight: Some(9.0)
            }),
            3.0
        );
        assert_eq!(
            seed_weight(&RecommendationSeed {
                anime_id: "1".into(),
                score: Some(10.0),
                weight: None
            }),
            2.0
        );
        assert_eq!(
            seed_weight(&RecommendationSeed {
                anime_id: "1".into(),
                score: Some(5.0),
                weight: None
            }),
            0.2
        );
    }

    #[test]
    fn matching_genres_and_era_score_higher() {
        let seed = AnimeMetadata {
            id: "seed".into(),
            title: "Seed".into(),
            original_title: None,
            alternative_titles: vec![],
            synopsis: None,
            description: None,
            poster: None,
            banner: None,
            year: Some(2021),
            season: None,
            season_year: Some(2021),
            status: None,
            content_type: ContentType::Anime,
            source: None,
            duration_minutes: Some(24),
            episode_count: None,
            score: None,
            rank: None,
            popularity: None,
            rating: None,
            genres: vec!["Action".into(), "Fantasy".into()],
            tags: vec!["Magic".into()],
            authors: vec![],
            studios: vec![],
            staff: vec![],
            country_of_origin: None,
            start_date: None,
            end_date: None,
            site_url: None,
            trailer_url: None,
            external_links: vec![],
            episodes: vec![],
            updated_at: None,
        };

        let close = score_preview_for_test(
            preview("close", &["Action", "Fantasy"], 2020, 7.0),
            seed.clone(),
        );
        let far = score_preview_for_test(preview("far", &["Slice of Life"], 1980, 10.0), seed);
        assert!(close.recommendation_score > far.recommendation_score);
        assert!(close.reasons.unwrap().contains(&"action".into()));
    }
}

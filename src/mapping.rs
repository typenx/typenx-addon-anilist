use crate::types::*;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AniListMedia {
    pub id: i64,
    pub id_mal: Option<i64>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub title: Option<AniListTitle>,
    pub synonyms: Option<Vec<String>>,
    pub description: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub season: Option<String>,
    pub season_year: Option<i32>,
    pub episodes: Option<u32>,
    pub chapters: Option<u32>,
    pub volumes: Option<u32>,
    pub duration: Option<u32>,
    pub source: Option<String>,
    pub country_of_origin: Option<String>,
    pub start_date: Option<AniListFuzzyDate>,
    pub end_date: Option<AniListFuzzyDate>,
    pub genres: Option<Vec<String>>,
    pub tags: Option<Vec<AniListTag>>,
    pub average_score: Option<f64>,
    pub mean_score: Option<f64>,
    pub popularity: Option<u32>,
    pub rankings: Option<Vec<AniListRanking>>,
    pub updated_at: Option<i64>,
    pub site_url: Option<String>,
    pub trailer: Option<AniListTrailer>,
    pub external_links: Option<Vec<AniListExternalLink>>,
    pub cover_image: Option<AniListCoverImage>,
    pub banner_image: Option<String>,
    pub studios: Option<AniListStudios>,
    pub staff: Option<AniListStaff>,
    pub streaming_episodes: Option<Vec<AniListStreamingEpisode>>,
    pub airing_schedule: Option<AniListAiringSchedule>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AniListCoverImage {
    pub extra_large: Option<String>,
    pub large: Option<String>,
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListFuzzyDate {
    pub year: Option<i32>,
    pub month: Option<u32>,
    pub day: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AniListTag {
    pub name: Option<String>,
    pub rank: Option<i32>,
    pub is_media_spoiler: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AniListRanking {
    pub rank: Option<u32>,
    #[serde(rename = "type")]
    pub ranking_type: Option<String>,
    pub all_time: Option<bool>,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AniListStudio {
    pub name: Option<String>,
    pub is_animation_studio: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListStudios {
    pub nodes: Option<Vec<AniListStudio>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListStaff {
    pub edges: Option<Vec<AniListStaffEdge>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListStaffEdge {
    pub role: Option<String>,
    pub node: Option<AniListStaffNode>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListStaffNode {
    pub name: Option<AniListStaffName>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListStaffName {
    pub full: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListStreamingEpisode {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub site: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListAiringSchedule {
    pub nodes: Option<Vec<AniListAiringEpisode>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AniListAiringEpisode {
    pub episode: Option<u32>,
    pub airing_at: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListTrailer {
    pub id: Option<String>,
    pub site: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AniListExternalLink {
    pub site: Option<String>,
    pub url: Option<String>,
}

pub fn to_preview(media: &AniListMedia) -> AnimePreview {
    AnimePreview {
        id: media.id.to_string(),
        title: title_of(media),
        poster: image_of(media),
        banner: media.banner_image.clone(),
        synopsis: clean_description(media.description.as_deref()),
        score: score_of(media),
        year: media
            .season_year
            .or(media.start_date.as_ref().and_then(|d| d.year)),
        content_type: content_type_of(media),
        genres: media.genres.clone().unwrap_or_default(),
        season_entries: None,
    }
}

pub fn to_metadata(media: &AniListMedia) -> AnimeMetadata {
    let description = clean_description(media.description.as_deref());
    let studios = studios_of(media);
    let is_manga = is_manga_media(media);
    AnimeMetadata {
        id: media.id.to_string(),
        title: title_of(media),
        original_title: media.title.as_ref().and_then(|title| title.native.clone()),
        alternative_titles: alternative_titles_of(media),
        synopsis: description.clone(),
        description,
        poster: image_of(media),
        banner: media.banner_image.clone().or_else(|| image_of(media)),
        year: media
            .season_year
            .or(media.start_date.as_ref().and_then(|d| d.year)),
        season: if is_manga {
            None
        } else {
            media.season.as_ref().map(|value| value.to_lowercase())
        },
        season_year: if is_manga {
            media.start_date.as_ref().and_then(|d| d.year)
        } else {
            media
                .season_year
                .or(media.start_date.as_ref().and_then(|d| d.year))
        },
        status: media.status.as_ref().map(|value| value.to_lowercase()),
        content_type: content_type_of(media),
        source: media.source.as_ref().map(|value| value.to_lowercase()),
        duration_minutes: if is_manga { None } else { media.duration },
        episode_count: if is_manga {
            positive_number(media.chapters).or_else(|| positive_number(media.volumes))
        } else {
            media.episodes
        },
        score: score_of(media),
        rank: rank_of(media),
        popularity: media.popularity,
        rating: None,
        genres: media.genres.clone().unwrap_or_default(),
        tags: tags_of(media),
        authors: authors_of(media, &studios),
        studios,
        staff: staff_of(media),
        country_of_origin: media.country_of_origin.clone(),
        start_date: fuzzy_date(media.start_date.as_ref()),
        end_date: fuzzy_date(media.end_date.as_ref()),
        site_url: media.site_url.clone(),
        trailer_url: trailer_url(media.trailer.as_ref()),
        external_links: external_links_of(media),
        episodes: if is_manga { vec![] } else { episodes_of(media) },
        updated_at: Some(
            media
                .updated_at
                .and_then(|ts| DateTime::from_timestamp(ts, 0))
                .unwrap_or_else(Utc::now)
                .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        ),
    }
}

fn title_of(media: &AniListMedia) -> String {
    media
        .title
        .as_ref()
        .and_then(|title| title.english.clone())
        .or_else(|| media.title.as_ref().and_then(|title| title.romaji.clone()))
        .or_else(|| media.title.as_ref().and_then(|title| title.native.clone()))
        .unwrap_or_else(|| media.id.to_string())
}

fn alternative_titles_of(media: &AniListMedia) -> Vec<String> {
    let primary = title_of(media);
    unique_strings(
        [
            media.title.as_ref().and_then(|title| title.romaji.clone()),
            media.title.as_ref().and_then(|title| title.english.clone()),
            media.title.as_ref().and_then(|title| title.native.clone()),
        ]
        .into_iter()
        .chain(
            media
                .synonyms
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(Some),
        )
        .collect(),
    )
    .into_iter()
    .filter(|title| title != &primary)
    .collect()
}

fn image_of(media: &AniListMedia) -> Option<String> {
    media.cover_image.as_ref().and_then(|image| {
        image
            .extra_large
            .clone()
            .or_else(|| image.large.clone())
            .or_else(|| image.medium.clone())
    })
}

fn clean_description(description: Option<&str>) -> Option<String> {
    let value = description?
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("<i>", "")
        .replace("</i>", "")
        .trim()
        .to_string();
    (!value.is_empty()).then_some(value)
}

fn is_manga_media(media: &AniListMedia) -> bool {
    media.media_type.as_deref() == Some("MANGA")
}

fn content_type_of(media: &AniListMedia) -> ContentType {
    if is_manga_media(media) {
        return if media.format.as_deref() == Some("NOVEL") {
            ContentType::LightNovel
        } else {
            ContentType::Manga
        };
    }
    match media.format.as_deref() {
        Some("MOVIE") => ContentType::Movie,
        Some("OVA") => ContentType::Ova,
        Some("ONA") => ContentType::Ona,
        Some("SPECIAL") => ContentType::Special,
        _ => ContentType::Anime,
    }
}

fn score_of(media: &AniListMedia) -> Option<f64> {
    media
        .average_score
        .or(media.mean_score)
        .map(|score| score / 10.0)
}

fn positive_number(value: Option<u32>) -> Option<u32> {
    value.filter(|value| *value > 0)
}

fn rank_of(media: &AniListMedia) -> Option<u32> {
    let rankings = media.rankings.as_ref()?;
    rankings
        .iter()
        .find(|ranking| {
            ranking.ranking_type.as_deref() == Some("RATED") && ranking.all_time == Some(true)
        })
        .and_then(|ranking| ranking.rank)
        .or_else(|| {
            rankings
                .iter()
                .find(|ranking| ranking.ranking_type.as_deref() == Some("RATED"))
                .and_then(|ranking| ranking.rank)
        })
}

fn tags_of(media: &AniListMedia) -> Vec<String> {
    let mut tags = media.tags.clone().unwrap_or_default();
    tags.retain(|tag| tag.is_media_spoiler != Some(true));
    tags.sort_by_key(|tag| std::cmp::Reverse(tag.rank.unwrap_or(0)));
    unique_strings(tags.into_iter().take(12).map(|tag| tag.name).collect())
}

fn studios_of(media: &AniListMedia) -> Vec<String> {
    let nodes = media
        .studios
        .as_ref()
        .and_then(|s| s.nodes.clone())
        .unwrap_or_default();
    let animation: Vec<_> = nodes
        .iter()
        .filter(|studio| studio.is_animation_studio == Some(true))
        .cloned()
        .collect();
    let selected = if animation.is_empty() {
        nodes
    } else {
        animation
    };
    unique_strings(selected.into_iter().map(|studio| studio.name).collect())
}

fn staff_of(media: &AniListMedia) -> Vec<StaffCredit> {
    media
        .staff
        .as_ref()
        .and_then(|staff| staff.edges.as_ref())
        .into_iter()
        .flatten()
        .filter_map(|edge| {
            let name = edge.node.as_ref()?.name.as_ref()?.full.clone()?;
            (!name.is_empty()).then_some(StaffCredit {
                name,
                role: edge.role.clone(),
            })
        })
        .collect()
}

fn authors_of(media: &AniListMedia, studios: &[String]) -> Vec<String> {
    let author_roles = [
        "Original Creator",
        "Original Story",
        "Story",
        "Director",
        "Series Composition",
    ];
    let staff_authors: Vec<_> = staff_of(media)
        .into_iter()
        .filter(|credit| {
            credit
                .role
                .as_ref()
                .is_some_and(|role| author_roles.iter().any(|needle| role.contains(needle)))
        })
        .map(|credit| Some(credit.name))
        .collect();
    if staff_authors.is_empty() {
        studios.to_vec()
    } else {
        unique_strings(staff_authors)
    }
}

fn external_links_of(media: &AniListMedia) -> Vec<ExternalLink> {
    media
        .external_links
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|link| {
            Some(ExternalLink {
                site: link.site?,
                url: link.url?,
            })
        })
        .collect()
}

fn trailer_url(trailer: Option<&AniListTrailer>) -> Option<String> {
    let trailer = trailer?;
    match (trailer.site.as_deref(), trailer.id.as_deref()) {
        (Some("youtube"), Some(id)) => Some(format!("https://www.youtube.com/watch?v={id}")),
        (Some("dailymotion"), Some(id)) => Some(format!("https://www.dailymotion.com/video/{id}")),
        _ => None,
    }
}

fn episodes_of(media: &AniListMedia) -> Vec<EpisodeMetadata> {
    let streaming = media.streaming_episodes.clone().unwrap_or_default();
    let schedule: HashMap<u32, Option<i64>> = media
        .airing_schedule
        .as_ref()
        .and_then(|schedule| schedule.nodes.as_ref())
        .into_iter()
        .flatten()
        .filter_map(|episode| Some((episode.episode?, episode.airing_at)))
        .collect();
    let count = media
        .episodes
        .unwrap_or(0)
        .max(streaming.len() as u32)
        .max(schedule.keys().copied().max().unwrap_or(0));
    (1..=count)
        .map(|number| {
            let streaming_episode = streaming.get((number - 1) as usize);
            EpisodeMetadata {
                id: format!("{}:{number}", media.id),
                anime_id: media.id.to_string(),
                season_number: None,
                number,
                title: streaming_episode
                    .and_then(|ep| ep.title.clone())
                    .or_else(|| Some(format!("Episode {number}"))),
                synopsis: None,
                thumbnail: streaming_episode.and_then(|ep| ep.thumbnail.clone()),
                duration_minutes: media.duration,
                source: streaming_episode.and_then(|ep| ep.site.clone()),
                aired_at: schedule
                    .get(&number)
                    .copied()
                    .flatten()
                    .and_then(|ts| DateTime::from_timestamp(ts, 0))
                    .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)),
            }
        })
        .collect()
}

fn fuzzy_date(date: Option<&AniListFuzzyDate>) -> Option<String> {
    let date = date?;
    Some(format!(
        "{:04}-{:02}-{:02}",
        date.year?,
        date.month.unwrap_or(1),
        date.day.unwrap_or(1)
    ))
}

fn unique_strings(values: Vec<Option<String>>) -> Vec<String> {
    let mut seen = HashSet::new();
    values
        .into_iter()
        .filter_map(|value| value.map(|value| value.trim().to_string()))
        .filter(|value| !value.is_empty() && seen.insert(value.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_manga_metadata_without_episodes() {
        let media = AniListMedia {
            id: 42,
            media_type: Some("MANGA".into()),
            title: Some(AniListTitle {
                english: Some("Frieren".into()),
                romaji: Some("Sousou no Frieren".into()),
                native: Some("葬送のフリーレン".into()),
            }),
            format: Some("MANGA".into()),
            chapters: Some(130),
            volumes: Some(13),
            season: Some("SPRING".into()),
            start_date: Some(AniListFuzzyDate {
                year: Some(2020),
                month: None,
                day: None,
            }),
            genres: Some(vec!["Fantasy".into()]),
            tags: Some(vec![AniListTag {
                name: Some("Magic".into()),
                rank: Some(90),
                is_media_spoiler: Some(false),
            }]),
            ..Default::default()
        };

        let mapped = to_metadata(&media);
        assert_eq!(mapped.content_type, ContentType::Manga);
        assert_eq!(mapped.episode_count, Some(130));
        assert_eq!(mapped.season, None);
        assert_eq!(mapped.season_year, Some(2020));
        assert!(mapped.episodes.is_empty());
        assert_eq!(
            mapped.alternative_titles,
            vec!["Sousou no Frieren", "葬送のフリーレン"]
        );
    }

    #[test]
    fn maps_anime_episode_schedule_and_links() {
        let media = AniListMedia {
            id: 7,
            media_type: Some("ANIME".into()),
            title: Some(AniListTitle {
                english: None,
                romaji: Some("Test Show".into()),
                native: None,
            }),
            episodes: Some(2),
            duration: Some(24),
            trailer: Some(AniListTrailer {
                id: Some("abc".into()),
                site: Some("youtube".into()),
            }),
            streaming_episodes: Some(vec![AniListStreamingEpisode {
                title: Some("Pilot".into()),
                thumbnail: Some("img".into()),
                site: Some("Crunchyroll".into()),
            }]),
            airing_schedule: Some(AniListAiringSchedule {
                nodes: Some(vec![AniListAiringEpisode {
                    episode: Some(2),
                    airing_at: Some(1_700_000_000),
                }]),
            }),
            ..Default::default()
        };

        let mapped = to_metadata(&media);
        assert_eq!(
            mapped.trailer_url,
            Some("https://www.youtube.com/watch?v=abc".into())
        );
        assert_eq!(mapped.episodes.len(), 2);
        assert_eq!(mapped.episodes[0].title, Some("Pilot".into()));
        assert_eq!(mapped.episodes[1].title, Some("Episode 2".into()));
        assert_eq!(
            mapped.episodes[1].aired_at,
            Some("2023-11-14T22:13:20.000Z".into())
        );
    }
}

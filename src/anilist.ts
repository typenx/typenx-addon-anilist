import type {
  AnimeMetadata,
  AnimePreview,
  CatalogRequest,
  CatalogResponse,
  ContentType,
  ExternalLink,
  SearchRequest,
  StaffCredit,
} from '@typenx/addon-ts-sdk'

const API_URL = 'https://graphql.anilist.co'

const MEDIA_FIELDS = `
  id
  idMal
  title {
    romaji
    english
    native
  }
  synonyms
  description(asHtml: false)
  format
  status
  season
  seasonYear
  episodes
  duration
  source(version: 3)
  countryOfOrigin
  startDate {
    year
    month
    day
  }
  endDate {
    year
    month
    day
  }
  genres
  tags {
    name
    rank
    isMediaSpoiler
  }
  averageScore
  meanScore
  popularity
  rankings {
    rank
    type
    allTime
    context
  }
  updatedAt
  siteUrl
  trailer {
    id
    site
  }
  externalLinks {
    site
    url
  }
  coverImage {
    extraLarge
    large
    medium
  }
  bannerImage
  studios {
    nodes {
      name
      isAnimationStudio
    }
  }
  staff(perPage: 12, sort: RELEVANCE) {
    edges {
      role
      node {
        name {
          full
        }
      }
    }
  }
  streamingEpisodes {
    title
    thumbnail
    site
  }
  airingSchedule(notYetAired: false, perPage: 25) {
    nodes {
      episode
      airingAt
    }
  }
`

const CATALOG_QUERY = `
  query TypenxCatalog($page: Int!, $perPage: Int!, $sort: [MediaSort], $status: MediaStatus) {
    Page(page: $page, perPage: $perPage) {
      media(type: ANIME, isAdult: false, sort: $sort, status: $status) {
        ${MEDIA_FIELDS}
      }
    }
  }
`

const SEARCH_QUERY = `
  query TypenxSearch($page: Int!, $perPage: Int!, $search: String!) {
    Page(page: $page, perPage: $perPage) {
      media(type: ANIME, isAdult: false, search: $search, sort: SEARCH_MATCH) {
        ${MEDIA_FIELDS}
      }
    }
  }
`

const ANIME_QUERY = `
  query TypenxAnime($id: Int!) {
    Media(id: $id, type: ANIME) {
      ${MEDIA_FIELDS}
    }
  }
`

type AniListTitle = {
  romaji?: string | null
  english?: string | null
  native?: string | null
}

type AniListCoverImage = {
  extraLarge?: string | null
  large?: string | null
  medium?: string | null
}

type AniListFuzzyDate = {
  year?: number | null
  month?: number | null
  day?: number | null
}

type AniListTag = {
  name?: string | null
  rank?: number | null
  isMediaSpoiler?: boolean | null
}

type AniListRanking = {
  rank?: number | null
  type?: string | null
  allTime?: boolean | null
  context?: string | null
}

type AniListStudio = {
  name?: string | null
  isAnimationStudio?: boolean | null
}

type AniListStaffEdge = {
  role?: string | null
  node?: {
    name?: {
      full?: string | null
    } | null
  } | null
}

type AniListStreamingEpisode = {
  title?: string | null
  thumbnail?: string | null
  site?: string | null
}

type AniListAiringEpisode = {
  episode?: number | null
  airingAt?: number | null
}

type AniListTrailer = {
  id?: string | null
  site?: string | null
}

type AniListExternalLink = {
  site?: string | null
  url?: string | null
}

type AniListMedia = {
  id: number
  idMal?: number | null
  title?: AniListTitle | null
  synonyms?: string[] | null
  description?: string | null
  format?: string | null
  status?: string | null
  season?: string | null
  seasonYear?: number | null
  episodes?: number | null
  duration?: number | null
  source?: string | null
  countryOfOrigin?: string | null
  startDate?: AniListFuzzyDate | null
  endDate?: AniListFuzzyDate | null
  genres?: string[] | null
  tags?: AniListTag[] | null
  averageScore?: number | null
  meanScore?: number | null
  popularity?: number | null
  rankings?: AniListRanking[] | null
  updatedAt?: number | null
  siteUrl?: string | null
  trailer?: AniListTrailer | null
  externalLinks?: AniListExternalLink[] | null
  coverImage?: AniListCoverImage | null
  bannerImage?: string | null
  studios?: { nodes?: AniListStudio[] | null } | null
  staff?: { edges?: AniListStaffEdge[] | null } | null
  streamingEpisodes?: AniListStreamingEpisode[] | null
  airingSchedule?: { nodes?: AniListAiringEpisode[] | null } | null
}

type GraphQlResponse<T> = {
  data?: T
  errors?: Array<{ message?: string }>
}

export class AniListCatalog {
  async catalog(request: CatalogRequest): Promise<CatalogResponse> {
    const limit = clampLimit(request.limit)
    if (request.query) {
      return this.search({ query: request.query, limit })
    }

    const variables = {
      page: skipToPage(request.skip, limit),
      perPage: limit,
      ...catalogVariables(request.catalog_id),
    }
    const data = await this.graphql<{ Page: { media: AniListMedia[] } }>(
      CATALOG_QUERY,
      variables,
    )
    return { items: data.Page.media.map(toPreview) }
  }

  async search(request: SearchRequest): Promise<CatalogResponse> {
    const limit = clampLimit(request.limit)
    const data = await this.graphql<{ Page: { media: AniListMedia[] } }>(
      SEARCH_QUERY,
      {
        page: 1,
        perPage: limit,
        search: request.query,
      },
    )
    return { items: data.Page.media.map(toPreview) }
  }

  async anime(id: string): Promise<AnimeMetadata> {
    const numericId = Number(id)
    if (!Number.isInteger(numericId) || numericId <= 0) {
      throw new Error(`Invalid AniList anime id: ${id}`)
    }

    const data = await this.graphql<{ Media: AniListMedia | null }>(ANIME_QUERY, {
      id: numericId,
    })
    if (!data.Media) {
      throw new Error(`AniList anime not found: ${id}`)
    }
    return toMetadata(data.Media)
  }

  private async graphql<T>(query: string, variables: Record<string, unknown>): Promise<T> {
    const response = await fetch(API_URL, {
      method: 'POST',
      headers: {
        accept: 'application/json',
        'content-type': 'application/json',
      },
      body: JSON.stringify({ query, variables }),
    })
    const body = (await response.json().catch(() => null)) as GraphQlResponse<T> | null

    if (!response.ok) {
      throw new Error(`AniList returned ${response.status}`)
    }
    if (!body) {
      throw new Error('AniList returned an empty response')
    }
    if (body.errors?.length) {
      throw new Error(body.errors.map((error) => error.message).filter(Boolean).join('; '))
    }
    if (!body.data) {
      throw new Error('AniList response did not include data')
    }
    return body.data
  }
}

function catalogVariables(catalogId: string) {
  if (catalogId === 'trending') {
    return { sort: ['TRENDING_DESC'] }
  }
  if (catalogId === 'airing') {
    return { sort: ['POPULARITY_DESC'], status: 'RELEASING' }
  }
  return { sort: ['POPULARITY_DESC'] }
}

function toPreview(media: AniListMedia): AnimePreview {
  return {
    id: String(media.id),
    title: titleOf(media),
    poster: imageOf(media),
    banner: media.bannerImage ?? null,
    synopsis: cleanDescription(media.description),
    score: scoreOf(media),
    year: media.seasonYear ?? media.startDate?.year ?? null,
    content_type: contentTypeOf(media.format),
  }
}

function toMetadata(media: AniListMedia): AnimeMetadata {
  const description = cleanDescription(media.description)
  const studios = studiosOf(media)
  return {
    id: String(media.id),
    title: titleOf(media),
    original_title: media.title?.native ?? null,
    alternative_titles: alternativeTitlesOf(media),
    synopsis: description,
    description,
    poster: imageOf(media),
    banner: media.bannerImage ?? imageOf(media),
    year: media.seasonYear ?? media.startDate?.year ?? null,
    season: media.season?.toLowerCase() ?? null,
    season_year: media.seasonYear ?? media.startDate?.year ?? null,
    status: media.status?.toLowerCase() ?? null,
    content_type: contentTypeOf(media.format),
    source: media.source?.toLowerCase() ?? null,
    duration_minutes: media.duration ?? null,
    episode_count: media.episodes ?? null,
    score: scoreOf(media),
    rank: rankOf(media),
    popularity: media.popularity ?? null,
    rating: null,
    genres: media.genres ?? [],
    tags: tagsOf(media),
    authors: authorsOf(media, studios),
    studios,
    staff: staffOf(media),
    country_of_origin: media.countryOfOrigin ?? null,
    start_date: fuzzyDate(media.startDate),
    end_date: fuzzyDate(media.endDate),
    site_url: media.siteUrl ?? null,
    trailer_url: trailerUrl(media.trailer),
    external_links: externalLinksOf(media),
    episodes: episodesOf(media),
    updated_at: media.updatedAt
      ? new Date(media.updatedAt * 1000).toISOString()
      : new Date().toISOString(),
  }
}

function titleOf(media: AniListMedia) {
  return media.title?.english ?? media.title?.romaji ?? media.title?.native ?? String(media.id)
}

function alternativeTitlesOf(media: AniListMedia) {
  return uniqueStrings([
    media.title?.romaji,
    media.title?.english,
    media.title?.native,
    ...(media.synonyms ?? []),
  ]).filter((title) => title !== titleOf(media))
}

function imageOf(media: AniListMedia) {
  return (
    media.coverImage?.extraLarge ??
    media.coverImage?.large ??
    media.coverImage?.medium ??
    null
  )
}

function cleanDescription(description: string | null | undefined) {
  return description
    ?.replace(/<br\s*\/?>/gi, '\n')
    .replace(/<\/?i>/gi, '')
    .trim() || null
}

function contentTypeOf(format: string | null | undefined): ContentType {
  if (format === 'MOVIE') return 'movie'
  if (format === 'OVA') return 'ova'
  if (format === 'ONA') return 'ona'
  if (format === 'SPECIAL') return 'special'
  return 'anime'
}

function scoreOf(media: AniListMedia) {
  const score = media.averageScore ?? media.meanScore
  return typeof score === 'number' ? score / 10 : null
}

function rankOf(media: AniListMedia) {
  return (
    media.rankings?.find((ranking) => ranking.type === 'RATED' && ranking.allTime)
      ?.rank ??
    media.rankings?.find((ranking) => ranking.type === 'RATED')?.rank ??
    null
  )
}

function tagsOf(media: AniListMedia) {
  return uniqueStrings(
    (media.tags ?? [])
      .filter((tag) => !tag.isMediaSpoiler)
      .sort((a, b) => (b.rank ?? 0) - (a.rank ?? 0))
      .slice(0, 12)
      .map((tag) => tag.name),
  )
}

function studiosOf(media: AniListMedia) {
  const nodes = media.studios?.nodes ?? []
  const animationStudios = nodes.filter((studio) => studio.isAnimationStudio)
  return uniqueStrings((animationStudios.length ? animationStudios : nodes).map((studio) => studio.name))
}

function staffOf(media: AniListMedia): StaffCredit[] {
  return (media.staff?.edges ?? [])
    .map((edge) => ({
      name: edge.node?.name?.full ?? '',
      role: edge.role ?? null,
    }))
    .filter((credit) => credit.name)
}

function authorsOf(media: AniListMedia, studios: string[]) {
  const authorRoles = ['Original Creator', 'Original Story', 'Story', 'Director', 'Series Composition']
  const staffAuthors = staffOf(media)
    .filter((credit) => credit.role && authorRoles.some((role) => credit.role?.includes(role)))
    .map((credit) => credit.name)
  return uniqueStrings(staffAuthors.length ? staffAuthors : studios)
}

function externalLinksOf(media: AniListMedia): ExternalLink[] {
  return (media.externalLinks ?? [])
    .filter((link): link is { site: string; url: string } => !!link.site && !!link.url)
    .map((link) => ({ site: link.site, url: link.url }))
}

function trailerUrl(trailer: AniListTrailer | null | undefined) {
  if (!trailer?.id || !trailer.site) return null
  if (trailer.site === 'youtube') return `https://www.youtube.com/watch?v=${trailer.id}`
  if (trailer.site === 'dailymotion') return `https://www.dailymotion.com/video/${trailer.id}`
  return null
}

function episodesOf(media: AniListMedia) {
  const streaming = media.streamingEpisodes ?? []
  const schedule = new Map(
    (media.airingSchedule?.nodes ?? [])
      .filter((episode) => episode.episode)
      .map((episode) => [episode.episode as number, episode.airingAt ?? null]),
  )
  const count = Math.max(
    media.episodes ?? 0,
    streaming.length,
    ...Array.from(schedule.keys()),
  )

  return Array.from({ length: count }, (_, index) => {
    const number = index + 1
    const streamingEpisode = streaming[index]
    const airedAt = schedule.get(number)
    return {
      id: `${media.id}:${number}`,
      anime_id: String(media.id),
      season_number: null,
      number,
      title: streamingEpisode?.title ?? `Episode ${number}`,
      synopsis: null,
      thumbnail: streamingEpisode?.thumbnail ?? null,
      duration_minutes: media.duration ?? null,
      source: streamingEpisode?.site ?? null,
      aired_at: airedAt ? new Date(airedAt * 1000).toISOString() : null,
    }
  })
}

function fuzzyDate(date: AniListFuzzyDate | null | undefined) {
  if (!date?.year) return null
  const month = String(date.month ?? 1).padStart(2, '0')
  const day = String(date.day ?? 1).padStart(2, '0')
  return `${date.year}-${month}-${day}`
}

function uniqueStrings(values: Array<string | null | undefined>) {
  return Array.from(
    new Set(values.map((value) => value?.trim()).filter((value): value is string => !!value)),
  )
}

function clampLimit(limit: number | undefined) {
  return Math.min(Math.max(limit ?? 24, 1), 50)
}

function skipToPage(skip: number | undefined, limit: number) {
  return Math.floor((skip ?? 0) / limit) + 1
}

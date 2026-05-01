import type {
  AnimeMetadata,
  AnimePreview,
  CatalogRequest,
  CatalogResponse,
  ContentType,
  SearchRequest,
} from '@typenx/addon-ts-sdk'

const API_URL = 'https://graphql.anilist.co'

const MEDIA_FIELDS = `
  id
  title {
    romaji
    english
    native
  }
  description(asHtml: false)
  format
  status
  seasonYear
  episodes
  genres
  updatedAt
  coverImage {
    extraLarge
    large
    medium
  }
  bannerImage
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

type AniListMedia = {
  id: number
  title?: AniListTitle | null
  description?: string | null
  format?: string | null
  status?: string | null
  seasonYear?: number | null
  episodes?: number | null
  genres?: string[] | null
  updatedAt?: number | null
  coverImage?: AniListCoverImage | null
  bannerImage?: string | null
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
    year: media.seasonYear ?? null,
    content_type: contentTypeOf(media.format),
  }
}

function toMetadata(media: AniListMedia): AnimeMetadata {
  return {
    id: String(media.id),
    title: titleOf(media),
    original_title: media.title?.native ?? null,
    synopsis: cleanDescription(media.description),
    poster: imageOf(media),
    banner: media.bannerImage ?? imageOf(media),
    year: media.seasonYear ?? null,
    status: media.status?.toLowerCase() ?? null,
    genres: media.genres ?? [],
    episodes: createEpisodes(String(media.id), media.episodes ?? 0),
    updated_at: media.updatedAt
      ? new Date(media.updatedAt * 1000).toISOString()
      : new Date().toISOString(),
  }
}

function titleOf(media: AniListMedia) {
  return media.title?.english ?? media.title?.romaji ?? media.title?.native ?? String(media.id)
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
  return description?.replace(/<br\s*\/?>/gi, '\n').trim() || null
}

function contentTypeOf(format: string | null | undefined): ContentType {
  if (format === 'MOVIE') return 'movie'
  if (format === 'OVA') return 'ova'
  if (format === 'ONA') return 'ona'
  if (format === 'SPECIAL') return 'special'
  return 'anime'
}

function createEpisodes(animeId: string, count: number) {
  return Array.from({ length: count }, (_, index) => {
    const number = index + 1
    return {
      id: `${animeId}:${number}`,
      anime_id: animeId,
      number,
      title: null,
      synopsis: null,
      thumbnail: null,
      aired_at: null,
    }
  })
}

function clampLimit(limit: number | undefined) {
  return Math.min(Math.max(limit ?? 24, 1), 50)
}

function skipToPage(skip: number | undefined, limit: number) {
  return Math.floor((skip ?? 0) / limit) + 1
}

import type {
  AnimeMetadata,
  AnimePreview,
  RecommendationPreview,
  RecommendationRequest,
  RecommendationResponse,
  RecommendationSeed,
} from '@typenx/addon-ts-sdk'

type ScoredSeed = {
  anime: AnimeMetadata
  weight: number
}

export async function recommendAnime(
  request: RecommendationRequest,
  options: {
    fetchAnime: (id: string) => Promise<AnimeMetadata>
    fetchCandidates: (limit: number) => Promise<AnimePreview[]>
  },
): Promise<RecommendationResponse> {
  const limit = clamp(request.limit ?? 24, 1, 50)
  const candidateLimit = clamp(request.candidate_limit ?? limit * 5, limit, 200)
  const liked = await hydrateSeeds(request.liked ?? [], options.fetchAnime, 1)
  const disliked = await hydrateSeeds(request.disliked ?? [], options.fetchAnime, -1)

  if (liked.length === 0) {
    return { items: [] }
  }

  const profile = buildProfile(liked, disliked)
  const seenIds = new Set([...liked, ...disliked].map((seed) => seed.anime.id))
  const candidates = await options.fetchCandidates(candidateLimit)
  const scored = candidates
    .filter((candidate) => !seenIds.has(candidate.id))
    .map((candidate) => scoreCandidate(candidate, profile, request.include_reasons ?? true))
    .sort((a, b) => b.recommendation_score - a.recommendation_score)

  return { items: diversify(scored, limit) }
}

async function hydrateSeeds(
  seeds: RecommendationSeed[],
  fetchAnime: (id: string) => Promise<AnimeMetadata>,
  polarity: 1 | -1,
) {
  const hydrated = await Promise.all(
    seeds
      .filter((seed) => seed.anime_id)
      .map(async (seed) => ({
        anime: await fetchAnime(seed.anime_id),
        weight: seedWeight(seed) * polarity,
      })),
  )
  return hydrated
}

function seedWeight(seed: RecommendationSeed) {
  if (typeof seed.weight === 'number') return clamp(seed.weight, 0.1, 3)
  if (typeof seed.score === 'number') return clamp(Math.abs(seed.score - 5) / 2.5, 0.2, 2)
  return 1
}

function buildProfile(liked: ScoredSeed[], disliked: ScoredSeed[]) {
  const weights = new Map<string, number>()
  const years: number[] = []
  const durations: number[] = []
  const allSeeds = [...liked, ...disliked]

  for (const seed of allSeeds) {
    const features = featuresOf(seed.anime)
    for (const feature of features) {
      weights.set(feature, (weights.get(feature) ?? 0) + seed.weight)
    }
    if (seed.weight > 0 && seed.anime.year) years.push(seed.anime.year)
    if (seed.weight > 0 && seed.anime.duration_minutes) durations.push(seed.anime.duration_minutes)
  }

  return {
    weights,
    preferredYear: mean(years),
    preferredDuration: mean(durations),
  }
}

function scoreCandidate(
  candidate: AnimePreview,
  profile: ReturnType<typeof buildProfile>,
  includeReasons: boolean,
): RecommendationPreview {
  const features = featuresOf(candidate)
  const affinity = features.reduce((total, feature) => total + (profile.weights.get(feature) ?? 0), 0)
  const normalizedAffinity = features.length ? affinity / Math.sqrt(features.length) : 0
  const quality = (candidate.score ?? 0) / 10
  const recency = profile.preferredYear && candidate.year
    ? Math.max(0, 1 - Math.abs(candidate.year - profile.preferredYear) / 20)
    : 0.25
  const score = normalizedAffinity * 0.72 + quality * 0.18 + recency * 0.1
  const reasons = includeReasons ? reasonsFor(candidate, features, profile) : undefined

  return {
    ...candidate,
    recommendation_score: Number(score.toFixed(4)),
    ...(reasons ? { reasons } : {}),
  }
}

function diversify(items: RecommendationPreview[], limit: number) {
  const selected: RecommendationPreview[] = []
  for (const item of items) {
    const itemGenres = new Set(item.genres ?? [])
    const overlaps = selected.filter((selectedItem) =>
      (selectedItem.genres ?? []).some((genre) => itemGenres.has(genre)),
    ).length
    if (overlaps < 4 || selected.length < Math.ceil(limit / 3)) selected.push(item)
    if (selected.length >= limit) break
  }
  return selected
}

function featuresOf(anime: AnimeMetadata | AnimePreview) {
  return [
    ...(anime.genres ?? []).map((genre) => `genre:${normalize(genre)}`),
    ...('tags' in anime ? anime.tags.map((tag) => `tag:${normalize(tag)}`) : []),
    anime.content_type ? `type:${anime.content_type}` : null,
    anime.year ? `era:${Math.floor(anime.year / 5) * 5}` : null,
  ].filter((value): value is string => !!value)
}

function reasonsFor(
  candidate: AnimePreview,
  features: string[],
  profile: ReturnType<typeof buildProfile>,
) {
  const matches = features
    .filter((feature) => (profile.weights.get(feature) ?? 0) > 0)
    .slice(0, 3)
    .map((feature) => feature.replace(/^(genre|tag|type|era):/, '').replace(/-/g, ' '))
  if (candidate.score && candidate.score >= 8) matches.push('strong community score')
  return matches.slice(0, 4)
}

function normalize(value: string) {
  return value.trim().toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '')
}

function mean(values: number[]) {
  return values.length ? values.reduce((total, value) => total + value, 0) / values.length : null
}

function clamp(value: number, min: number, max: number) {
  return Math.min(Math.max(value, min), max)
}

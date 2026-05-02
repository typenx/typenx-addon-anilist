# Typenx AniList Addon

Official Typenx metadata addon backed by the AniList GraphQL API.

This service provides catalog, search, anime metadata, and personalized recommendations. It does not return stream URLs or host media.

## Recommendations

`POST /recommendations` accepts liked and disliked anime IDs and returns ranked candidates:

```json
{
  "liked": [{ "anime_id": "1", "score": 10 }],
  "disliked": [{ "anime_id": "20", "score": 3 }],
  "limit": 24,
  "include_reasons": true
}
```

The current recommender is a lightweight hybrid model built for production data collection:

- Learns a taste profile from genres, tags, media type, era, positive feedback, and negative feedback.
- Ranks candidates with taste affinity, community quality, and freshness signals.
- Applies diversity pressure so recommendations do not become a wall of near-duplicates.
- Returns explanation snippets when `include_reasons` is true.

Next ML milestones: persist feedback events, train an implicit-feedback matrix-factorization candidate generator, add contextual bandit exploration, and A/B test ranking objectives against retention and completion-rate metrics.

## Local Development

```bash
npm install
npm run dev
```

The default port is `8788`.

## Typenx Core

Add the service URL to `TYPENX_BUILTIN_ADDONS`:

```env
TYPENX_BUILTIN_ADDONS=http://127.0.0.1:8787,http://127.0.0.1:8788
```

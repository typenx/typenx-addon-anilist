# Typenx AniList Addon

The official Typenx metadata addon for AniList.

This service speaks the Typenx addon protocol on top of AniList's GraphQL API. It returns catalogs, search results, full anime metadata, and personalized recommendations — without hosting media or returning stream URLs. Plug it into a [Typenx Core](https://github.com/typenx/typenx-core) instance and AniList becomes one more interchangeable source in the addon mesh.

## What it provides

- `GET /manifest` — declares `catalog`, `search`, `anime_meta`, and `recommendations` resources.
- `POST /catalog` — popular and seasonal feeds backed by AniList's MediaSort lists.
- `POST /search` — full-text search against the AniList index.
- `GET /anime/:id` — full metadata: synopsis, genres, tags, studios, staff, episode lists, external links.
- `POST /recommendations` — ranked candidates from a like/dislike taste profile.

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

The current recommender is a deliberately small hybrid model designed to collect clean feedback before heavier collaborative filtering goes in:

- Builds a positive and negative taste profile over genres, tags, media type, era, and feedback strength.
- Ranks candidates with taste affinity, community quality, and freshness signals.
- Applies diversity pressure so the response isn't a wall of near-duplicates.
- Returns explanation snippets when `include_reasons` is true, so a frontend can show *why* an item is on the list.

Roadmap from here: persist feedback events, train an implicit-feedback matrix-factorization candidate generator, layer contextual bandits for exploration, and A/B test ranking objectives against retention and completion rate.

## Local development

```bash
npm install
npm run dev
```

Default port: `8788`.

## Wiring it into Typenx Core

Add the service URL to `TYPENX_BUILTIN_ADDONS` so it's always on:

```env
TYPENX_BUILTIN_ADDONS=http://127.0.0.1:8787,http://127.0.0.1:8788
```

Or to `TYPENX_DEFAULT_ADDONS` if you want users to be able to disable it.

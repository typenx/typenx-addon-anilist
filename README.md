# Typenx AniList Addon

Official Typenx metadata addon backed by the AniList GraphQL API.

This service provides catalog, search, and anime metadata only. It does not return stream URLs or host media.

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

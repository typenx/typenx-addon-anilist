import { createTypenxAddon, serveTypenxAddon } from '@typenx/addon-ts-sdk'
import { AniListCatalog } from './anilist.js'

const anilist = new AniListCatalog()

const addon = createTypenxAddon({
  manifest: {
    id: 'typenx-addon-anilist',
    name: 'AniList',
    version: '0.1.0',
    description: 'Official Typenx metadata addon backed by AniList.',
    icon: 'https://anilist.co/img/icons/android-chrome-512x512.png',
    resources: ['catalog', 'search', 'anime_meta', 'recommendations'],
    catalogs: [
      {
        id: 'popular',
        name: 'Popular Anime',
        content_type: 'anime',
        filters: [],
      },
      {
        id: 'trending',
        name: 'Trending Anime',
        content_type: 'anime',
        filters: [],
      },
      {
        id: 'airing',
        name: 'Airing Anime',
        content_type: 'anime',
        filters: [],
      },
    ],
  },
  handlers: {
    catalog: (request) => anilist.catalog(request),
    search: (request) => anilist.search(request),
    anime: (id) => anilist.anime(id),
    recommendations: (request) => anilist.recommendations(request),
  },
})

serveTypenxAddon(addon, { port: Number(process.env.PORT ?? 8788) })

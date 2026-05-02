use anyhow::Context;
use std::env;
use typenx_addon_anilist::{api::AniListClient, server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "typenx_addon_anilist=info,tower_http=info".into()),
        )
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8788);

    let client = AniListClient::new();
    server::serve(client, port)
        .await
        .context("failed to serve Typenx AniList addon")
}

use fitgirl_ddl_lib::{errors::ExtractError, extract::{extract_ddl, DDL}, scrape::{scrape_game, GameInfo}};
use futures_util::StreamExt as _;
use itertools::Itertools as _;
use spdlog::{error, info, warn};

pub struct ExtractionInfo {
    pub saved_files: Vec<String>,
    pub missing_files: Vec<String>,
    pub scrape_errors: Vec<String>,
}

pub async fn export_ddl(game_urls: impl Iterator<Item = impl Into<String>>, workers: usize) -> Result<ExtractionInfo, ExtractError> {

    let scrape_results: Vec<_> = futures_util::stream::iter(game_urls.map(Into::into).collect::<Vec<String>>())
        .map(|game_url| {
            info!("processing {game_url}");
            async move {
                (game_url.clone(),
                scrape_game(&game_url)
                    .await
                    .inspect_err(|e| {
                        error!("failed to scrape {game_url}: {e}");
                    }))
            }
        })
        .buffer_unordered(workers)
        .collect()
        .await;

    let mut saved_files = Vec::new();
    let mut missing_files = Vec::new();
    let mut scrape_errors = Vec::new();
    for (game_url, result) in scrape_results {
        let GameInfo { path_part, fuckingfast_links } = match result {
            Ok(r) => r,
            Err(e) => {
                let error = format!("{game_url}: {e}");
                scrape_errors.push(error);
                continue
            },
        };

        let output_file = format!("{path_part}.txt");

        info!("start extracting for {path_part}");

        let ddls: Vec<_> = futures_util::stream::iter(fuckingfast_links.clone())
            .map(|ff_url| {
                info!("processing {ff_url}");
                async move {
                    extract_ddl(&ff_url).await.inspect_err(|e| {
                        error!("failed to extract {ff_url}: {e}");
                    })
                }
            })
            .buffer_unordered(workers)
            .collect()
            .await;

        let mut results = Vec::with_capacity(ddls.len());
        for result in ddls {
            match result {
                Err(ExtractError::RateLimited) => {
                    error!("early-exiting due to rate-limited error!");
                    return Err(ExtractError::RateLimited);
                }
                Err(ExtractError::FileNotFound(filename)) => {
                    warn!("missing file: {filename}");
                    missing_files.push(filename);
                }
                Ok(result) => results.push(result),
                _ => continue,
            }
        }

        #[rustfmt::skip] 
        let output_string: String = results.iter()
            .sorted_by(|&a, &b|{
                a.filename.cmp(&b.filename)
            })
            .map(
                |DDL {
                    filename,
                    direct_link,
                }| {
                    format!(
"{direct_link}
    out={filename}
    continue=true
"
                    )
                },
            ).collect();

        let _ = compio::fs::write(&output_file, output_string.into_bytes()).await;
        saved_files.push(output_file);
    }

    Ok(
        ExtractionInfo { saved_files, missing_files,scrape_errors }
    )
}

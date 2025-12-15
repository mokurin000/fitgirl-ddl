use std::path::Path;

use ahash::AHashMap;
use fitgirl_ddl_lib::{
    errors::ExtractError,
    extract::{DDL, extract_ddl},
    scrape::{GameInfo, scrape_game},
};
use futures_util::StreamExt as _;
use itertools::Itertools as _;
use tracing::{error, info, warn};
use winio::prelude::{ComponentSender, Layoutable as _, Monitor, MonitorExt as _, Window};

use crate::ui::main_model::{MainMessage, MainModel};

#[allow(unused)]
pub struct ExtractionInfo {
    pub saved_files: Vec<String>,
    pub missing_files: Vec<String>,
    pub scrape_errors: Vec<String>,
}

pub async fn export_ddl(
    game_urls: impl Iterator<Item = impl Into<String>>,
    mut workers: usize,
    sender: &ComponentSender<MainModel>,
    selective: bool,
) -> Result<ExtractionInfo, ExtractError> {
    let scrape_results: Vec<_> =
        futures_util::stream::iter(game_urls.map(Into::into).collect::<Vec<String>>())
            .map(|game_url| {
                info!("processing {game_url}");
                async move {
                    (
                        game_url.clone(),
                        scrape_game(&game_url).await.inspect_err(|e| {
                            error!("failed to scrape {game_url}: {e}");
                        }),
                    )
                }
            })
            .buffer_unordered(workers)
            .collect()
            .await;

    let mut saved_files = Vec::new();
    let mut missing_files = Vec::new();
    let mut scrape_errors = Vec::new();

    let total = scrape_results
        .iter()
        .filter_map(|r| r.1.as_ref().ok())
        .map(
            |GameInfo {
                 fuckingfast_links, ..
             }| fuckingfast_links.len(),
        )
        .sum::<usize>();
    sender.post(MainMessage::SetMaxCap(total));

    for (game_url, result) in scrape_results {
        let GameInfo {
            path_part,
            fuckingfast_links,
        } = match result {
            Ok(r) => r,
            Err(e) => {
                let error = format!("{game_url}: {e}");
                scrape_errors.push(error);
                continue;
            }
        };

        let output_file = format!("{path_part}_full.txt");

        info!("start extracting for {path_part}");

        if fuckingfast_links.len() >= 200 {
            info!("limiting workers due to too lots of DDL");
            workers = 1;
        }

        let ddls: Vec<_> = futures_util::stream::iter(fuckingfast_links.clone())
            .map(|ff_url| {
                info!("processing {ff_url}");
                async move {
                    let result = extract_ddl(&ff_url).await.inspect_err(|e| {
                        error!("failed to extract {ff_url}: {e}");
                    });
                    sender.post(MainMessage::IncreaseCount);
                    result
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

        write_aria2_input(&results, &output_file).await;

        if selective {
            sender.post(MainMessage::CreateSelection(results, path_part));
        }

        saved_files.push(output_file);
    }

    Ok(ExtractionInfo {
        saved_files,
        missing_files,
        scrape_errors,
    })
}

#[rustfmt::skip]
pub async fn write_aria2_input(
    ddls: impl IntoIterator<Item = &DDL>,
    output_file: impl AsRef<Path>,
) {
    let output_string: String = ddls
        .into_iter()
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

    match compio::fs::write(&output_file, output_string.into_bytes()).await.0 {
        Ok(_) => {
            info!("saved: {:?}", output_file.as_ref());
        }
        Err(e) => {
            error!("failed to save {:?}: {e}", output_file.as_ref());
        }
    }
}

pub fn collect_groups(ddls: impl IntoIterator<Item = DDL>) -> AHashMap<String, Vec<DDL>> {
    let mut groups: AHashMap<String, Vec<DDL>> = AHashMap::new();

    for DDL {
        filename,
        direct_link,
    } in ddls
    {
        let group_name = filename
            .split_once(".part")
            .map(|(pre, _)| pre.to_string())
            .unwrap_or(filename.clone());
        groups.entry(group_name).or_default().push(DDL {
            filename,
            direct_link,
        });
    }

    groups
}

pub fn centralize_window(window: &mut Window) -> winio::Result<()> {
    // centralize
    let monitor = Monitor::all()?.first().unwrap().client_scaled();
    window.set_loc(monitor.origin + monitor.size / 2. - window.size()? / 2.)?;
    Ok(())
}

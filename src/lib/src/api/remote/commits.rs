use crate::api::remote::client;
use crate::constants::HISTORY_DIR;
use crate::error::OxenError;
use crate::model::commit::CommitWithSize;
use crate::model::{Commit, LocalRepository, RemoteRepository};
use crate::util;
use crate::util::hasher::hash_buffer;
use crate::{api, constants};
// use crate::util::ReadProgress;
use crate::view::{CommitParentsResponse, CommitResponse, IsValidStatusMessage, StatusMessage};

use std::path::Path;
use std::str;
use std::sync::Arc;
use std::time;

use async_compression::futures::bufread::GzipDecoder;
use async_tar::Archive;
use bytesize::ByteSize;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures_util::TryStreamExt;
use indicatif::ProgressBar;

pub struct ChunkParams {
    pub chunk_num: usize,
    pub total_chunks: usize,
    pub total_size: usize,
}

pub async fn get_by_id(
    repository: &RemoteRepository,
    commit_id: &str,
) -> Result<Option<Commit>, OxenError> {
    let uri = format!("/commits/{commit_id}");
    let url = api::endpoint::url_from_repo(repository, &uri)?;
    log::debug!("remote::commits::get_by_id {}", url);

    let client = client::new_for_url(&url)?;
    if let Ok(res) = client.get(&url).send().await {
        if res.status() == 404 {
            return Ok(None);
        }

        let body = client::parse_json_body(&url, res).await?;
        log::debug!("api::remote::commits::get_by_id Got response {}", body);
        let response: Result<CommitResponse, serde_json::Error> = serde_json::from_str(&body);
        match response {
            Ok(j_res) => Ok(Some(j_res.commit)),
            Err(err) => Err(OxenError::basic_str(format!(
                "get_commit_by_id() Could not deserialize response [{err}]\n{body}"
            ))),
        }
    } else {
        Err(OxenError::basic_str("get_commit_by_id() Request failed"))
    }
}

pub async fn commit_is_synced(
    remote_repo: &RemoteRepository,
    commit_id: &str,
) -> Result<Option<IsValidStatusMessage>, OxenError> {
    let uri = format!("/commits/{commit_id}/is_synced");
    let url = api::endpoint::url_from_repo(remote_repo, &uri)?;
    log::debug!("commit_is_synced checking URL: {}", url);

    let client = client::new_for_url(&url)?;
    if let Ok(res) = client.get(&url).send().await {
        log::debug!("commit_is_synced Got response [{}]", res.status());
        if res.status() == 404 {
            return Ok(None);
        }

        let body = client::parse_json_body(&url, res).await?;
        log::debug!("commit_is_synced got response body: {}", body);
        let response: Result<IsValidStatusMessage, serde_json::Error> = serde_json::from_str(&body);
        match response {
            Ok(j_res) => Ok(Some(j_res)),
            Err(err) => {
                log::debug!("Error getting remote commit {}", err);
                Err(OxenError::basic_str(
                    "commit_is_synced() unable to parse body",
                ))
            }
        }
    } else {
        Err(OxenError::basic_str("commit_is_synced() Request failed"))
    }
}

pub async fn download_commit_db_by_id(
    local_repo: &LocalRepository,
    remote_repo: &RemoteRepository,
    commit_id: &str,
) -> Result<(), OxenError> {
    let uri = format!("/commits/{commit_id}/commit_db");
    let url = api::endpoint::url_from_repo(remote_repo, &uri)?;

    let client = client::new_for_url(&url)?;
    if let Ok(res) = client.get(url).send().await {
        // Unpack tarball to our hidden dir
        let hidden_dir = util::fs::oxen_hidden_dir(&local_repo.path);

        let reader = res
            .bytes_stream()
            .map_err(|e| futures::io::Error::new(futures::io::ErrorKind::Other, e))
            .into_async_read();
        let decoder = GzipDecoder::new(futures::io::BufReader::new(reader));
        let archive = Archive::new(decoder);
        archive.unpack(hidden_dir).await?;

        Ok(())
    } else {
        Err(OxenError::basic_str(
            "download_commit_db_by_id() Request failed",
        ))
    }
}

pub async fn get_remote_parent(
    remote_repo: &RemoteRepository,
    commit_id: &str,
) -> Result<Vec<Commit>, OxenError> {
    let uri = format!("/commits/{commit_id}/parents");
    let url = api::endpoint::url_from_repo(remote_repo, &uri)?;

    let client = client::new_for_url(&url)?;
    if let Ok(res) = client.get(&url).send().await {
        let body = client::parse_json_body(&url, res).await?;
        let response: Result<CommitParentsResponse, serde_json::Error> =
            serde_json::from_str(&body);
        match response {
            Ok(j_res) => Ok(j_res.parents),
            Err(err) => Err(OxenError::basic_str(format!(
                "get_remote_parent() Could not deserialize response [{err}]\n{body}"
            ))),
        }
    } else {
        Err(OxenError::basic_str("get_remote_parent() Request failed"))
    }
}

pub async fn post_push_complete(
    remote_repo: &RemoteRepository,
    commit_id: &str,
) -> Result<(), OxenError> {
    let uri = format!("/commits/{commit_id}/complete");
    let url = api::endpoint::url_from_repo(remote_repo, &uri)?;
    log::debug!("post_push_complete: {}", url);

    let client = client::new_for_url(&url)?;
    if let Ok(res) = client.post(&url).send().await {
        let body = client::parse_json_body(&url, res).await?;
        let response: Result<StatusMessage, serde_json::Error> = serde_json::from_str(&body);
        match response {
            Ok(_) => Ok(()),
            Err(err) => Err(OxenError::basic_str(format!(
                "post_push_complete() Could not deserialize response [{err}]\n{body}"
            ))),
        }
    } else {
        Err(OxenError::basic_str("post_push_complete() Request failed"))
    }
}

pub async fn post_commit_to_server(
    local_repo: &LocalRepository,
    remote_repo: &RemoteRepository,
    commit: &Commit,
    unsynced_entries_size: u64,
) -> Result<(), OxenError> {
    // Compute the size of the commit
    let commit_history_dir = util::fs::oxen_hidden_dir(&local_repo.path)
        .join(HISTORY_DIR)
        .join(&commit.id);
    let size = fs_extra::dir::get_size(commit_history_dir).unwrap() + unsynced_entries_size;

    // First create commit on server with size
    let commit_w_size = CommitWithSize::from_commit(commit, size);
    create_commit_obj_on_server(remote_repo, &commit_w_size).await?;

    // Then zip up and send the history db
    println!("Compressing commit {}", commit.id);

    // zip up the rocksdb in history dir, and post to server
    let commit_dir = util::fs::oxen_hidden_dir(&local_repo.path)
        .join(HISTORY_DIR)
        .join(commit.id.clone());
    // This will be the subdir within the tarball
    let tar_subdir = Path::new("history").join(commit.id.clone());

    let enc = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar = tar::Builder::new(enc);

    tar.append_dir_all(&tar_subdir, commit_dir)?;
    tar.finish()?;

    let buffer: Vec<u8> = tar.into_inner()?.finish()?;
    println!(
        "Syncing commit {} with size {}",
        commit.id,
        ByteSize::b(buffer.len() as u64)
    );

    let bar = Arc::new(ProgressBar::new(buffer.len() as u64));

    let is_compressed = true;
    let filename = None;
    post_data_to_server(remote_repo, commit, buffer, is_compressed, &filename, bar).await
}

async fn create_commit_obj_on_server(
    remote_repo: &RemoteRepository,
    commit: &CommitWithSize,
) -> Result<CommitResponse, OxenError> {
    let url = api::endpoint::url_from_repo(remote_repo, "/commits")?;
    let body = serde_json::to_string(&commit).unwrap();
    log::debug!("create_commit_obj_on_server {}\n{}", url, body);

    let client = client::new_for_url(&url)?;
    if let Ok(res) = client
        .post(&url)
        .body(reqwest::Body::from(body))
        .send()
        .await
    {
        let body = client::parse_json_body(&url, res).await?;
        log::debug!("create_commit_obj_on_server got response {}", body);
        let response: Result<CommitResponse, serde_json::Error> = serde_json::from_str(&body);
        match response {
            Ok(response) => Ok(response),
            Err(_) => Err(OxenError::basic_str(format!(
                "create_commit_obj_on_server Err deserializing \n\n{body}"
            ))),
        }
    } else {
        Err(OxenError::basic_str(
            "create_commit_obj_on_server error sending data from file",
        ))
    }
}

pub async fn post_data_to_server(
    remote_repo: &RemoteRepository,
    commit: &Commit,
    buffer: Vec<u8>,
    is_compressed: bool,
    filename: &Option<String>,
    bar: Arc<ProgressBar>,
) -> Result<(), OxenError> {
    let chunk_size: usize = constants::AVG_CHUNK_SIZE as usize;
    if buffer.len() > chunk_size {
        upload_data_to_server_in_chunks(
            remote_repo,
            commit,
            &buffer,
            chunk_size,
            is_compressed,
            filename,
            bar,
        )
        .await?;
    } else {
        upload_single_tarball_to_server_with_retry(remote_repo, commit, &buffer, bar).await?;
    }
    Ok(())
}

pub async fn upload_single_tarball_to_server_with_retry(
    remote_repo: &RemoteRepository,
    commit: &Commit,
    buffer: &[u8],
    bar: Arc<ProgressBar>,
) -> Result<(), OxenError> {
    let mut total_tries = 0;
    while total_tries < constants::NUM_HTTP_RETRIES {
        match upload_single_tarball_to_server(remote_repo, commit, buffer, bar.to_owned()).await {
            Ok(_) => {
                return Ok(());
            }
            Err(err) => {
                total_tries += 1;
                // Exponentially back off
                let sleep_time = total_tries * total_tries;
                log::debug!(
                    "upload_single_tarball_to_server_with_retry upload failed sleeping {}: {:?}",
                    sleep_time,
                    err
                );
                std::thread::sleep(std::time::Duration::from_secs(sleep_time));
            }
        }
    }

    Err(OxenError::basic_str("Upload retry failed."))
}

async fn upload_single_tarball_to_server(
    remote_repo: &RemoteRepository,
    commit: &Commit,
    buffer: &[u8],
    bar: Arc<ProgressBar>,
) -> Result<CommitResponse, OxenError> {
    let uri = format!("/commits/{}/data", commit.id);
    let url = api::endpoint::url_from_repo(remote_repo, &uri)?;

    let client = client::builder_for_url(&url)?
        .timeout(time::Duration::from_secs(120))
        .build()?;

    let size = buffer.len() as u64;
    match client.post(&url).body(buffer.to_owned()).send().await {
        Ok(res) => {
            let body = client::parse_json_body(&url, res).await?;

            log::debug!("upload_single_tarball_to_server got response {}", body);
            let response: Result<CommitResponse, serde_json::Error> = serde_json::from_str(&body);
            match response {
                Ok(response) => {
                    bar.inc(size);
                    Ok(response)
                }
                Err(_) => Err(OxenError::basic_str(format!(
                    "upload_single_tarball_to_server Err deserializing \n\n{body}"
                ))),
            }
        }
        Err(e) => {
            let err_str = format!("Err upload_single_tarball_to_server: {e:?}");
            Err(OxenError::basic_str(err_str))
        }
    }
}

async fn upload_data_to_server_in_chunks(
    remote_repo: &RemoteRepository,
    commit: &Commit,
    buffer: &[u8],
    chunk_size: usize,
    is_compressed: bool,
    filename: &Option<String>,
    bar: Arc<ProgressBar>,
) -> Result<(), OxenError> {
    let total_size = buffer.len();
    log::debug!(
        "upload_data_to_server_in_chunks chunking data {} ...",
        total_size
    );
    let chunks: Vec<&[u8]> = buffer.chunks(chunk_size).collect();
    let hash = hash_buffer(buffer);
    log::debug!(
        "upload_data_to_server_in_chunks got {} chunks from {}",
        chunks.len(),
        ByteSize::b(total_size as u64)
    );

    for (i, chunk) in chunks.iter().enumerate() {
        log::debug!(
            "upload_data_to_server_in_chunks uploading chunk {} of size {}",
            i,
            ByteSize::b(chunks.len() as u64)
        );

        let params = ChunkParams {
            chunk_num: i,
            total_chunks: chunks.len(),
            total_size,
        };
        match upload_data_chunk_to_server_with_retry(
            remote_repo,
            commit,
            chunk,
            &hash,
            &params,
            is_compressed,
            filename,
        )
        .await
        {
            Ok(_) => {
                log::debug!("Success uploading chunk!")
            }
            Err(err) => {
                log::error!("Err uploading chunk: {}", err)
            }
        }
        bar.inc(chunk.len() as u64)
    }
    Ok(())
}

pub async fn upload_data_chunk_to_server_with_retry(
    remote_repo: &RemoteRepository,
    commit: &Commit,
    chunk: &[u8],
    hash: &str,
    params: &ChunkParams,
    is_compressed: bool,
    filename: &Option<String>,
) -> Result<(), OxenError> {
    let mut total_tries = 0;
    while total_tries < constants::NUM_HTTP_RETRIES {
        match upload_data_chunk_to_server(
            remote_repo,
            commit,
            chunk,
            hash,
            params,
            is_compressed,
            filename,
        )
        .await
        {
            Ok(_) => {
                return Ok(());
            }
            Err(err) => {
                total_tries += 1;
                // Exponentially back off
                let sleep_time = total_tries * total_tries;
                log::debug!(
                    "upload_data_chunk_to_server_with_retry upload failed sleeping {}: {:?}",
                    sleep_time,
                    err
                );
                std::thread::sleep(std::time::Duration::from_secs(sleep_time));
            }
        }
    }

    Err(OxenError::basic_str("Upload chunk retry failed."))
}

async fn upload_data_chunk_to_server(
    remote_repo: &RemoteRepository,
    commit: &Commit,
    chunk: &[u8],
    hash: &str,
    params: &ChunkParams,
    is_compressed: bool,
    filename: &Option<String>,
) -> Result<CommitResponse, OxenError> {
    let maybe_filename = if !is_compressed {
        format!(
            "&filename={}",
            urlencoding::encode(
                filename
                    .as_ref()
                    .expect("Must provide filename if !compressed")
            )
        )
    } else {
        String::from("")
    };

    let uri = format!(
        "/commits/{}/upload_chunk?chunk_num={}&total_size={}&hash={}&total_chunks={}&is_compressed={}{}",
        commit.id, params.chunk_num, params.total_size, hash, params.total_chunks, is_compressed, maybe_filename
    );
    let url = api::endpoint::url_from_repo(remote_repo, &uri)?;
    let total_size = chunk.len() as u64;
    log::debug!(
        "upload_data_chunk_to_server posting {} to url {}",
        ByteSize::b(total_size),
        url
    );

    let client = client::builder_for_url(&url)?
        .timeout(time::Duration::from_secs(120))
        .build()?;

    match client.post(&url).body(chunk.to_owned()).send().await {
        Ok(res) => {
            let body = client::parse_json_body(&url, res).await?;

            log::debug!("upload_data_chunk_to_server got response {}", body);
            let response: Result<CommitResponse, serde_json::Error> = serde_json::from_str(&body);
            match response {
                Ok(response) => Ok(response),
                Err(_) => Err(OxenError::basic_str(format!(
                    "upload_data_chunk_to_server Err deserializing\n\n{body}"
                ))),
            }
        }
        Err(e) => {
            let err_str = format!("Err upload_data_chunk_to_server: {e:?}");
            Err(OxenError::basic_str(err_str))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api;
    use crate::command;
    use crate::constants;
    use crate::error::OxenError;
    use crate::test;

    use std::thread;

    #[tokio::test]
    async fn test_remote_commits_post_commit_to_server() -> Result<(), OxenError> {
        test::run_training_data_sync_test_no_commits(|local_repo, remote_repo| async move {
            // Track the annotations dir
            // has format
            //   annotations/
            //     train/
            //       one_shot.csv
            //       annotations.txt
            //     test/
            //       annotations.txt
            let annotations_dir = local_repo.path.join("annotations");
            command::add(&local_repo, &annotations_dir)?;
            // Commit the directory
            let commit = command::commit(
                &local_repo,
                "Adding annotations data dir, which has two levels",
            )?
            .unwrap();

            // Post commit
            let entries_size = 1000; // doesn't matter, since we aren't verifying size in tests
            api::remote::commits::post_commit_to_server(
                &local_repo,
                &remote_repo,
                &commit,
                entries_size,
            )
            .await?;

            Ok(remote_repo)
        })
        .await
    }

    #[tokio::test]
    async fn test_remote_commits_commit_is_valid() -> Result<(), OxenError> {
        test::run_training_data_repo_test_fully_committed_async(|local_repo| async move {
            let mut local_repo = local_repo;
            let commit_history = command::log(&local_repo)?;
            let commit = commit_history.first().unwrap();

            // Set the proper remote
            let name = local_repo.dirname();
            let remote = test::repo_remote_url_from(&name);
            command::add_remote(&mut local_repo, constants::DEFAULT_REMOTE_NAME, &remote)?;

            // Create Remote
            let remote_repo = command::create_remote(
                &local_repo,
                constants::DEFAULT_NAMESPACE,
                &local_repo.dirname(),
                test::test_host(),
            )
            .await?;

            // Push it
            command::push(&local_repo).await?;

            // We unzip in a background thread, so give it a second
            thread::sleep(std::time::Duration::from_secs(1));

            let is_synced = api::remote::commits::commit_is_synced(&remote_repo, &commit.id)
                .await?
                .unwrap();
            assert!(is_synced.is_valid);

            api::remote::repositories::delete(&remote_repo).await?;

            Ok(())
        })
        .await
    }

    #[tokio::test]
    async fn test_remote_commits_is_not_valid() -> Result<(), OxenError> {
        test::run_training_data_sync_test_no_commits(|local_repo, remote_repo| async move {
            // Track the annotations dir
            // has format
            //   annotations/
            //     train/
            //       one_shot.csv
            //       annotations.txt
            //     test/
            //       annotations.txt
            let annotations_dir = local_repo.path.join("annotations");
            command::add(&local_repo, &annotations_dir)?;
            // Commit the directory
            let commit = command::commit(
                &local_repo,
                "Adding annotations data dir, which has two levels",
            )?
            .unwrap();

            // Post commit but not the actual files
            let entries_size = 1000; // doesn't matter, since we aren't verifying size in tests
            api::remote::commits::post_commit_to_server(
                &local_repo,
                &remote_repo,
                &commit,
                entries_size,
            )
            .await?;

            // Should not be synced because we didn't actually post the files
            let is_synced =
                api::remote::commits::commit_is_synced(&remote_repo, &commit.id).await?;
            // We never kicked off the background processes
            assert!(is_synced.is_none());

            Ok(remote_repo)
        })
        .await
    }
}

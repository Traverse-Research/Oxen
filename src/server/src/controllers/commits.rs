use liboxen::api;
use liboxen::error::OxenError;
use liboxen::index::Committer;
use liboxen::model::{Commit, LocalRepository};
use liboxen::util;
use liboxen::view::http::{MSG_RESOURCE_CREATED, STATUS_SUCCESS};
use liboxen::view::{CommitResponse, ListCommitResponse, StatusMessage};

use crate::app_data::SyncDir;

use actix_web::{web, Error, HttpRequest, HttpResponse};
use flate2::read::GzDecoder;
use futures_util::stream::StreamExt as _;
use serde::Deserialize;
use std::path::Path;
use tar::Archive;

#[derive(Deserialize, Debug)]
pub struct CommitQuery {
    commit_id: String,
    parent_id: Option<String>,
    message: String,
    author: String,
    date: String,
}

// List commits for a repository
pub async fn index(req: HttpRequest) -> HttpResponse {
    let sync_dir = req.app_data::<SyncDir>().unwrap();
    let path: Option<&str> = req.match_info().get("name");

    if let Some(path) = path {
        let repo_dir = sync_dir.path.join(path);
        // TODO do less matching and take care of flow in subroutine and propigate up error
        match p_index(&repo_dir) {
            Ok(response) => HttpResponse::Ok().json(response),
            Err(err) => {
                let msg = format!("api err: {}", err);
                HttpResponse::NotFound().json(StatusMessage::error(&msg))
            }
        }
    } else {
        let msg = "Could not find `name` param...";
        HttpResponse::NotFound().json(StatusMessage::error(msg))
    }
}

fn p_index(repo_dir: &Path) -> Result<ListCommitResponse, OxenError> {
    let repo = LocalRepository::from_dir(repo_dir)?;
    let committer = Committer::new(&repo)?;
    let commits = committer.list_commits()?;
    Ok(ListCommitResponse::success(commits))
}

pub async fn upload(
    req: HttpRequest,
    mut body: web::Payload,        // the actual file body
    data: web::Query<CommitQuery>, // these are the query params -> struct
) -> Result<HttpResponse, Error> {
    let sync_dir = req.app_data::<SyncDir>().unwrap();
    // name to the repo, should be in url path so okay to unwap
    let name: &str = req.match_info().get("name").unwrap();
    match api::local::repositories::get_by_name(&sync_dir.path, name) {
        Ok(repo) => {
            let hidden_dir = util::fs::oxen_hidden_dir(&repo.path);

            // Create Commit from uri params
            let commit = Commit {
                id: data.commit_id.clone(),
                parent_id: data.parent_id.clone(),
                message: data.message.clone(),
                author: data.author.clone(),
                date: Commit::date_from_str(&data.date),
            };

            match create_commit(&repo.path, &commit) {
                Ok(_) => {
                    // Get tar.gz bytes for history/COMMIT_ID data
                    let mut bytes = web::BytesMut::new();
                    while let Some(item) = body.next().await {
                        bytes.extend_from_slice(&item?);
                    }

                    // Unpack tarball to our hidden dir
                    let mut archive = Archive::new(GzDecoder::new(&bytes[..]));
                    archive.unpack(hidden_dir)?;

                    Ok(HttpResponse::Ok().json(CommitResponse {
                        status: String::from(STATUS_SUCCESS),
                        status_message: String::from(MSG_RESOURCE_CREATED),
                        commit,
                    }))
                }
                Err(err) => {
                    let msg = format!("Err: {}", err);
                    Ok(HttpResponse::Ok().json(StatusMessage::error(&msg)))
                }
            }
        }
        Err(err) => {
            let msg = format!("Err: {}", err);
            Ok(HttpResponse::Ok().json(StatusMessage::error(&msg)))
        }
    }
}

fn create_commit(repo_dir: &Path, commit: &Commit) -> Result<(), OxenError> {
    let repo = LocalRepository::from_dir(repo_dir)?;
    let result = Committer::new(&repo);
    match result {
        Ok(mut committer) => match committer.add_commit(commit) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error adding commit to db: {:?}", err);
            }
        },
        Err(err) => {
            eprintln!("Error creating committer: {:?}", err);
        }
    };
    Ok(())
}

#[cfg(test)]
mod tests {

    use actix_web::body::to_bytes;
    use actix_web::{web, App};
    use chrono::Utc;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::path::Path;

    use liboxen::command;
    use liboxen::constants::OXEN_HIDDEN_DIR;
    use liboxen::error::OxenError;
    use liboxen::model::Commit;
    use liboxen::util;
    use liboxen::view::{CommitResponse, ListCommitResponse};

    use crate::app_data::SyncDir;
    use crate::controllers;
    use crate::test;

    #[actix_web::test]
    async fn test_controllers_respository_commits_index_empty() -> Result<(), OxenError> {
        let sync_dir = test::get_sync_dir()?;

        let name = "Testing-Name";
        test::create_local_repo(&sync_dir, name)?;

        let uri = format!("/repositories/{}/commits", name);
        let req = test::request_with_param(&sync_dir, &uri, "name", name);

        let resp = controllers::commits::index(req).await;

        let body = to_bytes(resp.into_body()).await.unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        let list: ListCommitResponse = serde_json::from_str(text)?;
        assert_eq!(list.commits.len(), 0);

        // cleanup
        std::fs::remove_dir_all(sync_dir)?;

        Ok(())
    }

    #[actix_web::test]
    async fn test_controllers_respository_list_two_commits() -> Result<(), OxenError> {
        let sync_dir = test::get_sync_dir()?;

        let name = "Testing-Name";
        let repo = test::create_local_repo(&sync_dir, name)?;

        liboxen::test::add_txt_file_to_dir(&repo.path, "hello")?;
        command::commit(&repo, "first commit")?;
        liboxen::test::add_txt_file_to_dir(&repo.path, "world")?;
        command::commit(&repo, "second commit")?;

        let uri = format!("/repositories/{}/commits", name);
        let req = test::request_with_param(&sync_dir, &uri, "name", name);

        let resp = controllers::commits::index(req).await;
        let body = to_bytes(resp.into_body()).await.unwrap();
        let text = std::str::from_utf8(&body).unwrap();
        let list: ListCommitResponse = serde_json::from_str(text)?;
        assert_eq!(list.commits.len(), 2);

        // cleanup
        std::fs::remove_dir_all(sync_dir)?;

        Ok(())
    }

    #[actix_web::test]
    async fn test_controllers_commits_upload() -> Result<(), OxenError> {
        let sync_dir = test::get_sync_dir()?;

        let name = "Testing-Name";
        let repo = test::create_local_repo(&sync_dir, name)?;
        let commit = Commit {
            id: format!("{}", uuid::Uuid::new_v4()),
            parent_id: None,
            message: String::from("Hello"),
            author: String::from("Greg"),
            date: Utc::now(),
        };

        // create random tarball to post.. currently no validation that it is a valid commit dir
        let path_to_compress = format!("history/{}", commit.id);
        let commit_dir_name = format!("/tmp/oxen/commit/{}", commit.id);
        let commit_dir = Path::new(&commit_dir_name);
        std::fs::create_dir_all(commit_dir)?;
        // Write a random file to it
        let random_file = commit_dir.join("blah.txt");
        util::fs::write_to_path(&random_file, "sup");

        println!("Compressing commit {}...", commit.id);
        let enc = GzEncoder::new(Vec::new(), Compression::default());
        let mut tar = tar::Builder::new(enc);

        tar.append_dir_all(&path_to_compress, &commit_dir)?;
        tar.finish()?;
        let payload: Vec<u8> = tar.into_inner()?.finish()?;

        let commit_query = Commit::to_uri_encoded(&commit);
        let uri = format!("/repositories/{}/commits?{}", name, commit_query);
        let app = actix_web::test::init_service(
            App::new()
                .app_data(SyncDir {
                    path: sync_dir.clone(),
                })
                .route(
                    "/repositories/{name}/commits",
                    web::post().to(controllers::commits::upload),
                ),
        )
        .await;

        let req = actix_web::test::TestRequest::post()
            .uri(&uri)
            .set_payload(payload)
            .to_request();

        let resp = actix_web::test::call_service(&app, req).await;
        let bytes = actix_http::body::to_bytes(resp.into_body()).await.unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        println!("GOT BODY: {}", body);
        let resp: CommitResponse = serde_json::from_str(body)?;

        // Make sure commit gets populated
        assert_eq!(resp.commit.id, commit.id);
        assert_eq!(resp.commit.message, commit.message);
        assert_eq!(resp.commit.author, commit.author);
        assert_eq!(resp.commit.parent_id, commit.parent_id);

        // Make sure we unzipped the tar ball
        let uploaded_file = sync_dir
            .join(repo.name)
            .join(OXEN_HIDDEN_DIR)
            .join(path_to_compress)
            .join("blah.txt");
        assert!(uploaded_file.exists());
        assert_eq!(util::fs::read_from_path(&uploaded_file)?, "sup");

        // cleanup
        std::fs::remove_dir_all(sync_dir)?;
        std::fs::remove_dir_all(commit_dir)?;

        Ok(())
    }
}

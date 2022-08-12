use actix_web::web;

use super::controllers;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.route(
        "/{namespace}",
        web::get().to(controllers::repositories::index),
    )
    .route(
        "/{namespace}/{repo_name}",
        web::get().to(controllers::repositories::show),
    )
    .route(
        "/{namespace}/{repo_name}",
        web::delete().to(controllers::repositories::delete),
    )
    .route(
        "/{namespace}/{repo_name}/commits",
        web::get().to(controllers::commits::index),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}",
        web::post().to(controllers::commits::upload),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}",
        web::get().to(controllers::commits::show),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}/is_synced",
        web::get().to(controllers::commits::is_synced),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}/commit_db",
        web::get().to(controllers::commits::download_commit_db),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}/parents",
        web::get().to(controllers::commits::parents),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}/entries",
        web::get().to(controllers::entries::list_entries),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}/files",
        web::get().to(controllers::entries::list_files_for_commit),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}/download_page",
        web::get().to(controllers::entries::download_page),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}/download_content_ids",
        web::post().to(controllers::entries::download_content_ids),
    )
    .route(
        "/{namespace}/{repo_name}/branches",
        web::get().to(controllers::branches::index),
    )
    .route(
        "/{namespace}/{repo_name}/branches",
        web::post().to(controllers::branches::create_or_get),
    )
    .route(
        "/{namespace}/{repo_name}/branches/{branch_name}",
        web::get().to(controllers::branches::show),
    )
    .route(
        "/{namespace}/{repo_name}/branches/{branch_name}",
        web::delete().to(controllers::branches::delete),
    )
    .route(
        "/{namespace}/{repo_name}/branches/{branch_name}/commits",
        web::get().to(controllers::commits::index_branch),
    )
    .route(
        "/{namespace}/{repo_name}/branches/{branch_name}/commits",
        web::post().to(controllers::commits::create),
    )
    .route(
        "/{namespace}/{repo_name}/entries",
        web::post().to(controllers::entries::create),
    )
    .route(
        "/{namespace}/{repo_name}/lines/{resource:.*}",
        web::get().to(controllers::entries::list_lines_in_file),
    )
    .route(
        "/{namespace}/{repo_name}/branches/{branch_name}/entries/{filename:.*}",
        web::get().to(controllers::repositories::get_file_for_branch),
    )
    .route(
        "/{namespace}/{repo_name}/commits/{commit_id}/entries/{filename:.*}",
        web::get().to(controllers::repositories::get_file_for_commit_id),
    )
    .route(
        "/{namespace}/{repo_name}/files",
        web::get().to(controllers::entries::list_files_for_head),
    )
    .route(
        "/repositories",
        web::post().to(controllers::repositories::create),
    );
}

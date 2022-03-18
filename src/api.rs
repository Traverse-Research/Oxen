use crate::config::oxen_config::OxenConfig;
use crate::config::repo_config::RepoConfig;
use crate::error::OxenError;
use crate::model::dataset::*;
use crate::model::entry::*;
use crate::model::user::*;
use reqwest::blocking::Client;
use serde_json::json;

pub mod datasets;
pub mod repositories;

pub fn login(config: &OxenConfig, email: &str, password: &str) -> Result<User, OxenError> {
    let url = format!("{}/login", config.endpoint());
    let params = json!({
      "user": {
        "email": email,
        "password": password,
      }
    });

    if let Ok(res) = Client::new().post(&url).json(&params).send() {
        let status = res.status();
        if let Ok(user_res) = res.json::<UserResponse>() {
            Ok(user_res.user)
        } else {
            let err = format!(
                "login failed status_code[{}], check email and password",
                status
            );
            Err(OxenError::Basic(err))
        }
    } else {
        Err(OxenError::Basic(format!("login failed [{}]", &url)))
    }
}

pub fn get_user(config: &RepoConfig) -> Result<User, OxenError> {
    let url = format!("{}/login", config.endpoint());
    let params = json!({
      "user": {
        "email": "denied",
        "password": "nope",
      }
    });

    if let Ok(res) = Client::new().post(url).json(&params).send() {
        let status = res.status();
        if let Ok(user_res) = res.json::<UserResponse>() {
            Ok(user_res.user)
        } else {
            Err(OxenError::basic_str(&format!(
                "status_code[{}], check email and password",
                status
            )))
        }
    } else {
        Err(OxenError::basic_str("api::get_user() API failed"))
    }
}

pub fn entry_from_hash(config: &RepoConfig, hash: &str) -> Result<Entry, OxenError> {
    if let Some(user) = &config.user {
        let url = format!("{}/entries/search?hash={}", config.endpoint(), hash);
        let client = reqwest::blocking::Client::new();
        if let Ok(res) = client
            .get(url)
            .header(reqwest::header::AUTHORIZATION, &user.token)
            .send()
        {
            if let Ok(entry_res) = res.json::<EntryResponse>() {
                Ok(entry_res.entry)
            } else {
                Err(OxenError::basic_str("Could not serialize entry"))
            }
        } else {
            println!("hash_exists request failed..");
            Err(OxenError::basic_str("Request failed"))
        }
    } else {
        Err(OxenError::basic_str("User is not logged in."))
    }
}

pub fn create_dataset(config: &RepoConfig, name: &str) -> Result<Dataset, OxenError> {
    if let (Some(user), Some(repository_id)) = (&config.user, &config.repository_id) {
        let url = format!(
            "{}/repositories/{}/datasets",
            config.endpoint(),
            repository_id
        );
        let params = json!({
          "name": name,
        });

        if let Ok(res) = Client::new()
            .post(url)
            .header(reqwest::header::AUTHORIZATION, &user.token)
            .json(&params)
            .send()
        {
            let status = res.status();
            if let Ok(user_res) = res.json::<DatasetResponse>() {
                Ok(user_res.dataset)
            } else {
                Err(OxenError::basic_str(&format!(
                    "status_code[{}], could not create dataset",
                    status
                )))
            }
        } else {
            Err(OxenError::basic_str("api::create_dataset() API failed"))
        }
    } else {
        Err(OxenError::basic_str("User is not logged in."))
    }
}
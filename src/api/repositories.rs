use crate::config::{HTTPConfig};
use crate::error::OxenError;
use crate::model::repository::{Repository, RepositoryResponse};
use crate::model::status_message::StatusMessage;
use serde_json::json;
use urlencoding::encode;

pub fn create<'a>(config: &'a dyn HTTPConfig<'a>, name: &str) -> Result<Repository, OxenError> {
    let url = format!("http://{}/api/v1/repositories", config.host());
    let params = json!({
        "name": name,
        "is_public": true
    });

    let client = reqwest::blocking::Client::new();
    if let Ok(res) = client
        .post(url)
        .json(&params)
        .header(reqwest::header::AUTHORIZATION, config.auth_token())
        .send()
    {
        let status = res.status();
        let body = res.text()?;
        let response: Result<RepositoryResponse, serde_json::Error> = serde_json::from_str(&body);
        match response {
            Ok(val) => {
                Ok(val.repository)
            },
            Err(_) => {
                Err(OxenError::basic_str(&format!(
                    "status_code[{}], could not create repository \n\n{}",
                    status,
                    body
                )))
            }
        }
    } else {
        Err(OxenError::basic_str(
            "api::repositories::create() Request failed",
        ))
    }
}

pub fn get_by_url<'a>(config: &'a dyn HTTPConfig<'a>, url: &str) -> Result<Repository, OxenError> {
    let encoded_url = encode(url);
    let client = reqwest::blocking::Client::new();
    if let Ok(res) = client
        .get(format!(
            "http://{}/api/v1/repositories/get_by_url?url={}",
            config.host(),
            encoded_url
        ))
        .header(reqwest::header::AUTHORIZATION, config.auth_token())
        .send()
    {
        match res.json::<RepositoryResponse>() {
            Ok(j_res) => Ok(j_res.repository),
            Err(err) => Err(OxenError::basic_str(&format!(
                "api::repositories::get_by_url() Could not serialize repository [{}]",
                err
            ))),
        }
    } else {
        Err(OxenError::basic_str(
            "api::repositories::create() Request failed",
        ))
    }
}

pub fn delete<'a>(config: &'a dyn HTTPConfig<'a>, repository: &Repository) -> Result<StatusMessage, OxenError> {
    let url = format!("http://{}/api/v1/repositories/{}", config.host(), repository.id);

    let client = reqwest::blocking::Client::new();
    if let Ok(res) = client
        .delete(url)
        .header(reqwest::header::AUTHORIZATION, config.auth_token())
        .send()
    {
        let status = res.status();
        let body = res.text()?;
        let response: Result<StatusMessage, serde_json::Error> = serde_json::from_str(&body);
        match response {
            Ok(val) => {
                Ok(val)
            },
            Err(_) => {
                Err(OxenError::basic_str(&format!(
                    "status_code[{}], could not delete repository \n\n{}",
                    status,
                    body
                )))
            }
        }
    } else {
        Err(OxenError::basic_str(
            "api::repositories::delete() Request failed",
        ))
    }
}

#[cfg(test)]
mod tests {

    use crate::api;
    use crate::config::{AuthConfig};
    use crate::error::OxenError;
    use crate::test;

    #[test]
    fn test_create_repository() -> Result<(), OxenError> {
        let path = test::auth_cfg_file();
        let config = AuthConfig::from(path);
        let name: &str = "test_create_repository";

        let repository = api::repositories::create(&config, name)?;
        assert_eq!(repository.name, name);

        // cleanup
        api::repositories::delete(&config, &repository)?;
        Ok(())
    }

    #[test]
    fn test_get_by_url() -> Result<(), OxenError> {
        let path = test::auth_cfg_file();
        let config = AuthConfig::from(path);
        let name: &str = "test_get_by_url";

        let repository = api::repositories::create(&config, name)?;
        let url_repo = api::repositories::get_by_url(&config, &repository.url)?;

        assert_eq!(repository.id, url_repo.id);

        // cleanup
        api::repositories::delete(&config, &repository)?;
        Ok(())
    }
}

use crate::api;
use crate::constants::NO_REPO_MSG;
use crate::error::OxenError;
use crate::model::{Remote, RemoteRepository};
use crate::util;
use crate::view::RepositoryView;

use http::Uri;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug, Clone)]
pub struct RepositoryNew {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalRepository {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    remote_name: Option<String>, // this is the current remote name
    remotes: Vec<Remote>,
}

impl LocalRepository {
    // Create a brand new repository with new ID
    pub fn new(path: &Path) -> Result<LocalRepository, OxenError> {
        // we're assuming the path is valid...
        let name = path.file_name().unwrap().to_str().unwrap();
        Ok(LocalRepository {
            // generate new uuid locally
            id: format!("{}", uuid::Uuid::new_v4()),
            name: String::from(name),
            path: path.to_path_buf(),
            remotes: vec![],
            remote_name: None,
        })
    }

    pub fn from_view(view: RepositoryView) -> Result<LocalRepository, OxenError> {
        Ok(LocalRepository {
            // generate new uuid locally
            id: view.id.clone(),
            name: view.name.clone(),
            path: std::env::current_dir()?.join(view.name),
            remotes: vec![],
            remote_name: None,
        })
    }

    pub fn from_remote(view: RemoteRepository) -> Result<LocalRepository, OxenError> {
        Ok(LocalRepository {
            // generate new uuid locally
            id: view.id.clone(),
            name: view.name.clone(),
            path: std::env::current_dir()?.join(view.name),
            remotes: vec![],
            remote_name: None,
        })
    }

    pub fn from_cfg(path: &Path) -> Result<LocalRepository, OxenError> {
        let contents = util::fs::read_from_path(path)?;
        let repo: LocalRepository = toml::from_str(&contents)?;
        Ok(repo)
    }

    pub fn from_dir(dir: &Path) -> Result<LocalRepository, OxenError> {
        let config_path = util::fs::config_filepath(dir);
        if !config_path.exists() {
            return Err(OxenError::basic_str(NO_REPO_MSG));
        }
        let repo = LocalRepository::from_cfg(&config_path)?;
        Ok(repo)
    }

    pub fn save(&self, path: &Path) -> Result<(), OxenError> {
        let toml = toml::to_string(&self)?;
        util::fs::write_to_path(path, &toml);
        Ok(())
    }

    pub fn save_default(&self) -> Result<(), OxenError> {
        let filename = util::fs::config_filepath(&self.path);
        self.save(&filename)?;
        Ok(())
    }

    pub fn clone_remote(url: &str) -> Result<LocalRepository, OxenError> {
        let name = LocalRepository::dirname_from_url(url)?;
        match api::remote::repositories::get_by_name(&name) {
            Ok(remote_repo) => LocalRepository::clone_repo(remote_repo),
            Err(_) => {
                let err = format!("Could not clone remote {} not found", url);
                Err(OxenError::basic_str(&err))
            }
        }
    }

    pub fn set_remote(&mut self, name: &str, value: &str) {
        self.remote_name = Some(String::from(name));
        let remote = Remote {
            name: String::from(name),
            value: String::from(value),
        };
        if self.has_remote(name) {
            // find remote by name and set
            for i in 0..self.remotes.len() {
                if self.remotes[i].name == name {
                    self.remotes[i] = remote.clone()
                }
            }
        } else {
            // we don't have the key, just push
            self.remotes.push(remote);
        }
    }

    pub fn has_remote(&self, name: &str) -> bool {
        for remote in self.remotes.iter() {
            if remote.name == name {
                return true;
            }
        }
        false
    }

    pub fn remote(&self) -> Option<Remote> {
        if let Some(name) = &self.remote_name {
            for remote in self.remotes.iter() {
                if &remote.name == name {
                    return Some(remote.clone());
                }
            }
            None
        } else {
            None
        }
    }

    fn clone_repo(repo: RemoteRepository) -> Result<LocalRepository, OxenError> {
        // get last part of URL for directory name
        let url = &repo.url;
        let dir_name = LocalRepository::dirname_from_url(url)?;

        // if directory already exists -> return Err
        let repo_path = Path::new(&dir_name);
        if repo_path.exists() {
            let err = format!("Directory already exists: {}", dir_name);
            return Err(OxenError::basic_str(&err));
        }

        // if directory does not exist, create it
        std::fs::create_dir(&repo_path)?;

        // if create successful, create .oxen directory
        let oxen_hidden_path = util::fs::oxen_hidden_dir(repo_path);
        std::fs::create_dir(&oxen_hidden_path)?;

        // save Repository in .oxen directory
        let repo_config_file = oxen_hidden_path.join(Path::new("config.toml"));
        let toml = toml::to_string(&repo)?;
        util::fs::write_to_path(&repo_config_file, &toml);

        println!(
            "🐂 cloned {} to {}\n\ncd {}\noxen pull",
            url, dir_name, dir_name
        );

        let mut local_repo = LocalRepository {
            id: repo.id.clone(),
            name: repo.name.clone(),
            path: repo_path.to_path_buf(),
            remotes: vec![],
            remote_name: None,
        };
        local_repo.set_remote("origin", url);
        Ok(local_repo)
    }

    pub fn dirname_from_url(url: &str) -> Result<String, OxenError> {
        let uri = url.parse::<Uri>()?;
        if let Some(dirname) = uri.path().split('/').last() {
            Ok(String::from(dirname))
        } else {
            Err(OxenError::basic_str(""))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api;
    use crate::error::OxenError;
    use crate::model::LocalRepository;
    use crate::test;

    use std::path::Path;

    #[test]
    fn test_get_dirname_from_url() -> Result<(), OxenError> {
        let url = "http://0.0.0.0:3000/repositories/OxenData";
        let dirname = LocalRepository::dirname_from_url(url)?;
        assert_eq!(dirname, "OxenData");
        Ok(())
    }

    #[test]
    fn test_clone_remote() -> Result<(), OxenError> {
        let name = "OxenDataTest";
        let remote_repo = api::remote::repositories::create_or_get(name)?;
        let url = &remote_repo.url;

        let local_repo = LocalRepository::clone_remote(url)?;

        let cfg_path = format!("{}/.oxen/config.toml", name);
        let path = Path::new(&cfg_path);
        assert!(path.exists());
        assert_eq!(local_repo.name, local_repo.name);
        assert_eq!(local_repo.id, local_repo.id);

        // cleanup
        api::remote::repositories::delete(remote_repo)?;
        std::fs::remove_dir_all(name)?;

        Ok(())
    }

    #[test]
    fn test_read_cfg() -> Result<(), OxenError> {
        let path = test::repo_cfg_file();
        let repo = LocalRepository::from_cfg(path)?;
        assert_eq!(repo.id, "0af558cc-a57c-4197-a442-50eb889e9495");
        assert_eq!(repo.name, "Mini-Dogs-Vs-Cats");
        assert_eq!(repo.path, Path::new("/tmp/Mini-Dogs-Vs-Cats"));
        Ok(())
    }

    #[test]
    fn test_create_repo_cfg() -> Result<(), OxenError> {
        let name: &str = "Test Repo";
        let repository = test::create_remote_repo(name)?;
        assert_eq!(repository.name, name);
        // cleanup
        api::remote::repositories::delete(repository)?;
        Ok(())
    }

    #[test]
    fn test_local_repository_save() -> Result<(), OxenError> {
        let final_path = Path::new("/tmp/repo_config.toml");
        let orig_repo = LocalRepository::from_cfg(test::repo_cfg_file())?;

        orig_repo.save(final_path)?;

        let repo = LocalRepository::from_cfg(final_path)?;
        assert_eq!(repo.id, orig_repo.id);
        assert_eq!(repo.name, orig_repo.name);

        std::fs::remove_file(final_path)?;

        Ok(())
    }
}

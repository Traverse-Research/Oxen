
use crate::db;
use crate::constants::COMMITS_DB;
use crate::error::OxenError;
use crate::index::CommitDBReader;
use crate::model::Commit;
use crate::util;

use rocksdb::{DBWithThreadMode, MultiThreaded};
use std::str;

use crate::model::LocalRepository;


pub struct CommitReader {
    repository: LocalRepository,
    db: DBWithThreadMode<MultiThreaded>,
}

impl CommitReader {
    /// Create a new reader that can find commits, list history, etc
    pub fn new(repository: &LocalRepository) -> Result<CommitReader, OxenError> {
        let db_path = util::fs::oxen_hidden_dir(&repository.path).join(COMMITS_DB);
        let opts = db::opts::default();
        if !db_path.exists() {
            std::fs::create_dir_all(&db_path)?;
            // open it then lose scope to close it
            let _db : DBWithThreadMode<MultiThreaded> = DBWithThreadMode::open(&opts, &db_path)?;
        }
        
        Ok(CommitReader {
            repository: repository.clone(),
            db: DBWithThreadMode::open_for_read_only(&opts, &db_path, false)?,
        })
    }

    /// Return the head commit
    pub fn head_commit(&self) -> Result<Commit, OxenError> {
        CommitDBReader::head_commit(&self.repository, &self.db)
    }

    pub fn root_commit(&self) -> Result<Commit, OxenError> {
        CommitDBReader::root_commit(&self.repository, &self.db)
    }

    /// List the commit history starting at a commit id
    pub fn history_from_commit_id(&self, commit_id: &str) -> Result<Vec<Commit>, OxenError> {
        let mut commits: Vec<Commit> = vec![];

        self.p_list_commits(&commit_id, &mut commits)?;
        Ok(commits)
    }

    /// List the commit history from the HEAD commit
    pub fn history_from_head(&self) -> Result<Vec<Commit>, OxenError> {
        let head_commit = self.head_commit()?;
        CommitDBReader::history_from_commit(&self.db, &head_commit)
    }

    /// See if a commit id exists
    pub fn commit_id_exists(&self, commit_id: &str) -> bool {
        CommitDBReader::commit_id_exists(&self.db, commit_id)
    }

    /// Get a commit object from an ID
    pub fn get_commit_by_id(&self, commit_id: &str) -> Result<Option<Commit>, OxenError> {
        CommitDBReader::get_commit_by_id(&self.db, commit_id)
    }

    fn p_list_commits(&self, commit_id: &str, commits: &mut Vec<Commit>) -> Result<(), OxenError> {
        if let Some(commit) = self.get_commit_by_id(commit_id)? {
            commits.push(commit.clone());
            if let Some(parent_id) = &commit.parent_id {
                self.p_list_commits(parent_id, commits)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::constants::INITIAL_COMMIT_MSG;
    use crate::error::OxenError;
    use crate::index::{CommitReader};
    use crate::test;

    #[test]
    fn test_get_root_commit() -> Result<(), OxenError> {
        test::run_training_data_repo_test_fully_committed(|repo| {
            let commit_reader = CommitReader::new(&repo)?;
            let root_commit = commit_reader.root_commit()?;

            assert_eq!(root_commit.message, INITIAL_COMMIT_MSG);

            Ok(())
        })
    }
}
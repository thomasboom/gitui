//! jj has no stash. The working-copy-as-a-commit model plus the operation
//! log (`jj undo`) covers the old stash workflows.
//!
//! The signatures mirror the old git backend so the UI compiles; the repo
//! simply never has stashes.

use super::commits_info::CommitId;
use crate::{
	error::{Error, Result},
	sync::RepoPath,
};

///
pub fn get_stashes(_repo_path: &RepoPath) -> Result<Vec<CommitId>> {
	Ok(Vec::new())
}

///
pub fn stash_save(
	_repo_path: &RepoPath,
	_message: Option<&str>,
	_include_untracked: bool,
	_keep_index: bool,
) -> Result<CommitId> {
	Err(Error::Unsupported(
		"jj has no stash; use `jj new` to shelve work, `jj undo` to recover".to_string(),
	))
}

///
pub fn stash_apply(
	_repo_path: &RepoPath,
	_stash_id: CommitId,
	_allow_conflicts: bool,
) -> Result<()> {
	Err(Error::Unsupported("jj has no stash".to_string()))
}

///
pub fn stash_drop(
	_repo_path: &RepoPath,
	_stash_id: CommitId,
) -> Result<()> {
	Err(Error::Unsupported("jj has no stash".to_string()))
}

///
pub fn stash_pop(
	_repo_path: &RepoPath,
	_stash_id: CommitId,
) -> Result<()> {
	Err(Error::Unsupported("jj has no stash".to_string()))
}

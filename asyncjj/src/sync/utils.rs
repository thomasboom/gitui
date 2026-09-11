//! Compat utilities.

use super::{commits_info::CommitId, RepoPath};
use crate::{error::Result, sync::operations::get_head};
use std::path::{Path, PathBuf};

///
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Head {
	///
	pub name: String,
	///
	pub id: CommitId,
}

///
pub fn get_head_tuple(repo_path: &RepoPath) -> Result<Head> {
	Ok(Head {
		name: "@".to_string(),
		id: get_head(repo_path)?,
	})
}

/// workspace root
pub fn repo_dir(repo_path: &RepoPath) -> Result<PathBuf> {
	Ok(repo_path.jj_root().to_path_buf())
}

///
pub fn repo_work_dir(repo_path: &RepoPath) -> Result<String> {
	Ok(repo_path.jj_root().to_string_lossy().into_owned())
}

/// jj has no index: staging is a no-op kept for UI compat.
pub fn stage_add_file(
	_repo_path: &RepoPath,
	_path: &Path,
) -> Result<()> {
	Ok(())
}

/// no-op (no staging area)
pub fn stage_add_all(
	_repo_path: &RepoPath,
	_pattern: &str,
	_stage_untracked: Option<super::status::ShowUntrackedFilesConfig>,
) -> Result<()> {
	Ok(())
}

/// no-op
pub fn stage_addremoved(
	_repo_path: &RepoPath,
	_path: &Path,
) -> Result<()> {
	Ok(())
}

#[allow(dead_code)]
pub(crate) fn bytes2string(bytes: &[u8]) -> Result<String> {
	Ok(String::from_utf8(bytes.to_vec())?)
}

/// write a file in the workspace
#[allow(dead_code)]
pub(crate) fn repo_write_file(
	repo_path: &RepoPath,
	file: &str,
	content: &str,
) -> Result<()> {
	let path = repo_path.jj_root().join(file);
	if let Some(parent) = path.parent() {
		std::fs::create_dir_all(parent)?;
	}
	std::fs::write(path, content)?;
	Ok(())
}

///
pub fn read_file(path: &Path) -> Result<String> {
	Ok(std::fs::read_to_string(path)?)
}

///
pub fn get_head_repo_id(repo_path: &RepoPath) -> Result<CommitId> {
	get_head(repo_path)
}

///
pub fn undo_last_commit(repo_path: &RepoPath) -> Result<()> {
	super::operations::undo(repo_path)
}

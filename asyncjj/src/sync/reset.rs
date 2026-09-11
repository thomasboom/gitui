//! Reset equivalents (`jj restore`, `jj abandon`).

use crate::{
	error::Result,
	sync::{
		commits_info::CommitId,
		diff::{DiffLinePosition, DiffOptions},
		jj_cmd::run_jj,
		RepoPath,
	},
};

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetType {
	///
	Soft,
	///
	Mixed,
	///
	Hard,
}

///
pub fn reset_repo(
	repo_path: &RepoPath,
	id: CommitId,
	_reset_type: ResetType,
) -> Result<()> {
	// Approximate `git reset`: move the working copy to the target.
	run_jj(repo_path.jj_root(), &["edit", id.as_str()])?;
	Ok(())
}

///
pub fn reset_stage(_repo_path: &RepoPath, _path: &str) -> Result<()> {
	Ok(())
}

///
pub fn reset_workdir(repo_path: &RepoPath, path: &str) -> Result<()> {
	if path.is_empty() {
		run_jj(repo_path.jj_root(), &["restore", "-r", "@"])?;
	} else {
		run_jj(repo_path.jj_root(), &["restore", path])?;
	}
	Ok(())
}

///
pub fn reset_hunk(
	_repo_path: &RepoPath,
	_path: &str,
	_hunk_hash: u64,
	_options: Option<DiffOptions>,
) -> Result<()> {
	Err(crate::error::Error::Unsupported(
		"hunk reset is not supported in the jj backend".to_string(),
	))
}

///
pub fn stage_hunk(
	_repo_path: &RepoPath,
	_path: &str,
	_hunk_hash: u64,
	_options: Option<DiffOptions>,
) -> Result<()> {
	// No staging area in jj: everything is already tracked in `@`.
	Ok(())
}

///
pub fn unstage_hunk(
	_repo_path: &RepoPath,
	_path: &str,
	_hunk_hash: u64,
	_options: Option<DiffOptions>,
) -> Result<()> {
	Ok(())
}

///
pub fn discard_lines(
	_repo_path: &RepoPath,
	_path: &str,
	_lines: &[DiffLinePosition],
) -> Result<()> {
	Err(crate::error::Error::Unsupported(
		"line discard is not supported in the jj backend".to_string(),
	))
}

///
pub fn stage_lines(
	_repo_path: &RepoPath,
	_path: &str,
	_is_stage: bool,
	_lines: &[DiffLinePosition],
) -> Result<()> {
	Ok(())
}

///
pub fn revert_head(repo_path: &RepoPath) -> Result<CommitId> {
	run_jj(repo_path.jj_root(), &["revert", "-r", "@-"])?;
	CommitId::from_revision(repo_path, "@")
}

///
pub fn revert_commit(
	repo_path: &RepoPath,
	id: CommitId,
) -> Result<CommitId> {
	run_jj(repo_path.jj_root(), &["revert", "-r", id.as_str()])?;
	CommitId::from_revision(repo_path, "@")
}

///
pub fn commit_revert(
	repo_path: &RepoPath,
	msg: &str,
) -> Result<CommitId> {
	crate::sync::commit(repo_path, msg)
}

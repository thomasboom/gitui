//! Merge / rebase equivalents (`jj rebase`, `jj squash`).

use crate::{
	error::{Error, Result},
	sync::{commits_info::CommitId, jj_cmd::run_jj, RepoPath},
};

///
pub fn merge_branch(
	repo_path: &RepoPath,
	branch: &str,
	_branch_type: super::branch::BranchType,
) -> Result<()> {
	// Approximate `git merge <branch>` by rebasing @ onto the bookmark.
	run_jj(
		repo_path.jj_root(),
		&["rebase", "-r", "@", "-d", branch],
	)?;
	Ok(())
}

///
pub fn merge_commit(
	repo_path: &RepoPath,
	msg: &str,
	ids: &[CommitId],
) -> Result<CommitId> {
	let _ = (msg, ids);
	run_jj(repo_path.jj_root(), &["new"])?;
	CommitId::from_revision(repo_path, "@")
}

///
pub fn mergehead_ids(_repo_path: &RepoPath) -> Result<Vec<CommitId>> {
	Ok(Vec::new())
}

///
pub fn abort_pending_state(_repo_path: &RepoPath) -> Result<()> {
	Ok(())
}

///
pub fn abort_pending_rebase(_repo_path: &RepoPath) -> Result<()> {
	Ok(())
}

///
pub fn continue_pending_rebase(_repo_path: &RepoPath) -> Result<()> {
	Ok(())
}

///
pub fn rebase_progress(
	_repo_path: &RepoPath,
) -> Result<RebaseProgress> {
	Ok(RebaseProgress {
		steps: 0,
		current: 0,
		current_commit: None,
	})
}

///
#[derive(PartialEq, Eq, Debug)]
pub struct RebaseProgress {
	///
	pub steps: usize,
	///
	pub current: usize,
	///
	pub current_commit: Option<CommitId>,
}

///
pub fn rebase_branch(
	repo_path: &RepoPath,
	branch: &str,
	_branch_type: super::branch::BranchType,
) -> Result<()> {
	run_jj(repo_path.jj_root(), &["rebase", "-b", branch])?;
	Ok(())
}

///
pub fn branch_merge_upstream_fastforward(
	_repo_path: &RepoPath,
	_branch: &str,
) -> Result<()> {
	Err(Error::Unsupported("fast-forward NYI".to_string()))
}

///
pub fn merge_upstream_commit(
	_repo_path: &RepoPath,
	_branch: &str,
) -> Result<()> {
	Err(Error::Unsupported("merge-upstream NYI".to_string()))
}

///
pub fn merge_upstream_rebase(
	_repo_path: &RepoPath,
	_branch: &str,
) -> Result<()> {
	Err(Error::Unsupported("merge-upstream NYI".to_string()))
}

///
pub fn config_is_pull_rebase(_repo_path: &RepoPath) -> Result<bool> {
	Ok(true)
}

///
pub mod branch {
	use crate::error::Result;

	///
	pub fn checkout_remote_branch(
		repo_path: &super::RepoPath,
		branch: &crate::sync::branch::BranchInfo,
	) -> Result<()> {
		super::run_jj(
			repo_path.jj_root(),
			&["edit", branch.name.as_str()],
		)?;
		Ok(())
	}
}

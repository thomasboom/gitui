//! Repo state (merge/rebase in progress).
//!
//! jj tracks conflicts per-commit rather than via `MERGE_HEAD` files, so
//! there is normally no global "pending state". We report `Ready` unless the
//! working-copy commit itself is conflicted.

use crate::{
	error::Result,
	sync::{jj_cmd::run_jj, RepoPath},
};

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoState {
	///
	Clean,
	///
	Merge,
	///
	Rebase,
	///
	Revert,
	///
	Other,
	///
	Ready,
}

///
pub fn repo_state(repo_path: &RepoPath) -> Result<RepoState> {
	let out = run_jj(
		repo_path.jj_root(),
		&["log", "--no-graph", "-r", "@", "-T", "conflict"],
	)?;
	if out.trim() == "true" {
		Ok(RepoState::Merge)
	} else {
		Ok(RepoState::Clean)
	}
}

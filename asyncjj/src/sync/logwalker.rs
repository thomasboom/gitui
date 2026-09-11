//! Revision walking via `jj log`.
//!
//! The revset defaults to mutable revisions; callers can override with
//! [`LogWalker::set_revset`].

use crate::{
	error::Result,
	sync::{commits_info::CommitId, jj_cmd::run_jj, RepoPath},
};
use scopetime::scope_time;

/// Walk revisions newest-first, yielding [`CommitId`]s.
pub struct LogWalker {
	repo: RepoPath,
	limit: usize,
	revset: String,
}

impl LogWalker {
	/// new walker over `revsets.log` default
	pub fn new(repo_path: &RepoPath, limit: usize) -> Result<Self> {
		Ok(Self {
			repo: repo_path.clone(),
			limit,
			revset: "::@".to_string(),
		})
	}

	/// override revset (e.g. `all()`, `bookmarks()`, file history)
	pub fn set_revset(&mut self, revset: impl Into<String>) {
		self.revset = revset.into();
	}

	/// read up to `limit` ids into `out`
	pub fn read(&self, out: &mut Vec<CommitId>) -> Result<usize> {
		scope_time!("LogWalker::read");
		let limit = self.limit.to_string();
		let raw = run_jj(
			self.repo.jj_root(),
			&[
				"log",
				"--no-graph",
				"-r",
				self.revset.as_str(),
				"-T",
				"commit_id ++ \"\\n\"",
				"--limit",
				limit.as_str(),
			],
		)?;
		out.clear();
		for line in raw.lines() {
			let line = line.trim();
			if line.is_empty() {
				continue;
			}
			out.push(CommitId::from_str_unchecked(line)?);
		}
		Ok(out.len())
	}
}

/// Variant without path filtering (kept for API compat).
pub struct LogWalkerWithoutFilter(LogWalker);

impl LogWalkerWithoutFilter {
	///
	pub fn new(repo_path: &RepoPath, limit: usize) -> Result<Self> {
		Ok(Self(LogWalker::new(repo_path, limit)?))
	}

	///
	pub fn read(&self, out: &mut Vec<CommitId>) -> Result<usize> {
		self.0.read(out)
	}
}

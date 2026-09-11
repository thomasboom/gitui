//! Commits, describes, and history rewriting.
//!
//! Mapping notes (see <https://docs.jj-vcs.dev/latest/git-comparison/>):
//! * `commit()` on a clean `@` with a message = `jj describe` (keeps the
//!   same change) — there is no staging area to consume.
//! * amending = `jj describe` / `jj squash`.
//! * `jj new` creates the next working-copy change.

use crate::{
	error::Result,
	sync::{commits_info::CommitId, jj_cmd::run_jj, RepoPath},
};
use scopetime::scope_time;

///
pub fn commit(repo_path: &RepoPath, msg: &str) -> Result<CommitId> {
	scope_time!("commit");
	// `jj commit` describes @ and creates a new change on top, leaving a
	// clean working copy — the equivalent of `git commit`.
	run_jj(repo_path.jj_root(), &["commit", "-m", msg])?;
	CommitId::from_revision(repo_path, "@-")
}

///
pub fn amend(
	repo_path: &RepoPath,
	id: CommitId,
	msg: &str,
) -> Result<CommitId> {
	let rev = id.to_string();
	describe(repo_path, rev.as_str(), msg)
}

///
pub fn describe(
	repo_path: &RepoPath,
	rev: &str,
	msg: &str,
) -> Result<CommitId> {
	run_jj(repo_path.jj_root(), &["describe", "-r", rev, "-m", msg])?;
	CommitId::from_revision(repo_path, rev)
}

///
pub fn new_change(
	repo_path: &RepoPath,
	msg: Option<&str>,
) -> Result<CommitId> {
	let mut args = vec!["new"];
	let mut owned = Vec::new();
	if let Some(m) = msg {
		args.push("-m");
		owned.push(m.to_string());
	}
	let mut full: Vec<&str> = args;
	for o in &owned {
		full.push(o.as_str());
	}
	run_jj(repo_path.jj_root(), &full)?;
	CommitId::from_revision(repo_path, "@")
}

///
pub fn abandon(repo_path: &RepoPath, rev: &str) -> Result<()> {
	run_jj(repo_path.jj_root(), &["abandon", rev])?;
	Ok(())
}

///
pub fn squash(
	repo_path: &RepoPath,
	from_rev: &str,
	into_rev: &str,
) -> Result<()> {
	run_jj(
		repo_path.jj_root(),
		&["squash", "--from", from_rev, "--into", into_rev],
	)?;
	Ok(())
}

///
pub fn reword(
	repo_path: &RepoPath,
	commit: CommitId,
	msg: &str,
) -> Result<CommitId> {
	describe(repo_path, commit.as_str(), msg)
}

///
pub fn tag_commit(
	repo_path: &RepoPath,
	id: &CommitId,
	tag: &str,
	_annotation: Option<&str>,
) -> Result<()> {
	run_jj(
		repo_path.jj_root(),
		&["tag", "set", tag, "-r", id.as_str()],
	)?;
	Ok(())
}

///
pub fn commit_message_prettify(
	_repo_path: &RepoPath,
	msg: String,
) -> Result<String> {
	Ok(msg.trim().to_string())
}

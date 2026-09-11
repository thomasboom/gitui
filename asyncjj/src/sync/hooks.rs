//! Hooks do not exist in jj. Stubs for UI compat.

use super::RepoPath;
use crate::error::Result;

///
#[derive(Debug, PartialEq, Eq)]
pub enum HookResult {
	/// Everything went fine
	Ok,
	/// Hook returned error
	NotOk(String),
}

///
#[derive(Debug, Clone)]
pub enum PrepareCommitMsgSource {
	///
	Message,
	///
	Template,
	///
	Merge,
	///
	Squash,
	///
	Commit(super::commits_info::CommitId),
}

///
pub enum PrePushTarget<'a> {
	/// Push a single branch.
	Branch {
		/// Local branch name being pushed.
		branch: &'a str,
		/// Whether this is a delete push.
		delete: bool,
	},
	/// Push tags.
	Tags,
}

///
pub fn hooks_pre_commit(_repo_path: &RepoPath) -> Result<HookResult> {
	Ok(HookResult::Ok)
}

///
pub fn hooks_commit_msg(
	_repo_path: &RepoPath,
	_msg: &mut String,
) -> Result<HookResult> {
	Ok(HookResult::Ok)
}

///
pub fn hooks_post_commit(
	_repo_path: &RepoPath,
) -> Result<HookResult> {
	Ok(HookResult::Ok)
}

///
pub fn hooks_prepare_commit_msg(
	_repo_path: &RepoPath,
	_source: PrepareCommitMsgSource,
	_msg: &mut String,
) -> Result<HookResult> {
	Ok(HookResult::Ok)
}

///
pub fn hooks_pre_push(
	_repo_path: &RepoPath,
	_remote: &str,
	_target: &PrePushTarget<'_>,
	_cred: Option<super::cred::BasicAuthCredential>,
) -> Result<HookResult> {
	Ok(HookResult::Ok)
}

///
pub fn merge_msg(_repo_path: &RepoPath) -> Result<String> {
	Ok(String::new())
}

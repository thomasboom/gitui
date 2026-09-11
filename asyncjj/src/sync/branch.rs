//! Bookmarks (the jj equivalent of git branches) plus a
//! branch-shaped compat layer so the existing branch UI keeps compiling
//! while it migrates to jj-native concepts.
//!
//! Mapping: local bookmarks → [`BranchDetails::Local`], remote bookmarks →
//! [`BranchDetails::Remote`]. There is no "checked out branch" in jj (the
//! working-copy commit `@` is anonymous by default); `is_head` is true when
//! the bookmark points at `@`.

use crate::{
	error::Result,
	sync::{
		commits_info::{get_commit_info, CommitId},
		jj_cmd::run_jj,
		RepoPath,
	},
};
use scopetime::scope_time;

///
#[derive(Clone, Debug)]
pub struct LocalBranch {
	///
	pub is_head: bool,
	///
	pub has_upstream: bool,
	///
	pub upstream: Option<UpstreamBranch>,
	///
	pub remote: Option<String>,
}

///
#[derive(Clone, Debug)]
pub struct UpstreamBranch {
	///
	pub reference: String,
}

///
#[derive(Clone, Debug)]
pub struct RemoteBranch {
	///
	pub has_tracking: bool,
}

///
#[derive(Clone, Debug)]
pub enum BranchDetails {
	///
	Local(LocalBranch),
	///
	Remote(RemoteBranch),
}

///
#[derive(Clone, Debug)]
pub struct BranchInfo {
	///
	pub name: String,
	///
	pub reference: String,
	///
	pub top_commit_message: String,
	///
	pub top_commit: CommitId,
	///
	pub details: BranchDetails,
}

impl BranchInfo {
	/// returns details about local branch or None
	pub const fn local_details(&self) -> Option<&LocalBranch> {
		if let BranchDetails::Local(details) = &self.details {
			return Some(details);
		}

		None
	}

	/// returns whether branch is local
	pub const fn is_local(&self) -> bool {
		matches!(self.details, BranchDetails::Local(_))
	}
}

/// local vs remote bookmark (compat for the branch UI)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchType {
	///
	Local,
	///
	Remote,
}

/// jj-native alias.
pub type BookmarkInfo = BranchInfo;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BranchCompare {
	///
	pub ahead: usize,
	///
	pub behind: usize,
}

const LIST_TEMPLATE: &str = "name ++ \"\\x1f\" ++ if(remote, remote, \"\") ++ \"\\x1f\" ++ if(tracked, \"t\", \"\") ++ \"\\x1f\" ++ if(conflict, \"c\", \"\") ++ \"\\x1f\" ++ self.normal_target().commit_id() ++ \"\\x1e\"";

fn output_records(out: &str) -> Vec<&str> {
	out.split('\x1e')
		.map(str::trim)
		.filter(|s| !s.is_empty())
		.collect()
}

fn describe_target(
	repo_path: &RepoPath,
	target: &str,
) -> (CommitId, String) {
	let id = CommitId::from_str_unchecked(target).unwrap_or_default();
	let message = get_commit_info(repo_path, &id)
		.map(|c| {
			c.message.lines().next().unwrap_or_default().to_string()
		})
		.unwrap_or_default();
	(id, message)
}

///
pub fn get_branches_info(
	repo_path: &RepoPath,
	local_only: bool,
) -> Result<Vec<BranchInfo>> {
	scope_time!("get_branches_info");
	let args = vec!["bookmark", "list", "-T", LIST_TEMPLATE];
	let out = run_jj(repo_path.jj_root(), &args)?;
	let head = CommitId::from_revision(repo_path, "@").ok();

	let mut res = Vec::new();
	for record in output_records(&out) {
		let mut parts = record.split('\x1f');
		let (
			Some(name),
			Some(remote),
			Some(tracked),
			Some(_conflict),
			Some(target),
		) = (
			parts.next(),
			parts.next(),
			parts.next(),
			parts.next(),
			parts.next(),
		)
		else {
			continue;
		};
		let name = name.trim().to_string();
		if name.is_empty() {
			continue;
		}
		let remote = remote.trim();
		let is_remote = !remote.is_empty();
		if local_only && is_remote {
			continue;
		}
		let target = target.trim();
		if target.is_empty() {
			continue;
		}
		let (id, message) = describe_target(repo_path, target);
		let details = if is_remote {
			BranchDetails::Remote(RemoteBranch {
				has_tracking: tracked.trim() == "t",
			})
		} else {
			BranchDetails::Local(LocalBranch {
				is_head: head.as_ref() == Some(&id),
				has_upstream: tracked.trim() == "t",
				upstream: None,
				remote: None,
			})
		};
		res.push(BranchInfo {
			reference: name.clone(),
			name,
			top_commit_message: message,
			top_commit: id,
			details,
		});
	}
	Ok(res)
}

///
pub fn get_bookmarks(
	repo_path: &RepoPath,
) -> Result<Vec<BranchInfo>> {
	get_branches_info(repo_path, false)
}

///
pub fn validate_branch_name(name: &str) -> Result<bool> {
	Ok(!name.trim().is_empty()
		&& !name.contains(char::is_whitespace)
		&& !name.contains(".."))
}

///
pub fn get_branch_name(repo_path: &RepoPath) -> Result<String> {
	// No checked-out branch in jj; report a bookmark on @ if exactly one exists.
	let out = run_jj(
		repo_path.jj_root(),
		&[
			"log",
			"--no-graph",
			"-r",
			"@",
			"-T",
			"bookmarks.join(\",\")",
		],
	)?;
	Ok(out.trim().to_string())
}

///
pub fn get_branch_remote(
	_repo_path: &RepoPath,
	_branch: &str,
) -> Result<Option<String>> {
	Ok(None)
}

///
pub fn get_branch_upstream_merge(
	_repo_path: &RepoPath,
	_branch: &str,
) -> Result<Option<String>> {
	Ok(None)
}

///
pub fn branch_compare_upstream(
	_repo_path: &RepoPath,
	_branch: &str,
) -> Result<BranchCompare> {
	Ok(BranchCompare::default())
}

///
pub fn create_branch(repo_path: &RepoPath, name: &str) -> Result<()> {
	let head = CommitId::from_revision(repo_path, "@")?;
	create_bookmark(repo_path, name, &head)
}

///
pub fn create_bookmark(
	repo_path: &RepoPath,
	name: &str,
	target: &CommitId,
) -> Result<()> {
	run_jj(
		repo_path.jj_root(),
		&["bookmark", "create", name, "-r", target.as_str()],
	)?;
	Ok(())
}

///
pub fn delete_branch(repo_path: &RepoPath, name: &str) -> Result<()> {
	delete_bookmark(repo_path, name)
}

///
pub fn delete_bookmark(
	repo_path: &RepoPath,
	name: &str,
) -> Result<()> {
	run_jj(repo_path.jj_root(), &["bookmark", "delete", name])?;
	Ok(())
}

///
pub fn rename_branch(
	repo_path: &RepoPath,
	old: &str,
	new: &str,
) -> Result<()> {
	run_jj(repo_path.jj_root(), &["bookmark", "rename", old, new])?;
	Ok(())
}

///
pub fn checkout_branch(
	repo_path: &RepoPath,
	name: &str,
) -> Result<()> {
	run_jj(repo_path.jj_root(), &["edit", name])?;
	Ok(())
}

///
pub fn checkout_commit(
	repo_path: &RepoPath,
	id: CommitId,
) -> Result<()> {
	run_jj(repo_path.jj_root(), &["edit", id.as_str()])?;
	Ok(())
}

///
pub fn checkout_remote_branch(
	repo_path: &RepoPath,
	branch: &BranchInfo,
) -> Result<()> {
	run_jj(repo_path.jj_root(), &["edit", branch.name.as_str()])?;
	Ok(())
}

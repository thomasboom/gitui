//! Operation log, undo/redo, and working-copy metadata.
//!
//! The op log (`jj op log`) replaces git's reflog; `jj undo` / `jj redo`
//! replace stash-based "safety nets".

use super::utils::Head;
use crate::{
	error::Result,
	sync::{commits_info::CommitId, jj_cmd::run_jj, RepoPath},
};
use scopetime::scope_time;

///
#[derive(Debug, Clone)]
pub struct OperationInfo {
	///
	pub id: String,
	///
	pub description: String,
	///
	pub current: bool,
}

const OP_TEMPLATE: &str = "id.short(16) ++ \"\\x1f\" ++ description.first_line() ++ \"\\x1f\" ++ if(current_operation, \"current\", \"\") ++ \"\\x1e\"";

///
pub fn get_operations(
	repo_path: &RepoPath,
	limit: usize,
) -> Result<Vec<OperationInfo>> {
	scope_time!("get_operations");
	let out = run_jj(
		repo_path.jj_root(),
		&[
			"op",
			"log",
			"--no-graph",
			"-T",
			OP_TEMPLATE,
			"-n",
			limit.to_string().as_str(),
		],
	)?;
	let mut res = Vec::new();
	for record in out.split('\x1e') {
		let record = record.trim();
		if record.is_empty() {
			continue;
		}
		let mut parts = record.splitn(3, '\x1f');
		let (Some(id), Some(desc), Some(cur)) =
			(parts.next(), parts.next(), parts.next())
		else {
			continue;
		};
		res.push(OperationInfo {
			id: id.trim().to_string(),
			description: desc.trim().to_string(),
			current: cur.trim() == "current",
		});
	}
	Ok(res)
}

///
pub fn undo(repo_path: &RepoPath) -> Result<()> {
	run_jj(repo_path.jj_root(), &["undo"])?;
	Ok(())
}

///
pub fn redo(repo_path: &RepoPath) -> Result<()> {
	run_jj(repo_path.jj_root(), &["redo"])?;
	Ok(())
}

///
pub fn restore_operation(
	repo_path: &RepoPath,
	op: &str,
) -> Result<()> {
	run_jj(repo_path.jj_root(), &["op", "restore", op])?;
	Ok(())
}

///
pub fn undo_last_commit(repo_path: &RepoPath) -> Result<()> {
	undo(repo_path)
}

///
pub fn get_head(repo_path: &RepoPath) -> Result<CommitId> {
	CommitId::from_revision(repo_path, "@")
}

///
pub fn get_head_tuple(repo_path: &RepoPath) -> Result<Head> {
	Ok(Head {
		name: "@".to_string(),
		id: get_head(repo_path)?,
	})
}

/// Path to the workspace root (compat for `repo_dir`).
pub fn repo_dir(repo_path: &RepoPath) -> Result<std::path::PathBuf> {
	Ok(repo_path.jj_root().to_path_buf())
}

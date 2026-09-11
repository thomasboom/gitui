//! Files changed in a revision.

use super::{
	commits_info::CommitId,
	status::{StatusItem, StatusItemType},
};
use crate::{
	error::Result,
	sync::{jj_cmd::run_jj, RepoPath},
};

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OldNew<T> {
	///
	pub old: T,
	///
	pub new: T,
}

///
pub fn get_commit_files(
	repo_path: &RepoPath,
	id: CommitId,
	_other: Option<CommitId>,
) -> Result<Vec<StatusItem>> {
	let out = run_jj(
		repo_path.jj_root(),
		&["diff", "--summary", "-r", id.as_str()],
	)?;
	let mut res = Vec::new();
	for line in out.lines() {
		let line = line.trim();
		if line.is_empty() {
			continue;
		}
		let mut parts = line.splitn(2, char::is_whitespace);
		let (Some(kind), Some(path)) = (parts.next(), parts.next())
		else {
			continue;
		};
		let status = match kind {
			"A" => StatusItemType::New,
			"D" => StatusItemType::Deleted,
			"R" => StatusItemType::Renamed,
			"C" => StatusItemType::Conflicted,
			"T" => StatusItemType::Typechange,
			_ => StatusItemType::Modified,
		};
		res.push(StatusItem {
			path: path.trim().to_string(),
			status,
		});
	}
	Ok(res)
}

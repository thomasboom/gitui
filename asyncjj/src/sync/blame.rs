//! Blame via `jj file annotate`.

use super::commits_info::CommitId;
use crate::{
	error::Result,
	sync::{jj_cmd::run_jj, RepoPath},
};

/// A `BlameHunk` contains all the information that will be shown to the user.
#[derive(Clone, Hash, Debug, PartialEq, Eq)]
pub struct BlameHunk {
	///
	pub commit_id: CommitId,
	///
	pub author: String,
	///
	pub time: i64,
	/// 0-based start line
	pub start_line: usize,
	///
	pub end_line: usize,
}

/// A `BlameFile` represents a collection of lines. This is targeted at how the
/// data will be used by the UI.
#[derive(Clone, Debug)]
pub struct FileBlame {
	///
	pub commit_id: CommitId,
	///
	pub path: String,
	///
	pub lines: Vec<(Option<BlameHunk>, String)>,
}

///
pub fn blame_file(
	repo_path: &RepoPath,
	file_path: &str,
	commit_id: Option<CommitId>,
) -> Result<FileBlame> {
	let rev =
		commit_id.map_or_else(|| "@".to_string(), |c| c.to_string());
	let out = run_jj(
		repo_path.jj_root(),
		&[
			"file",
			"annotate",
			"-r",
			rev.as_str(),
			"-T",
			"commit.commit_id() ++ \"\\x1f\" ++ commit.author().name() ++ \"\\n\"",
			file_path,
		],
	)?;

	// Fallback: plain content without template support.
	let content = if out.trim().is_empty() {
		run_jj(
			repo_path.jj_root(),
			&["file", "show", "-r", rev.as_str(), file_path],
		)?
	} else {
		String::new()
	};

	let head = CommitId::from_revision(repo_path, rev.as_str())
		.unwrap_or_default();

	let mut lines = Vec::new();
	if content.is_empty() {
		for (idx, line) in out.lines().enumerate() {
			let mut parts = line.splitn(2, '\x1f');
			let id = parts.next().unwrap_or("").trim();
			let text = parts.next().unwrap_or("").to_string();
			let hunk = if id.is_empty() {
				None
			} else {
				Some(BlameHunk {
					commit_id: CommitId::from_str_unchecked(id)
						.unwrap_or_default(),
					author: String::new(),
					time: 0,
					start_line: idx,
					end_line: idx,
				})
			};
			lines.push((hunk, text));
		}
	} else {
		for text in content.lines() {
			lines.push((None, text.to_string()));
		}
	}

	Ok(FileBlame {
		commit_id: head,
		path: file_path.to_string(),
		lines,
	})
}

//! `jj status` wrapper.
//!
//! Jujutsu has no staging area: the working-copy commit (`@`) auto-snapshots.
//! We therefore report all working-copy changes as [`StatusType::WorkingDir`]
//! and return an empty list for [`StatusType::Stage`] so existing consumers
//! keep working while the UI migrates to jj-native concepts.

use crate::{
	error::Result,
	sync::{jj_cmd::run_jj, RepoPath},
};
use scopetime::scope_time;
use std::path::Path;

///
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum StatusItemType {
	///
	New,
	///
	Modified,
	///
	Deleted,
	///
	Renamed,
	///
	Typechange,
	///
	Conflicted,
}

///
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct StatusItem {
	///
	pub path: String,
	///
	pub status: StatusItemType,
}

///
#[derive(Copy, Clone, Default, Hash, PartialEq, Eq, Debug)]
pub enum StatusType {
	///
	#[default]
	WorkingDir,
	///
	Stage,
	///
	Both,
}

///
#[derive(
	Copy,
	Clone,
	Debug,
	Default,
	PartialEq,
	Eq,
	Hash,
	serde::Serialize,
	serde::Deserialize,
)]
pub enum ShowUntrackedFilesConfig {
	///
	#[default]
	No,
	///
	Normal,
	///
	All,
}

impl ShowUntrackedFilesConfig {
	///
	pub const fn include_none(self) -> bool {
		matches!(self, Self::No)
	}

	///
	pub const fn include_untracked(self) -> bool {
		matches!(self, Self::Normal | Self::All)
	}

	///
	pub const fn recurse_untracked_dirs(self) -> bool {
		matches!(self, Self::All)
	}
}

///
pub fn is_workdir_clean(
	repo_path: &RepoPath,
	_show_untracked: Option<ShowUntrackedFilesConfig>,
) -> Result<bool> {
	Ok(get_status(repo_path, StatusType::WorkingDir, None)?
		.is_empty())
}

fn parse_summary_line(line: &str) -> Option<StatusItem> {
	let line = line.trim();
	if line.is_empty() {
		return None;
	}
	// `jj diff --summary` prints `<STATUS> <path>`, e.g. `M foo/bar`.
	// Renames show as `R old -> new`; we report the new path.
	let mut parts = line.splitn(2, char::is_whitespace);
	let status = parts.next()?.trim();
	let rest = parts.next()?.trim();
	if rest.is_empty() {
		return None;
	}
	let (kind, path) = match status {
		"A" => (StatusItemType::New, rest.to_string()),
		"D" => (StatusItemType::Deleted, rest.to_string()),
		"R" => {
			let target =
				rest.split("->").last().map_or(rest, str::trim);
			(StatusItemType::Renamed, target.to_string())
		}
		"C" => (StatusItemType::Conflicted, rest.to_string()),
		"T" => (StatusItemType::Typechange, rest.to_string()),
		_ => (StatusItemType::Modified, rest.to_string()),
	};
	Some(StatusItem { path, status: kind })
}

/// guarantees sorting
pub fn get_status(
	repo_path: &RepoPath,
	status_type: StatusType,
	_show_untracked: Option<ShowUntrackedFilesConfig>,
) -> Result<Vec<StatusItem>> {
	scope_time!("get_status");

	if matches!(status_type, StatusType::Stage) {
		// No staging area in jj.
		return Ok(Vec::new());
	}

	let out = run_jj(
		repo_path.jj_root(),
		&["diff", "--summary", "-r", "@"],
	)?;
	let mut res: Vec<StatusItem> =
		out.lines().filter_map(parse_summary_line).collect();

	res.sort_by(|a, b| {
		Path::new(a.path.as_str()).cmp(Path::new(b.path.as_str()))
	});

	Ok(res)
}

/// Discard all working-copy changes (`jj restore`).
pub fn discard_status(repo_path: &RepoPath) -> Result<bool> {
	run_jj(repo_path.jj_root(), &["restore", "-r", "@"])?;
	Ok(true)
}

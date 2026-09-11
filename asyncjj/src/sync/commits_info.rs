use super::RepoPath;
use crate::{
	error::{Error, Result},
	sync::jj_cmd::run_jj,
};
use scopetime::scope_time;
use std::fmt::Display;
use unicode_truncate::UnicodeTruncateStr;

/// Identifies a single jj commit (commit id, hex).
///
/// In jj every commit additionally has a *change id* (see [`ChangeId`]);
/// the commit id identifies the snapshot, the change id tracks the logical
/// change across rewrites.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct CommitId(String);

/// Logical change identifier (jj-native, e.g. `zztprrms...`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct ChangeId(String);

impl Default for CommitId {
	fn default() -> Self {
		Self("0".repeat(40))
	}
}

impl CommitId {
	/// create new `CommitId` from a hex string
	pub fn new(id: impl Into<String>) -> Self {
		Self(id.into())
	}

	/// raw hex string
	pub fn as_str(&self) -> &str {
		&self.0
	}

	/// 7 chars short hash
	pub fn get_short_string(&self) -> String {
		self.0.chars().take(7).collect()
	}

	/// Resolve a revset (`@`, `@-`, bookmark name, prefix, …) to a commit.
	pub fn from_revision(
		repo_path: &RepoPath,
		revision: &str,
	) -> Result<Self> {
		scope_time!("CommitId::from_revision");
		let out = run_jj(
			repo_path.jj_root(),
			&[
				"log",
				"--no-graph",
				"-r",
				revision,
				"-T",
				"commit_id",
				"--limit",
				"1",
			],
		)?;
		let id = out.trim();
		if id.is_empty() {
			return Err(Error::Generic(format!(
				"revision `{revision}` not found"
			)));
		}
		Ok(Self::new(id))
	}

	/// Convert a hex string without validation against the repo.
	pub fn from_str_unchecked(commit_id_str: &str) -> Result<Self> {
		let s = commit_id_str.trim();
		if s.is_empty() {
			return Err(Error::Generic(
				"empty commit id".to_string(),
			));
		}
		Ok(Self::new(s))
	}
}

impl Display for CommitId {
	fn fmt(
		&self,
		f: &mut std::fmt::Formatter<'_>,
	) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl From<String> for CommitId {
	fn from(s: String) -> Self {
		Self(s)
	}
}

impl ChangeId {
	/// create new `ChangeId`
	pub fn new(id: impl Into<String>) -> Self {
		Self(id.into())
	}

	/// raw id string
	pub fn as_str(&self) -> &str {
		&self.0
	}

	/// short prefix
	pub fn get_short_string(&self) -> String {
		self.0.chars().take(8).collect()
	}
}

impl Display for ChangeId {
	fn fmt(
		&self,
		f: &mut std::fmt::Formatter<'_>,
	) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}

///
#[derive(Debug, Clone)]
pub struct CommitInfo {
	///
	pub message: String,
	///
	pub time: i64,
	///
	pub author: String,
	///
	pub id: CommitId,
	/// jj-native logical change id
	pub change_id: ChangeId,
}

const LOG_TEMPLATE: &str = "commit_id ++ \"\\x1f\" ++ change_id ++ \"\\x1f\" ++ author.name() ++ \"\\x1f\" ++ committer.timestamp().format(\"%s\") ++ \"\\x1f\" ++ description.first_line() ++ \"\\x1e\"";

fn parse_log(output: &str, limit: usize) -> Vec<CommitInfo> {
	let mut res = Vec::new();
	for record in output.split('\x1e') {
		let record = record.trim();
		if record.is_empty() {
			continue;
		}
		let mut parts = record.splitn(5, '\x1f');
		let (
			Some(commit),
			Some(change),
			Some(author),
			Some(time),
			Some(msg),
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
		res.push(CommitInfo {
			message: msg.unicode_truncate(limit).0.to_string(),
			author: author.to_string(),
			time: time.trim().parse().unwrap_or(0),
			id: CommitId::new(commit.trim()),
			change_id: ChangeId::new(change.trim()),
		});
	}
	res
}

///
pub fn get_commits_info(
	repo_path: &RepoPath,
	ids: &[CommitId],
	message_length_limit: usize,
) -> Result<Vec<CommitInfo>> {
	scope_time!("get_commits_info");

	if ids.is_empty() {
		return Ok(Vec::new());
	}

	// Resolve each id individually via its own revset so order is preserved.
	let mut res = Vec::with_capacity(ids.len());
	for id in ids {
		res.push(get_commit_info(repo_path, id)?);
	}
	// Apply first-line + truncate like the old backend did.
	for c in &mut res {
		c.message = c
			.message
			.lines()
			.next()
			.unwrap_or_default()
			.unicode_truncate(message_length_limit)
			.0
			.to_string();
	}
	Ok(res)
}

///
pub fn get_commit_info(
	repo_path: &RepoPath,
	commit_id: &CommitId,
) -> Result<CommitInfo> {
	scope_time!("get_commit_info");

	let out = run_jj(
		repo_path.jj_root(),
		&[
			"log",
			"--no-graph",
			"-r",
			commit_id.as_str(),
			"-T",
			LOG_TEMPLATE,
			"--limit",
			"1",
		],
	)?;
	let mut v = parse_log(&out, usize::MAX);
	v.pop().ok_or_else(|| {
		Error::Generic(format!(
			"commit {} not found",
			commit_id.as_str()
		))
	})
}

/// First line of a description, truncated.
pub fn first_line_limited(msg: &str, limit: Option<usize>) -> String {
	let line = msg.lines().next().unwrap_or_default().trim();
	limit.map_or_else(
		|| line.to_string(),
		|l| line.unicode_truncate(l).0.to_string(),
	)
}

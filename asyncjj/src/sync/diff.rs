//! Diff support backed by `jj diff --git`.
//!
//! Output is git-format diff, parsed into the same [`FileDiff`] shape the
//! git backend produced so the UI keeps working.

use super::commit_files::OldNew;
use crate::{
	error::Result,
	hash,
	sync::{commits_info::CommitId, jj_cmd::run_jj, RepoPath},
};
use scopetime::scope_time;
use serde::{Deserialize, Serialize};

/// type of diff of a single line
#[derive(Copy, Clone, Default, PartialEq, Eq, Hash, Debug)]
pub enum DiffLineType {
	/// just surrounding line, no change
	#[default]
	None,
	/// header of the hunk
	Header,
	/// line added
	Add,
	/// line deleted
	Delete,
}

///
#[derive(Clone, Copy, Default, Hash, Debug, PartialEq, Eq)]
pub struct DiffLinePosition {
	///
	pub old_lineno: Option<u32>,
	///
	pub new_lineno: Option<u32>,
}

///
#[derive(Default, Clone, Hash, Debug)]
pub struct DiffLine {
	///
	pub content: Box<str>,
	///
	pub line_type: DiffLineType,
	///
	pub position: DiffLinePosition,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Hash)]
pub(crate) struct HunkHeader {
	pub old_start: u32,
	pub old_lines: u32,
	pub new_start: u32,
	pub new_lines: u32,
}

/// single diff hunk
#[derive(Default, Clone, Hash, Debug)]
pub struct Hunk {
	/// hash of the hunk header
	pub header_hash: u64,
	/// list of `DiffLine`s
	pub lines: Vec<DiffLine>,
}

/// collection of hunks, sum of all diff lines
#[derive(Default, Clone, Hash, Debug)]
pub struct FileDiff {
	/// list of hunks
	pub hunks: Vec<Hunk>,
	/// lines total summed up over hunks
	pub lines: usize,
	///
	pub untracked: bool,
	/// old and new file size in bytes
	pub sizes: (u64, u64),
	/// size delta in bytes
	pub size_delta: i64,
}

/// diff display options (subset honored: passed as `--context`).
#[derive(
	Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize,
)]
pub struct DiffOptions {
	/// ignore whitespace (`-w`)
	pub ignore_whitespace: bool,
	/// context lines
	pub context: u32,
	/// interhunk lines (unused by jj)
	pub interhunk_lines: u32,
}

impl Default for DiffOptions {
	fn default() -> Self {
		Self {
			ignore_whitespace: false,
			context: 3,
			interhunk_lines: 0,
		}
	}
}

fn jj_diff_args(
	rev: &str,
	path: &str,
	options: Option<DiffOptions>,
	extra: &mut Vec<String>,
) -> Vec<String> {
	let mut args = vec![
		"diff".to_string(),
		"--git".to_string(),
		"-r".to_string(),
		rev.to_string(),
	];
	if let Some(o) = options {
		args.push("--context".to_string());
		args.push(o.context.to_string());
		if o.ignore_whitespace {
			args.push("--ignore-all-space".to_string());
		}
	}
	if !path.is_empty() {
		args.push("--".to_string());
		args.push(path.to_string());
	}
	args.append(extra);
	args
}

/// returns diff of a specific file either in `stage` or workdir.
///
/// `stage` is ignored (jj has no staging area); always diffs `@`.
pub fn get_diff(
	repo_path: &RepoPath,
	p: &str,
	_stage: bool,
	options: Option<DiffOptions>,
) -> Result<FileDiff> {
	scope_time!("get_diff");
	let args = jj_diff_args("@", p, options, &mut Vec::new());
	let refs: Vec<&str> = args.iter().map(String::as_str).collect();
	let raw = run_jj(repo_path.jj_root(), &refs)?;
	Ok(parse_git_diff(&raw))
}

/// returns diff of a specific file inside a commit
pub fn get_diff_commit(
	repo_path: &RepoPath,
	id: CommitId,
	p: String,
	options: Option<DiffOptions>,
) -> Result<FileDiff> {
	scope_time!("get_diff_commit");
	let args = jj_diff_args(
		id.as_str(),
		p.as_str(),
		options,
		&mut Vec::new(),
	);
	let refs: Vec<&str> = args.iter().map(String::as_str).collect();
	let raw = run_jj(repo_path.jj_root(), &refs)?;
	Ok(parse_git_diff(&raw))
}

/// get file changes of a diff between two commits
pub fn get_diff_commits(
	repo_path: &RepoPath,
	ids: OldNew<CommitId>,
	p: String,
	options: Option<DiffOptions>,
) -> Result<FileDiff> {
	scope_time!("get_diff_commits");
	let from = ids.old.to_string();
	let to = ids.new.to_string();
	let mut extra =
		vec!["--from".to_string(), from, "--to".to_string(), to];
	let args = jj_diff_args("@", p.as_str(), options, &mut extra);
	let refs: Vec<&str> = args.iter().map(String::as_str).collect();
	let raw = run_jj(repo_path.jj_root(), &refs)?;
	Ok(parse_git_diff(&raw))
}

/// Parse `git`-format diff into hunks.
pub fn parse_git_diff(raw: &str) -> FileDiff {
	let mut diff = FileDiff::default();
	let mut current_lines: Vec<DiffLine> = Vec::new();
	let mut current_header: Option<HunkHeader> = None;
	let mut old_line: u32 = 0;
	let mut new_line: u32 = 0;

	let flush = |diff: &mut FileDiff,
	             header: &mut Option<HunkHeader>,
	             lines: &mut Vec<DiffLine>| {
		if lines.is_empty() {
			return;
		}
		let h = header.take().unwrap_or_default();
		diff.hunks.push(Hunk {
			header_hash: hash(&h),
			lines: std::mem::take(lines),
		});
	};

	for line in raw.lines() {
		if let Some(header) = parse_hunk_header(line) {
			flush(&mut diff, &mut current_header, &mut current_lines);
			// header line itself is shown as first hunk line
			old_line = header.old_start;
			new_line = header.new_start;
			current_header = Some(header);
			current_lines.push(DiffLine {
				content: line.into(),
				line_type: DiffLineType::Header,
				position: DiffLinePosition {
					old_lineno: None,
					new_lineno: None,
				},
			});
			continue;
		}
		if current_header.is_none() {
			// file headers (`diff --git`, `---`, `+++`, …) are skipped;
			// sizes tracked roughly.
			continue;
		}
		let (kind, content) = match line.chars().next() {
			Some('+') => (DiffLineType::Add, &line[1..]),
			Some('-') => (DiffLineType::Delete, &line[1..]),
			Some(' ') => (DiffLineType::None, &line[1..]),
			Some('\\') => continue, // "\ No newline at end of file"
			_ => (DiffLineType::None, line),
		};
		let position = match kind {
			DiffLineType::Add => {
				let p = DiffLinePosition {
					old_lineno: None,
					new_lineno: Some(new_line),
				};
				new_line += 1;
				p
			}
			DiffLineType::Delete => {
				let p = DiffLinePosition {
					old_lineno: Some(old_line),
					new_lineno: None,
				};
				old_line += 1;
				p
			}
			_ => {
				let p = DiffLinePosition {
					old_lineno: Some(old_line),
					new_lineno: Some(new_line),
				};
				old_line += 1;
				new_line += 1;
				p
			}
		};
		current_lines.push(DiffLine {
			content: content.into(),
			line_type: kind,
			position,
		});
	}
	flush(&mut diff, &mut current_header, &mut current_lines);
	diff.lines = diff.hunks.iter().map(|h| h.lines.len()).sum();
	diff
}

fn parse_hunk_header(line: &str) -> Option<HunkHeader> {
	// `@@ -<old_start>[,<old_lines>] +<new_start>[,<new_lines>] @@`
	let line = line.strip_prefix("@@")?;
	let end = line.find("@@")?;
	let inner = line[..end].trim();
	let mut parts = inner.split_whitespace();
	let old = parts.next()?.strip_prefix('-')?;
	let new = parts.next()?.strip_prefix('+')?;
	let (old_start, old_lines) = parse_range(old);
	let (new_start, new_lines) = parse_range(new);
	Some(HunkHeader {
		old_start,
		old_lines,
		new_start,
		new_lines,
	})
}

fn parse_range(s: &str) -> (u32, u32) {
	if let Some((a, b)) = s.split_once(',') {
		(a.parse().unwrap_or(0), b.parse().unwrap_or(0))
	} else {
		(s.parse().unwrap_or(0), 1)
	}
}

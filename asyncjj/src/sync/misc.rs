//! Commit filtering (revset search), tree listing, submodules, ignore.
//!
//! The filter API mirrors the old git backend shapes
//! (`LogFilterSearch`, `SearchFields`, …) but evaluates against `jj log`
//! metadata instead of libgit2 objects.

use crate::{
	error::Result,
	sync::{commits_info::CommitId, jj_cmd::run_jj, RepoPath},
};
use bitflags::bitflags;
use fuzzy_matcher::FuzzyMatcher;
use std::sync::Arc;

///
pub type SharedCommitFilterFn =
	Arc<dyn Fn(&RepoPath, &CommitId) -> Result<bool> + Send + Sync>;

bitflags! {
	///
	#[derive(Debug, Clone, Copy)]
	pub struct SearchFields: u32 {
		///
		const MESSAGE_SUMMARY = 1 << 0;
		///
		const MESSAGE_BODY = 1 << 1;
		///
		const FILENAMES = 1 << 2;
		///
		const AUTHORS = 1 << 3;
	}
}

impl Default for SearchFields {
	fn default() -> Self {
		Self::MESSAGE_SUMMARY
	}
}

bitflags! {
	///
	#[derive(Debug, Clone, Copy)]
	pub struct SearchOptions: u32 {
		///
		const CASE_SENSITIVE = 1 << 0;
		///
		const FUZZY_SEARCH = 1 << 1;
	}
}

impl Default for SearchOptions {
	fn default() -> Self {
		Self::empty()
	}
}

///
#[derive(Default, Debug, Clone)]
pub struct LogFilterSearchOptions {
	///
	pub search_pattern: String,
	///
	pub fields: SearchFields,
	///
	pub options: SearchOptions,
}

///
#[derive(Default)]
pub struct LogFilterSearch {
	///
	pub matcher: fuzzy_matcher::skim::SkimMatcherV2,
	///
	pub options: LogFilterSearchOptions,
}

impl LogFilterSearch {
	///
	pub fn new(options: LogFilterSearchOptions) -> Self {
		let mut options = options;
		if !options.options.contains(SearchOptions::CASE_SENSITIVE) {
			options.search_pattern =
				options.search_pattern.to_lowercase();
		}
		Self {
			matcher: fuzzy_matcher::skim::SkimMatcherV2::default(),
			options,
		}
	}

	///
	pub fn match_text(&self, text: &str) -> bool {
		if self.options.options.contains(SearchOptions::FUZZY_SEARCH)
		{
			self.matcher
				.fuzzy_match(
					text,
					self.options.search_pattern.as_str(),
				)
				.is_some()
		} else if self
			.options
			.options
			.contains(SearchOptions::CASE_SENSITIVE)
		{
			text.contains(self.options.search_pattern.as_str())
		} else {
			text.to_lowercase()
				.contains(self.options.search_pattern.as_str())
		}
	}
}

///
pub fn diff_contains_file(file_path: String) -> SharedCommitFilterFn {
	Arc::new(move |repo_path: &RepoPath, id: &CommitId| {
		let out = run_jj(
			repo_path.jj_root(),
			&["diff", "--summary", "-r", id.as_str()],
		)?;
		Ok(out.lines().any(|l| l.contains(file_path.as_str())))
	})
}

///
pub fn filter_commit_by_search(
	filter: LogFilterSearch,
) -> SharedCommitFilterFn {
	Arc::new(move |repo_path: &RepoPath, id: &CommitId| {
		let wants_summary = filter
			.options
			.fields
			.contains(SearchFields::MESSAGE_SUMMARY);
		let wants_body = filter
			.options
			.fields
			.contains(SearchFields::MESSAGE_BODY);
		let wants_files =
			filter.options.fields.contains(SearchFields::FILENAMES);
		let wants_authors =
			filter.options.fields.contains(SearchFields::AUTHORS);

		if wants_summary || wants_body {
			let details = super::commit_details::get_commit_details(
				repo_path,
				id.clone(),
			)?;
			if wants_summary {
				if let Some(message) = &details.message {
					if filter.match_text(&message.subject) {
						return Ok(true);
					}
				}
			}
			if wants_body {
				if let Some(message) = &details.message {
					if let Some(body) = &message.body {
						if filter.match_text(body) {
							return Ok(true);
						}
					}
				}
			}
		}
		if wants_authors {
			let details = super::commit_details::get_commit_details(
				repo_path,
				id.clone(),
			)?;
			if filter.match_text(&details.author.name) {
				return Ok(true);
			}
		}
		if wants_files {
			let out = run_jj(
				repo_path.jj_root(),
				&["diff", "--summary", "-r", id.as_str()],
			)?;
			if out.lines().any(|l| filter.match_text(l)) {
				return Ok(true);
			}
		}
		Ok(false)
	})
}

///
pub fn add_to_ignore(
	_repo_path: &RepoPath,
	_path: &str,
) -> Result<()> {
	Err(crate::error::Error::Unsupported(
		"add_to_ignore NYI: edit .gitignore manually".to_string(),
	))
}

// --- submodules (unsupported by jj) ---

///
#[derive(Debug, Clone)]
pub struct SubmoduleInfo {
	///
	pub name: String,
	///
	pub path: std::path::PathBuf,
	///
	pub url: Option<String>,
	///
	pub id: Option<super::commits_info::CommitId>,
	///
	pub head_id: Option<super::commits_info::CommitId>,
	///
	pub status: SubmoduleStatus,
}

///
#[derive(Debug, Clone)]
pub struct SubmoduleParentInfo {
	/// where to find parent repo
	pub parent_gitpath: std::path::PathBuf,
}

///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubmoduleStatus(u8);

impl SubmoduleStatus {
	///
	pub const fn empty() -> Self {
		Self(0)
	}

	///
	pub const fn is_in_wd(self) -> bool {
		false
	}
}

///
pub fn submodule_parent_info(
	_repo_path: &RepoPath,
) -> Result<Option<SubmoduleParentInfo>> {
	Ok(None)
}

///
pub fn get_submodules(
	_repo_path: &RepoPath,
) -> Result<Vec<SubmoduleInfo>> {
	Ok(Vec::new())
}

///
pub fn update_submodule(
	_repo_path: &RepoPath,
	_path: &str,
) -> Result<()> {
	Err(crate::error::Error::Unsupported(
		"submodules are not supported by jj".to_string(),
	))
}

// --- tree ---

///
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TreeFile {
	/// path of this file
	pub path: std::path::PathBuf,
	/// unix filemode (stubbed)
	pub filemode: i32,
}

///
pub fn tree_files(
	repo_path: &RepoPath,
	commit: CommitId,
) -> Result<Vec<TreeFile>> {
	let out = run_jj(
		repo_path.jj_root(),
		&["file", "list", "-r", commit.as_str()],
	)?;
	let mut files: Vec<TreeFile> = out
		.lines()
		.map(str::trim)
		.filter(|l| !l.is_empty())
		.map(|p| TreeFile {
			path: std::path::PathBuf::from(p),
			filemode: 0,
		})
		.collect();
	files.sort_by(|a, b| a.path.cmp(&b.path));
	Ok(files)
}

///
pub fn tree_file_content(
	repo_path: &RepoPath,
	file: &TreeFile,
) -> Result<String> {
	let path = file.path.to_string_lossy().into_owned();
	run_jj(
		repo_path.jj_root(),
		&["file", "show", "-r", "@", path.as_str()],
	)
}

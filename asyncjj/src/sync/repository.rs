use std::{
	cell::RefCell,
	path::{Path, PathBuf},
};

use crate::{
	error::{Error, Result},
	sync::jj_cmd::run_jj,
};

///
pub type RepoPathRef = RefCell<RepoPath>;

///
/// Workspace root for jj invocations.
#[derive(Clone, Debug)]
pub enum RepoPath {
	///
	Path(PathBuf),
	///
	Workdir {
		///
		gitdir: PathBuf,
		///
		workdir: PathBuf,
	},
}

impl RepoPath {
	///
	pub fn gitpath(&self) -> &Path {
		match self {
			Self::Path(p) => p.as_path(),
			Self::Workdir { gitdir, .. } => gitdir.as_path(),
		}
	}

	///
	pub fn workdir(&self) -> Option<&Path> {
		match self {
			Self::Path(_) => None,
			Self::Workdir { workdir, .. } => Some(workdir.as_path()),
		}
	}

	/// Workspace root for jj invocations.
	pub fn jj_root(&self) -> &Path {
		match self {
			Self::Path(p) => p.as_path(),
			Self::Workdir { workdir, .. } => workdir.as_path(),
		}
	}
}

impl From<PathBuf> for RepoPath {
	fn from(value: PathBuf) -> Self {
		Self::Path(value)
	}
}

impl From<&str> for RepoPath {
	fn from(p: &str) -> Self {
		Self::Path(PathBuf::from(p))
	}
}

impl From<&Path> for RepoPath {
	fn from(p: &Path) -> Self {
		Self::Path(p.to_path_buf())
	}
}

/// Discover the workspace root (`jj root`) for `start` or any subdir.
pub fn discover_root(start: &Path) -> Result<PathBuf> {
	// `jj root` prints the workspace root and fails outside a repo.
	let out = run_jj(start, &["root"])?;
	let root = out.trim();
	if root.is_empty() {
		return Err(Error::Generic(
			"`jj root` returned empty output".to_string(),
		));
	}
	Ok(PathBuf::from(root))
}

/// Returns `None` when `path` is inside a jj repo, else `Some(error string)`.
pub fn repo_open_error(repo_path: &RepoPath) -> Option<String> {
	match discover_root(repo_path.jj_root()) {
		Ok(_) => None,
		Err(e) => Some(e.to_string()),
	}
}

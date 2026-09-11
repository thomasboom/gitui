//! Config helpers (jj TOML config instead of gitconfig).

use super::status::ShowUntrackedFilesConfig;
use crate::{
	error::Result,
	sync::{jj_cmd::run_jj, RepoPath},
};

///
pub fn get_config_string(
	repo_path: &RepoPath,
	name: &str,
) -> Result<Option<String>> {
	let out = run_jj(repo_path.jj_root(), &["config", "get", name]);
	out.map_or(Ok(None), |s| {
		let s = s.trim();
		Ok((!s.is_empty()).then(|| s.to_string()))
	})
}

///
pub fn untracked_files_config(
	_repo_path: &RepoPath,
) -> Result<ShowUntrackedFilesConfig> {
	Ok(ShowUntrackedFilesConfig::All)
}

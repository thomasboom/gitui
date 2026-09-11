//! Credentials: jj delegates to ssh/agent helpers; nothing to prompt for.
//!
//! The `need_*` / `extract_*` shapes mirror the old git backend so the
//! credential popups keep compiling; they report "no input needed".

use super::RepoPath;
use crate::error::Result;

/// basic Authentication Credentials
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BasicAuthCredential {
	///
	pub username: Option<String>,
	///
	pub password: Option<String>,
}

impl BasicAuthCredential {
	///
	pub const fn is_complete(&self) -> bool {
		self.username.is_some() && self.password.is_some()
	}
	///
	pub const fn new(
		username: Option<String>,
		password: Option<String>,
	) -> Self {
		Self { username, password }
	}
}

///
pub fn need_username_password(_repo_path: &RepoPath) -> Result<bool> {
	Ok(false)
}

///
pub fn need_username_password_for_fetch(
	_repo_path: &RepoPath,
) -> Result<bool> {
	Ok(false)
}

///
pub fn need_username_password_for_push(
	_repo_path: &RepoPath,
) -> Result<bool> {
	Ok(false)
}

///
pub fn extract_username_password(
	_repo_path: &RepoPath,
) -> Result<BasicAuthCredential> {
	Ok(BasicAuthCredential::new(None, None))
}

///
pub fn extract_username_password_for_fetch(
	_repo_path: &RepoPath,
) -> Result<BasicAuthCredential> {
	Ok(BasicAuthCredential::new(None, None))
}

///
pub fn extract_username_password_for_push(
	_repo_path: &RepoPath,
) -> Result<BasicAuthCredential> {
	Ok(BasicAuthCredential::new(None, None))
}

///
pub fn extract_cred_from_url(url: &str) -> BasicAuthCredential {
	let _ = url;
	BasicAuthCredential::new(None, None)
}

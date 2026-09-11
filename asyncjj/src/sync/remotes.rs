//! Remotes via `jj git fetch` / `jj git push`.
//!
//! jj repos using the git backend keep git remotes under the hood.

use crate::{
	error::{Error, Result},
	sync::{cred::BasicAuthCredential, jj_cmd::run_jj, RepoPath},
};
use crossbeam_channel::Sender;
use scopetime::scope_time;

pub use tags::tags_missing_remote;

///
pub fn get_remotes(repo_path: &RepoPath) -> Result<Vec<String>> {
	let out =
		run_jj(repo_path.jj_root(), &["git", "remote", "list"])?;
	Ok(out
		.lines()
		.map(str::trim)
		.filter(|l| !l.is_empty())
		.map(|l| l.split_whitespace().next().unwrap_or(l).to_string())
		.collect())
}

///
pub fn get_remote_url(
	repo_path: &RepoPath,
	remote: &str,
) -> Result<Option<String>> {
	let out = run_jj(
		repo_path.jj_root(),
		&["config", "get", &format!("remotes.{remote}.url")],
	);
	out.map_or(Ok(None), |s| {
		let s = s.trim();
		Ok((!s.is_empty()).then(|| s.to_string()))
	})
}

///
pub fn get_default_remote(repo_path: &RepoPath) -> Result<String> {
	let remotes = get_remotes(repo_path)?;
	remotes
		.iter()
		.find(|r| *r == "origin")
		.or_else(|| remotes.first())
		.cloned()
		.ok_or(Error::UnknownRemote)
}

///
pub fn get_default_remote_for_fetch(
	repo_path: &RepoPath,
) -> Result<String> {
	get_default_remote(repo_path)
}

///
pub fn get_default_remote_for_push(
	repo_path: &RepoPath,
) -> Result<String> {
	get_default_remote(repo_path)
}

///
pub fn add_remote(
	repo_path: &RepoPath,
	name: &str,
	url: &str,
) -> Result<()> {
	debug_assert!(validate_remote_name(name));
	run_jj(
		repo_path.jj_root(),
		&["git", "remote", "add", name, url],
	)?;
	Ok(())
}

///
pub fn delete_remote(repo_path: &RepoPath, name: &str) -> Result<()> {
	run_jj(repo_path.jj_root(), &["git", "remote", "remove", name])?;
	Ok(())
}

///
pub fn rename_remote(
	repo_path: &RepoPath,
	old: &str,
	new: &str,
) -> Result<()> {
	debug_assert!(validate_remote_name(new));
	run_jj(
		repo_path.jj_root(),
		&["git", "remote", "rename", old, new],
	)?;
	Ok(())
}

///
pub fn update_remote_url(
	repo_path: &RepoPath,
	remote: &str,
	url: &str,
) -> Result<()> {
	run_jj(
		repo_path.jj_root(),
		&["git", "remote", "set-url", remote, url],
	)?;
	Ok(())
}

/// sync port of `git2::validate_remote_name`-ish check
pub fn validate_remote_name(name: &str) -> bool {
	!name.trim().is_empty() && !name.contains(char::is_whitespace)
}

///
pub fn fetch(
	repo_path: &RepoPath,
	branch: &str,
	_cred: Option<BasicAuthCredential>,
	progress_sender: Option<Sender<push::ProgressNotification>>,
) -> Result<usize> {
	use push::ProgressNotification;
	let remote = get_default_remote(repo_path)
		.unwrap_or_else(|_| "origin".to_string());
	let _ = branch;
	git_fetch_remote(repo_path, Some(remote.as_str()))?;
	if let Some(tx) = progress_sender {
		let _ = tx.send(ProgressNotification::Done);
	}
	Ok(0)
}

///
pub fn fetch_all(
	repo_path: &RepoPath,
	_cred: &Option<BasicAuthCredential>,
	_progress: &Option<Sender<push::ProgressNotification>>,
) -> Result<()> {
	git_fetch_remote(repo_path, None)
}

fn git_fetch_remote(
	repo_path: &RepoPath,
	remote: Option<&str>,
) -> Result<()> {
	scope_time!("git_fetch");
	match remote {
		Some(r) => {
			run_jj(
				repo_path.jj_root(),
				&["git", "fetch", "--remote", r],
			)?;
		}
		None => {
			run_jj(repo_path.jj_root(), &["git", "fetch"])?;
		}
	}
	Ok(())
}

///
pub fn git_fetch(
	repo_path: &RepoPath,
	remote: Option<&str>,
) -> Result<()> {
	git_fetch_remote(repo_path, remote)
}

///
pub fn git_push(
	repo_path: &RepoPath,
	_bookmark: Option<&str>,
	_remote: Option<&str>,
	_allow_new: bool,
) -> Result<()> {
	scope_time!("git_push");
	run_jj(repo_path.jj_root(), &["git", "push"])?;
	Ok(())
}

///
pub mod push {
	use super::{
		run_jj, BasicAuthCredential, RepoPath, Result, Sender,
	};
	use crate::progress::ProgressPercent;

	///
	pub trait AsyncProgress: Clone + Send {
		///
		fn is_done(&self) -> bool;
		///
		fn progress(&self) -> ProgressPercent;
	}

	///
	#[derive(Debug, Clone)]
	pub enum ProgressNotification {
		///
		Update,
		///
		Done,
	}

	impl AsyncProgress for ProgressNotification {
		fn is_done(&self) -> bool {
			matches!(self, Self::Done)
		}

		fn progress(&self) -> ProgressPercent {
			match self {
				Self::Done => ProgressPercent::full(),
				Self::Update => ProgressPercent::empty(),
			}
		}
	}

	///
	#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
	pub enum PushType {
		///
		#[default]
		Branch,
		///
		Tag,
	}

	///
	#[allow(clippy::too_many_arguments)]
	pub fn push_raw(
		repo_path: &RepoPath,
		remote: &str,
		branch: &str,
		push_type: PushType,
		_force: bool,
		delete: bool,
		_basic_credential: Option<BasicAuthCredential>,
		progress_sender: Option<Sender<ProgressNotification>>,
	) -> Result<()> {
		let _ = (remote, push_type, delete);
		if branch.is_empty() {
			run_jj(repo_path.jj_root(), &["git", "push"])?;
		} else {
			// Push the bookmark matching the branch name.
			let res = run_jj(
				repo_path.jj_root(),
				&["git", "push", "--bookmark", branch],
			);
			if let Some(tx) = progress_sender {
				let _ = tx.send(ProgressNotification::Update);
			}
			res?;
		}
		Ok(())
	}

	///
	pub fn push_branch(
		repo_path: &RepoPath,
		remote: &str,
		branch: &str,
		force: bool,
		delete: bool,
		basic_credential: Option<BasicAuthCredential>,
		progress_sender: Option<Sender<ProgressNotification>>,
	) -> Result<()> {
		push_raw(
			repo_path,
			remote,
			branch,
			PushType::Branch,
			force,
			delete,
			basic_credential,
			progress_sender,
		)
	}
}

///
pub mod tags {
	use super::push::AsyncProgress;
	use super::{
		run_jj, BasicAuthCredential, RepoPath, Result, Sender,
	};
	use crate::progress::ProgressPercent;

	///
	#[derive(Debug, Clone, Copy, PartialEq, Eq)]
	pub enum PushTagsProgress {
		/// checking remote for missing tags
		CheckRemote,
		/// pushing local tags that are missing remote
		Push {
			///
			pushed: usize,
			///
			total: usize,
		},
		/// done
		Done,
	}

	impl AsyncProgress for PushTagsProgress {
		fn is_done(&self) -> bool {
			matches!(self, Self::Done)
		}

		fn progress(&self) -> ProgressPercent {
			match self {
				Self::CheckRemote => ProgressPercent::empty(),
				Self::Push { pushed, total } => {
					ProgressPercent::new(*pushed, *total)
				}
				Self::Done => ProgressPercent::full(),
			}
		}
	}

	///
	pub fn tags_missing_remote(
		repo_path: &RepoPath,
		remote: &str,
		_cred: Option<BasicAuthCredential>,
	) -> Result<Vec<String>> {
		// Best effort: list local tags; jj push --all covers them.
		let out = run_jj(
			repo_path.jj_root(),
			&["tag", "list", "-T", "name ++ \"\\n\""],
		)?;
		let _ = remote;
		Ok(out
			.lines()
			.map(str::trim)
			.filter(|l| !l.is_empty())
			.map(str::to_string)
			.collect())
	}

	///
	pub fn push_tags(
		repo_path: &RepoPath,
		remote: &str,
		_cred: Option<BasicAuthCredential>,
		progress_sender: Option<Sender<PushTagsProgress>>,
	) -> Result<()> {
		let _ = remote;
		let res =
			run_jj(repo_path.jj_root(), &["git", "push", "--all"])?;
		let _ = res;
		if let Some(tx) = progress_sender {
			let _ = tx.send(PushTagsProgress::Done);
		}
		Ok(())
	}
}

//! Integration tests for the `jj` CLI backend.
//!
//! These drive a real `jj` binary in temp dirs, mirroring how `lazyjj`
//! interoperates with Jujutsu (CLI as the stable automation surface).

use asyncjj::sync::{
	abandon, commit, describe, get_branches_info, get_commit_files,
	get_commit_info, get_commits_info, get_diff_commit, get_head,
	get_operations, get_status, new_change, redo, undo, CommitId,
	RepoPath, StatusType,
};
use std::{path::PathBuf, process::Command};

fn jj(args: &[&str], dir: &std::path::Path) {
	let status = Command::new("jj")
		.args(args)
		.current_dir(dir)
		.env("JJ_NO_PAGER", "1")
		.status()
		.expect("jj binary must be installed for asyncjj tests");
	assert!(status.success(), "jj {args:?} failed");
}

fn init_repo() -> (tempfile::TempDir, RepoPath) {
	let td = tempfile::TempDir::new().unwrap();
	let dir: PathBuf = td.path().to_path_buf();
	jj(&["git", "init"], &dir);
	jj(&["config", "set", "--repo", "user.name", "tester"], &dir);
	jj(
		&[
			"config",
			"set",
			"--repo",
			"user.email",
			"tester@example.com",
		],
		&dir,
	);
	std::fs::write(dir.join("file.txt"), "hello\n").unwrap();
	jj(&["describe", "-m", "first"], &dir);
	let repo_path: RepoPath = dir.to_str().unwrap().into();
	(td, repo_path)
}

#[test]
fn status_reports_working_copy_changes() {
	let (_td, repo) = init_repo();
	let root = repo.jj_root();

	// The described change shows its content as working-copy changes…
	let items =
		get_status(&repo, StatusType::WorkingDir, None).unwrap();
	assert_eq!(items.len(), 1);
	assert_eq!(items[0].path, "file.txt");

	// …until a new empty change is created on top (`jj new`).
	new_change(&repo, None).unwrap();
	let clean =
		get_status(&repo, StatusType::WorkingDir, None).unwrap();
	assert!(clean.is_empty());

	// New edits on disk show up again.
	std::fs::write(root.join("file.txt"), "hello\nworld\n").unwrap();
	let items =
		get_status(&repo, StatusType::WorkingDir, None).unwrap();
	assert_eq!(items.len(), 1);
	assert_eq!(items[0].path, "file.txt");

	// jj has no staging area: Stage is always empty.
	let staged = get_status(&repo, StatusType::Stage, None).unwrap();
	assert!(staged.is_empty());
}

#[test]
fn log_and_commit_info_roundtrip() {
	let (_td, repo) = init_repo();

	let head = get_head(&repo).unwrap();
	let info = get_commit_info(&repo, &head).unwrap();
	assert_eq!(info.message.lines().next().unwrap(), "first");
	assert!(!info.author.is_empty());

	let infos =
		get_commits_info(&repo, std::slice::from_ref(&head), 50)
			.unwrap();
	assert_eq!(infos.len(), 1);
	assert_eq!(infos[0].id, head);

	let resolved = CommitId::from_revision(&repo, "@").unwrap();
	assert_eq!(resolved, head);
}

#[test]
fn diff_and_commit_files() {
	let (_td, repo) = init_repo();
	let head = get_head(&repo).unwrap();

	let diff =
		get_diff_commit(&repo, head.clone(), String::new(), None)
			.unwrap();
	assert!(!diff.hunks.is_empty());

	let files = get_commit_files(&repo, head, None).unwrap();
	assert_eq!(files.len(), 1);
	assert_eq!(files[0].path, "file.txt");
}

#[test]
fn bookmarks_crud() {
	let (_td, repo) = init_repo();
	let head = get_head(&repo).unwrap();

	assert!(get_branches_info(&repo, false).unwrap().is_empty());

	asyncjj::sync::create_bookmark(&repo, "feature", &head).unwrap();
	let all = get_branches_info(&repo, false).unwrap();
	assert_eq!(all.len(), 1);
	assert_eq!(all[0].name, "feature");
	assert!(all[0].is_local());

	asyncjj::sync::rename_branch(&repo, "feature", "feat2").unwrap();
	let all = get_branches_info(&repo, false).unwrap();
	assert_eq!(all[0].name, "feat2");

	asyncjj::sync::delete_bookmark(&repo, "feat2").unwrap();
	assert!(get_branches_info(&repo, false).unwrap().is_empty());
}

#[test]
fn describe_new_abandon_and_undo() {
	let (_td, repo) = init_repo();

	describe(&repo, "@", "renamed").unwrap();
	let head = get_head(&repo).unwrap();
	assert_eq!(
		get_commit_info(&repo, &head)
			.unwrap()
			.message
			.lines()
			.next()
			.unwrap(),
		"renamed"
	);

	new_change(&repo, Some("second")).unwrap();
	commit(&repo, "via-commit-fn").unwrap();

	let ops = get_operations(&repo, 5).unwrap();
	assert!(!ops.is_empty());

	abandon(&repo, "@").unwrap();

	// undo restores the abandoned change
	undo(&repo).unwrap();
	redo(&repo).unwrap();
}

//! sync jj api

pub mod blame;
pub mod branch;
pub mod commit;
mod commit_details;
pub mod commit_files;
///
pub mod commits_info;
pub mod config;
pub mod cred;
pub mod diff;
pub mod hooks;
pub mod jj_cmd;
pub mod logwalker;
pub mod merge;
pub mod misc;
pub mod operations;
pub mod remotes;
mod repository;
pub mod reset;
pub mod stash;
pub mod state;
pub mod status;
pub mod tags;
pub mod utils;

pub use blame::{blame_file, BlameHunk, FileBlame};
pub use branch::{
	branch_compare_upstream, checkout_branch, checkout_commit,
	checkout_remote_branch, create_bookmark, create_branch,
	delete_bookmark, delete_branch, get_bookmarks, get_branch_name,
	get_branch_remote, get_branch_upstream_merge, get_branches_info,
	rename_branch, validate_branch_name, BookmarkInfo, BranchCompare,
	BranchDetails, BranchInfo, BranchType, LocalBranch, RemoteBranch,
	UpstreamBranch,
};
pub use commit::{
	abandon, amend, commit, commit_message_prettify, describe,
	new_change, reword, squash, tag_commit,
};
pub use commit_details::{
	get_author_of_commit_str, get_commit_details, CommitDetails,
	CommitMessage, CommitSignature,
};
pub use commit_files::{get_commit_files, OldNew};
pub use commits_info::{
	get_commit_info, get_commits_info, ChangeId, CommitId, CommitInfo,
};
pub use config::{get_config_string, untracked_files_config};
pub use cred::{
	extract_cred_from_url, extract_username_password,
	extract_username_password_for_fetch,
	extract_username_password_for_push, need_username_password,
	need_username_password_for_fetch,
	need_username_password_for_push, BasicAuthCredential,
};
pub use diff::{
	get_diff, get_diff_commit, get_diff_commits, parse_git_diff,
	DiffLine, DiffLinePosition, DiffLineType, DiffOptions, FileDiff,
	Hunk,
};
pub use hooks::{
	hooks_commit_msg, hooks_post_commit, hooks_pre_commit,
	hooks_pre_push, hooks_prepare_commit_msg, merge_msg, HookResult,
	PrePushTarget, PrepareCommitMsgSource,
};
pub use logwalker::{LogWalker, LogWalkerWithoutFilter};
pub use merge::{
	abort_pending_rebase, abort_pending_state,
	branch_merge_upstream_fastforward, config_is_pull_rebase,
	continue_pending_rebase, merge_branch, merge_commit,
	merge_upstream_commit, merge_upstream_rebase, mergehead_ids,
	rebase_branch, rebase_progress, RebaseProgress,
};
pub use misc::{
	add_to_ignore, diff_contains_file, filter_commit_by_search,
	get_submodules, submodule_parent_info, tree_file_content,
	tree_files, update_submodule, LogFilterSearch,
	LogFilterSearchOptions, SearchFields, SearchOptions,
	SharedCommitFilterFn, SubmoduleInfo, SubmoduleParentInfo,
	SubmoduleStatus, TreeFile,
};
pub use operations::{
	get_head, get_head_tuple, get_operations, redo, repo_dir,
	restore_operation, undo, undo_last_commit, OperationInfo,
};
pub use remotes::push::{AsyncProgress, PushType};
pub use remotes::tags::{tags_missing_remote, PushTagsProgress};
pub use remotes::{
	add_remote, delete_remote, fetch, fetch_all, get_default_remote,
	get_default_remote_for_fetch, get_default_remote_for_push,
	get_remote_url, get_remotes, git_fetch, git_push, rename_remote,
	update_remote_url, validate_remote_name,
};
pub use repository::{
	discover_root, repo_open_error, RepoPath, RepoPathRef,
};
pub use reset::{
	commit_revert, discard_lines, reset_hunk, reset_repo,
	reset_stage, reset_workdir, revert_commit, revert_head,
	stage_hunk, stage_lines, unstage_hunk, ResetType,
};
pub use stash::{
	get_stashes, stash_apply, stash_drop, stash_pop, stash_save,
};
pub use state::{repo_state, RepoState};
pub use status::{
	discard_status, get_status, is_workdir_clean,
	ShowUntrackedFilesConfig, StatusItem, StatusItemType, StatusType,
};
pub use tags::{
	delete_tag, get_tags, get_tags_with_metadata, CommitTags, Tag,
	TagWithMetadata, Tags,
};
pub use utils::{
	repo_work_dir, stage_add_all, stage_add_file, stage_addremoved,
	Head,
};
/// signing is not supported by the jj backend
pub mod sign {
	/// signing is not supported by the jj backend
	#[derive(Debug, thiserror::Error)]
	///
	#[error("{0}")]
	pub struct SignBuilderError(pub String);
	/// signing is not supported by the jj backend
	#[derive(Debug, thiserror::Error)]
	///
	#[error("{0}")]
	pub struct SignError(pub String);
}

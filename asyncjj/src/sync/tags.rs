//! Tags via `jj tag list`.

use super::commits_info::CommitId;
use crate::{
	error::Result,
	sync::{jj_cmd::run_jj, RepoPath},
};
use scopetime::scope_time;
use std::collections::BTreeMap;

///
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Tag {
	/// tag name
	pub name: String,
	/// tag annotation
	pub annotation: Option<String>,
}

impl Tag {
	///
	pub fn new(name: &str) -> Self {
		Self {
			name: name.into(),
			annotation: None,
		}
	}
}

/// all tags pointing to a single commit
pub type CommitTags = Vec<Tag>;
/// hashmap of tag target commit hash to tag names
pub type Tags = BTreeMap<CommitId, CommitTags>;

///
pub struct TagWithMetadata {
	///
	pub name: String,
	///
	pub author: String,
	///
	pub time: i64,
	///
	pub message: String,
	///
	pub commit_id: CommitId,
	///
	pub annotation: Option<String>,
}

///
pub fn get_tags(repo_path: &RepoPath) -> Result<Tags> {
	scope_time!("get_tags");

	let out = run_jj(
		repo_path.jj_root(),
		&[
			"tag",
			"list",
			"-T",
			"name ++ \"\\x1f\" ++ self.normal_target().commit_id() ++ \"\\x1e\"",
		],
	)?;
	let mut res = Tags::new();
	for record in out.split('\x1e') {
		let record = record.trim();
		if record.is_empty() {
			continue;
		}
		let mut parts = record.splitn(2, '\x1f');
		let (Some(name), Some(id)) = (parts.next(), parts.next())
		else {
			continue;
		};
		let id = CommitId::from_str_unchecked(id.trim())?;
		let tag = Tag::new(name.trim());
		if let Some(key) = res.get_mut(&id) {
			key.push(tag);
		} else {
			res.insert(id, vec![tag]);
		}
	}
	Ok(res)
}

///
pub fn get_tags_with_metadata(
	repo_path: &RepoPath,
) -> Result<Vec<TagWithMetadata>> {
	Ok(get_tags(repo_path)?
		.into_iter()
		.flat_map(|(id, tags)| {
			tags.into_iter().map(move |t| TagWithMetadata {
				name: t.name,
				author: String::new(),
				time: 0,
				message: String::new(),
				commit_id: id.clone(),
				annotation: t.annotation,
			})
		})
		.collect())
}

///
pub fn delete_tag(
	repo_path: &RepoPath,
	tag_name: &str,
) -> Result<()> {
	scope_time!("delete_tag");
	run_jj(repo_path.jj_root(), &["tag", "delete", tag_name])?;
	Ok(())
}

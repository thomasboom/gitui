//! Commit details via `jj show`.

use super::commits_info::CommitId;
use crate::{
	error::Result,
	sync::{jj_cmd::run_jj, RepoPath},
};
use scopetime::scope_time;

///
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct CommitSignature {
	///
	pub name: String,
	///
	pub email: String,
	/// time in secs since Unix epoch
	pub time: i64,
}

///
#[derive(Default, Clone)]
pub struct CommitMessage {
	/// first line
	pub subject: String,
	/// remaining lines if more than one
	pub body: Option<String>,
}

impl CommitMessage {
	///
	pub fn from(s: &str) -> Self {
		let mut lines = s.lines();
		let subject =
			lines.next().map_or_else(String::new, str::to_string);

		let body: Vec<String> = lines.map(str::to_string).collect();

		Self {
			subject,
			body: if body.is_empty() {
				None
			} else {
				Some(body.join("\n"))
			},
		}
	}

	///
	pub fn combine(self) -> String {
		if let Some(body) = self.body {
			format!("{}\n{body}", self.subject)
		} else {
			self.subject
		}
	}
}

///
#[derive(Default, Clone)]
pub struct CommitDetails {
	///
	pub author: CommitSignature,
	/// committer when differs to `author` otherwise None
	pub committer: Option<CommitSignature>,
	///
	pub message: Option<CommitMessage>,
	///
	pub hash: String,
}

impl CommitDetails {
	///
	pub fn short_hash(&self) -> &str {
		self.hash.get(0..7).unwrap_or(self.hash.as_str())
	}
}

const DETAILS_TEMPLATE: &str = "author.name() ++ \"\\x1f\" ++ author.email() ++ \"\\x1f\" ++ committer.timestamp().format(\"%s\") ++ \"\\x1f\" ++ committer.name() ++ \"\\x1f\" ++ committer.email() ++ \"\\x1f\" ++ description ++ \"\\x1e\"";

///
pub fn get_commit_details(
	repo_path: &RepoPath,
	id: CommitId,
) -> Result<CommitDetails> {
	scope_time!("get_commit_details");
	let out = run_jj(
		repo_path.jj_root(),
		&[
			"log",
			"--no-graph",
			"-r",
			id.as_str(),
			"-T",
			DETAILS_TEMPLATE,
		],
	)?;
	let record = out.split('\x1e').next().unwrap_or("").trim();
	let mut parts = record.splitn(6, '\x1f');
	let (
		Some(an),
		Some(ae),
		Some(at),
		Some(cn),
		Some(ce),
		Some(desc),
	) = (
		parts.next(),
		parts.next(),
		parts.next(),
		parts.next(),
		parts.next(),
		parts.next(),
	)
	else {
		return Err(crate::error::Error::Parse(
			"unexpected jj details output".to_string(),
		));
	};
	let message = CommitMessage::from(desc);
	let time = at.trim().parse().unwrap_or(0);
	let author = CommitSignature {
		name: an.trim().to_string(),
		email: ae.trim().to_string(),
		time,
	};
	let committer = CommitSignature {
		name: cn.trim().to_string(),
		email: ce.trim().to_string(),
		time,
	};
	let committer = if committer == author {
		None
	} else {
		Some(committer)
	};
	Ok(CommitDetails {
		author,
		committer,
		message: if message.subject.is_empty()
			&& message.body.is_none()
		{
			None
		} else {
			Some(message)
		},
		hash: id.to_string(),
	})
}

///
pub fn get_author_of_commit_str(
	repo_path: &RepoPath,
	id: &CommitId,
) -> Result<String> {
	Ok(get_commit_details(repo_path, id.clone())?.author.name)
}

use std::{
	num::TryFromIntError, path::StripPrefixError,
	string::FromUtf8Error,
};
use thiserror::Error;

///
#[derive(Error, Debug)]
pub enum Error {
	///
	#[error("`{0}`")]
	Generic(String),

	///
	#[error("jj: no working-copy commit found")]
	NoHead,

	///
	#[error("jj: no parent of commit found")]
	NoParent,

	///
	#[error("jj: not on a bookmark")]
	NoBookmark,

	///
	#[error("jj: work dir error")]
	NoWorkDir,

	///
	#[error("jj: uncommitted changes")]
	UncommittedChanges,

	///
	#[error("jj: cannot run blame on a binary file")]
	NoBlameOnBinaryFile,

	///
	#[error("binary file")]
	BinaryFile,

	///
	#[error("io error:{0}")]
	Io(#[from] std::io::Error),

	///
	#[error(
		"jj command `{command}` failed (status {status}): {stderr}"
	)]
	Jj {
		///
		command: String,
		///
		status: String,
		///
		stderr: String,
	},

	///
	#[error("jj binary not found: {0}")]
	JjNotFound(String),

	///
	#[error("failed to parse jj output: {0}")]
	Parse(String),

	///
	#[error("operation not supported in the jj backend: {0}")]
	Unsupported(String),

	///
	#[error("strip prefix error: {0}")]
	StripPrefix(#[from] StripPrefixError),

	///
	#[error("utf8 error:{0}")]
	Utf8Conversion(#[from] FromUtf8Error),

	///
	#[error("TryFromInt error:{0}")]
	IntConversion(#[from] TryFromIntError),

	///
	#[error("EasyCast error:{0}")]
	EasyCast(#[from] easy_cast::Error),

	///
	#[error("rayon error: {0}")]
	ThreadPool(#[from] rayon_core::ThreadPoolBuildError),

	// Kept for source-compat with the former git backend.
	///
	#[error("git: conflict during rebase")]
	RebaseConflict,

	///
	#[error("git: remote url not found")]
	UnknownRemote,

	///
	#[error("git: inconclusive remotes")]
	NoDefaultRemoteFound,
}

///
pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<std::sync::PoisonError<T>> for Error {
	fn from(error: std::sync::PoisonError<T>) -> Self {
		Self::Generic(format!("poison error: {error}"))
	}
}

impl<T> From<crossbeam_channel::SendError<T>> for Error {
	fn from(error: crossbeam_channel::SendError<T>) -> Self {
		Self::Generic(format!("send error: {error}"))
	}
}

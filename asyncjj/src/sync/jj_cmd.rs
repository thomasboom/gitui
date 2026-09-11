//! Low-level `jj` CLI invocation.
//!
//! All automation goes through the `jj` binary (see
//! <https://docs.jj-vcs.dev/latest/cli-reference/>). We always pin
//! `--no-pager` and `--color never` so output is stable for parsing, and pass
//! `-R <repo>` explicitly instead of relying on cwd discovery.

use crate::error::{Error, Result};
use std::{
	ffi::OsStr,
	path::Path,
	process::{Command, Output},
};

/// Locate the `jj` binary.
pub fn jj_binary() -> Result<String> {
	const CANDIDATE: &str = "jj";
	// Respect PATH; give a clear error if missing.
	which::which(CANDIDATE).map_or_else(
		|_| {
			Err(Error::JjNotFound(
				"`jj` binary not found in PATH. Install Jujutsu: https://docs.jj-vcs.dev/latest/install-and-setup/"
					.to_string(),
			))
		},
		|p| Ok(p.to_string_lossy().into_owned()),
	)
}

/// Run `jj` with the given args in `repo_path` (workspace root or subdir).
///
/// Always prepends the global flags `--no-pager --color never`.
/// On non-zero exit, returns [`Error::Jj`] with stderr attached.
pub fn run_jj(
	repo_path: &Path,
	args: &[impl AsRef<OsStr>],
) -> Result<String> {
	run_jj_with(repo_path, args, None)
}

/// Run `jj`, optionally feeding `stdin`.
pub fn run_jj_with(
	repo_path: &Path,
	args: &[impl AsRef<OsStr>],
	stdin: Option<&[u8]>,
) -> Result<String> {
	let bin = jj_binary()?;
	let mut cmd = Command::new(&bin);
	cmd.arg("--no-pager")
		.arg("--color")
		.arg("never")
		.args(args)
		.current_dir(repo_path)
		.env("JJ_NO_PAGER", "1")
		.env("NO_PAGER", "1");

	// Human-readable command for error messages.
	let command_str = format!(
		"{bin} {}",
		args.iter()
			.map(|a| a.as_ref().to_string_lossy().into_owned())
			.collect::<Vec<_>>()
			.join(" ")
	);

	log::debug!("run: {command_str} (cwd={})", repo_path.display());

	let output: Output = if let Some(input) = stdin {
		use std::io::Write;
		cmd.stdin(std::process::Stdio::piped())
			.stdout(std::process::Stdio::piped())
			.stderr(std::process::Stdio::piped());
		let mut child = cmd.spawn()?;
		child
			.stdin
			.as_mut()
			.ok_or_else(|| {
				Error::Generic("failed to open jj stdin".to_string())
			})?
			.write_all(input)?;
		child.wait_with_output()?
	} else {
		cmd.output()?
	};

	if output.status.success() {
		Ok(String::from_utf8_lossy(&output.stdout).into_owned())
	} else {
		Err(Error::Jj {
			command: command_str,
			status: output.status.to_string(),
			stderr: String::from_utf8_lossy(&output.stderr)
				.trim()
				.to_string(),
		})
	}
}

/// Run `jj` with `-R <repo>` pinning (for calls where cwd may be outside the repo).
pub fn run_jj_in(
	repo_root: &Path,
	args: &[impl AsRef<OsStr>],
) -> Result<String> {
	let owned: Vec<std::ffi::OsString> =
		std::iter::once(OsStr::new("-R").to_os_string())
			.chain(std::iter::once(
				repo_root.as_os_str().to_os_string(),
			))
			.chain(args.iter().map(|a| a.as_ref().to_os_string()))
			.collect();
	run_jj(repo_root, &owned)
}

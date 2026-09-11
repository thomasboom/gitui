use crate::{
	error::Result,
	sync::{CommitId, LogWalker, RepoPath, SharedCommitFilterFn},
	AsyncJjNotification as AsyncGitNotification, Error,
};
use crossbeam_channel::Sender;
use scopetime::scope_time;
use std::{
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc, Mutex,
	},
	thread,
	time::{Duration, Instant},
};

///
#[derive(PartialEq, Eq, Debug)]
pub enum FetchStatus {
	/// previous fetch still running
	Pending,
	/// no change expected
	NoChange,
	/// new walk was started
	Started,
}

///
pub struct AsyncLogResult {
	///
	pub commits: Vec<CommitId>,
	///
	pub duration: Duration,
}
///
pub struct AsyncLog {
	current: Arc<Mutex<AsyncLogResult>>,
	current_head: Arc<Mutex<Option<CommitId>>>,
	sender: Sender<AsyncGitNotification>,
	pending: Arc<AtomicBool>,
	background: Arc<AtomicBool>,
	filter: Option<SharedCommitFilterFn>,
	partial_extract: AtomicBool,
	repo: RepoPath,
}

static LIMIT_COUNT: usize = 3000;
static SLEEP_FOREGROUND: Duration = Duration::from_millis(2);
static SLEEP_BACKGROUND: Duration = Duration::from_secs(1);

impl AsyncLog {
	///
	pub fn new(
		repo: RepoPath,
		sender: &Sender<AsyncGitNotification>,
		filter: Option<SharedCommitFilterFn>,
	) -> Self {
		Self {
			repo,
			current: Arc::new(Mutex::new(AsyncLogResult {
				commits: Vec::new(),
				duration: Duration::default(),
			})),
			current_head: Arc::new(Mutex::new(None)),
			sender: sender.clone(),
			pending: Arc::new(AtomicBool::new(false)),
			background: Arc::new(AtomicBool::new(false)),
			filter,
			partial_extract: AtomicBool::new(false),
		}
	}

	///
	pub fn count(&self) -> Result<usize> {
		Ok(self.current.lock()?.commits.len())
	}

	///
	pub fn get_slice(
		&self,
		start_index: usize,
		amount: usize,
	) -> Result<Vec<CommitId>> {
		if self.partial_extract.load(Ordering::Relaxed) {
			return Err(Error::Generic(String::from("Faulty usage of AsyncLog: Cannot partially extract items and rely on get_items slice to still work!")));
		}

		let list = &self.current.lock()?.commits;
		let list_len = list.len();
		let min = start_index.min(list_len);
		let max = min + amount;
		let max = max.min(list_len);
		Ok(list[min..max].to_vec())
	}

	///
	pub fn get_items(&self) -> Result<Vec<CommitId>> {
		if self.partial_extract.load(Ordering::Relaxed) {
			return Err(Error::Generic(String::from("Faulty usage of AsyncLog: Cannot partially extract items and rely on get_items slice to still work!")));
		}

		let list = &self.current.lock()?.commits;
		Ok(list.clone())
	}

	///
	pub fn extract_items(&self) -> Result<Vec<CommitId>> {
		self.partial_extract.store(true, Ordering::Relaxed);
		let list = &mut self.current.lock()?.commits;
		let result = list.clone();
		list.clear();
		Ok(result)
	}

	///
	pub fn get_last_duration(&self) -> Result<Duration> {
		Ok(self.current.lock()?.duration)
	}

	///
	pub fn is_pending(&self) -> bool {
		self.pending.load(Ordering::Relaxed)
	}

	///
	pub fn set_background(&self) {
		self.background.store(true, Ordering::Relaxed);
	}

	///
	fn current_head(&self) -> Result<Option<CommitId>> {
		Ok(self.current_head.lock()?.clone())
	}

	///
	fn head_changed(&self) -> Result<bool> {
		if let Ok(head) = CommitId::from_revision(&self.repo, "@") {
			return Ok(Some(head) != self.current_head()?);
		}
		Ok(false)
	}

	///
	pub fn fetch(&self) -> Result<FetchStatus> {
		self.background.store(false, Ordering::Relaxed);

		if self.is_pending() {
			return Ok(FetchStatus::Pending);
		}

		if !self.head_changed()? {
			return Ok(FetchStatus::NoChange);
		}

		self.pending.store(true, Ordering::Relaxed);

		self.clear()?;

		let arc_current = Arc::clone(&self.current);
		let sender = self.sender.clone();
		let arc_pending = Arc::clone(&self.pending);
		let arc_background = Arc::clone(&self.background);
		let filter = self.filter.clone();
		let repo_path = self.repo.clone();

		if let Ok(head) = CommitId::from_revision(&self.repo, "@") {
			*self.current_head.lock()? = Some(head);
		}

		rayon_core::spawn(move || {
			scope_time!("async::revlog");

			Self::fetch_helper(
				&repo_path,
				&arc_current,
				&arc_background,
				&sender,
				filter,
			)
			.expect("failed to fetch");

			arc_pending.store(false, Ordering::Relaxed);

			Self::notify(&sender);
		});

		Ok(FetchStatus::Started)
	}

	fn fetch_helper(
		repo_path: &RepoPath,
		arc_current: &Arc<Mutex<AsyncLogResult>>,
		arc_background: &Arc<AtomicBool>,
		sender: &Sender<AsyncGitNotification>,
		filter: Option<SharedCommitFilterFn>,
	) -> Result<()> {
		let start_time = Instant::now();

		let walker = LogWalker::new(repo_path, LIMIT_COUNT)?;
		let mut entries = Vec::with_capacity(LIMIT_COUNT);
		// `LogWalker::read` fills the vec via push, it does not `read()`
		// bytes into it.
		#[allow(clippy::read_zero_byte_vec)]
		walker.read(&mut entries)?;

		let commits = if let Some(filter) = filter {
			let mut kept = Vec::new();
			for (i, id) in entries.into_iter().enumerate() {
				if filter(repo_path, &id).unwrap_or(true) {
					kept.push(id);
				}
				if i % 200 == 0 {
					let mut current = arc_current.lock()?;
					current.commits.clone_from(&kept);
					current.duration = start_time.elapsed();
					drop(current);
					Self::notify(sender);

					let sleep_duration =
						if arc_background.load(Ordering::Relaxed) {
							SLEEP_BACKGROUND
						} else {
							SLEEP_FOREGROUND
						};
					thread::sleep(sleep_duration);
				}
			}
			kept
		} else {
			entries
		};

		let mut current = arc_current.lock()?;
		current.commits = commits;
		current.duration = start_time.elapsed();

		Ok(())
	}

	fn clear(&self) -> Result<()> {
		self.current.lock()?.commits.clear();
		*self.current_head.lock()? = None;
		self.partial_extract.store(false, Ordering::Relaxed);
		Ok(())
	}

	fn notify(sender: &Sender<AsyncGitNotification>) {
		sender
			.send(AsyncGitNotification::Log)
			.expect("error sending");
	}
}

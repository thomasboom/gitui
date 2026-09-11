//! Progress reporting for `jj git fetch` / `jj git push`.
//!
//! The CLI does not stream machine-readable progress, so notifications are
//! coarse (`Update` → `Done`).

use crate::{
	error::Result,
	progress::ProgressPercent,
	sync::remotes::push::{AsyncProgress, ProgressNotification},
	AsyncJjNotification,
};
use crossbeam_channel::{Receiver, Sender};
use std::{
	sync::{Arc, Mutex},
	thread::{self, JoinHandle},
};

/// used for push/pull
#[derive(Clone, Debug)]
pub enum RemoteProgressState {
	///
	PackingAddingObject,
	///
	PackingDeltafiction,
	///
	Pushing,
	/// fetch progress
	Transfer,
	/// remote progress done
	Done,
}

///
#[derive(Clone, Debug)]
pub struct RemoteProgress {
	///
	pub state: RemoteProgressState,
	///
	pub progress: ProgressPercent,
}

impl RemoteProgress {
	///
	pub fn new(
		state: RemoteProgressState,
		current: usize,
		total: usize,
	) -> Self {
		Self {
			state,
			progress: ProgressPercent::new(current, total),
		}
	}

	///
	pub const fn get_progress_percent(&self) -> u8 {
		self.progress.progress
	}

	pub(crate) fn set_progress<T>(
		progress: &Arc<Mutex<Option<T>>>,
		state: Option<T>,
	) -> Result<()> {
		let mut progress = progress.lock()?;

		*progress = state;

		Ok(())
	}

	/// spawn thread to listen to progress notifications coming in from blocking remote jj method (fetch/push)
	pub(crate) fn spawn_receiver_thread<
		T: 'static + AsyncProgress,
	>(
		notification_type: AsyncJjNotification,
		sender: Sender<AsyncJjNotification>,
		receiver: Receiver<T>,
		progress: Arc<Mutex<Option<T>>>,
	) -> JoinHandle<()> {
		thread::spawn(move || loop {
			let incoming = receiver.recv();
			match incoming {
				Ok(update) => {
					Self::set_progress(
						&progress,
						Some(update.clone()),
					)
					.expect("set progress failed");
					sender
						.send(notification_type)
						.expect("Notification error");

					thread::yield_now();

					if update.is_done() {
						break;
					}
				}
				Err(e) => {
					log::error!(
						"remote progress receiver error: {e}",
					);
					break;
				}
			}
		})
	}
}

impl From<ProgressNotification> for RemoteProgress {
	fn from(progress: ProgressNotification) -> Self {
		match progress {
			ProgressNotification::Update => {
				Self::new(RemoteProgressState::Transfer, 0, 1)
			}
			ProgressNotification::Done => {
				Self::new(RemoteProgressState::Done, 1, 1)
			}
		}
	}
}

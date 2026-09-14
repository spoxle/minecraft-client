use std::{borrow::Cow, fs, path::PathBuf};

use gpui::{AssetSource, SharedString};

pub struct Assets {
	base: PathBuf,
}

impl Assets {
	pub fn new() -> Self {
		Self {
			base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
		}
	}
}

impl AssetSource for Assets {
	fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
		Ok(Some(Cow::Owned(fs::read(self.base.join(path))?)))
	}

	fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
		Ok(fs::read_dir(self.base.join(path))?
			.filter_map(|entry| {
				entry
					.ok()
					.and_then(|entry| entry.file_name().into_string().ok())
					.map(SharedString::from)
			})
			.collect())
	}
}

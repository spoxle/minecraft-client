use gpui::*;

use crate::ui::pages::{explore, library, profile, settings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PageId {
	Library,
	Explore,
	Profile,
	Settings,
}

impl PageId {
	pub const ALL: &'static [Self] =
		&[Self::Library, Self::Explore, Self::Profile, Self::Settings];

	pub const fn as_str(self) -> &'static str {
		match self {
			Self::Library => "library",
			Self::Explore => "explore",
			Self::Profile => "profile",
			Self::Settings => "settings",
		}
	}
}

pub struct Page {
	current: PageId,
	library: Entity<library::Library>,
	explore: Entity<explore::Explore>,
	profile: Entity<profile::Profile>,
	settings: Entity<settings::Settings>,
}

impl Page {
	pub fn new(cx: &mut Context<Self>) -> Self {
		Self {
			current: PageId::Library,
			library: cx.new(|_| library::Library {}),
			explore: cx.new(|_| explore::Explore {}),
			profile: cx.new(|_| profile::Profile {}),
			settings: cx.new(|_| settings::Settings {}),
		}
	}

	pub fn current(&self) -> PageId {
		self.current
	}

	pub fn navigate(&mut self, page: PageId, cx: &mut Context<Self>) {
		if self.current != page {
			self.current = page;
			cx.notify();
		}
	}
}

impl Render for Page {
	fn render(
		&mut self,
		_window: &mut Window,
		_cx: &mut Context<Self>,
	) -> impl IntoElement {
		let content = match self.current {
			PageId::Library => self
				.library
				.clone()
				.into_any_element(),
			PageId::Explore => self
				.explore
				.clone()
				.into_any_element(),
			PageId::Profile => self
				.profile
				.clone()
				.into_any_element(),
			PageId::Settings => self
				.settings
				.clone()
				.into_any_element(),
		};

		div()
			.h_full()
			.p_2()
			.flex_grow()
			.child(content)
	}
}

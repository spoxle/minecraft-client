use gpui::prelude::*;
use gpui::*;

use crate::ui::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
	Library,
	Explore,
	Profile,
	Settings,
}

impl SidebarTab {
	const ALL: &'static [Self] = &[Self::Library, Self::Explore, Self::Profile, Self::Settings];

	fn as_str(self) -> &'static str {
		match self {
			Self::Library => "library",
			Self::Explore => "explore",
			Self::Profile => "profile",
			Self::Settings => "settings",
		}
	}
}

pub struct Sidebar {
	pub active_tab: SidebarTab,
}

impl Sidebar {
	fn render_button(&self, tab: SidebarTab, cx: &Context<Self>) -> impl IntoElement {
		let is_active = self.active_tab == tab;
		let label = tab.as_str();
		let icon_path = format!("icons/{label}.svg");
		let icon_color = if is_active {
			Theme::TEXT
		} else {
			Theme::TEXT_2
		};

		div()
			.group(label)
			.id(label)
			.when(!is_active, |element| {
				element.hover(|element| element.bg(rgb(Theme::SECONDARY)))
			})
			.when(is_active, |element| element.bg(rgb(Theme::ACCENT)))
			.on_click(cx.listener(move |sidebar, _, _, cx| {
				sidebar.active_tab = tab;
				cx.notify();
			}))
			.p_2()
			.rounded_lg()
			.child(
				svg()
					.when(!is_active, |icon| {
						icon.group_hover(label, |style| style.text_color(rgb(Theme::TEXT)))
					})
					.text_color(rgb(icon_color))
					.path(icon_path)
					.size_6(),
			)
	}
}

impl Render for Sidebar {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		div()
			.h_full()
			.bg(rgb(Theme::FOREGROUND))
			.flex()
			.flex_col()
			.gap_2()
			.p_2()
			.children(
				SidebarTab::ALL
					.iter()
					.copied()
					.map(|tab| self.render_button(tab, cx)),
			)
	}
}

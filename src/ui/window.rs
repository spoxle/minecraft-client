use gpui::*;

use crate::ui::assets::Assets;
use crate::ui::components::sidebar::*;
use crate::ui::theme::Theme;

pub struct WindowView {
	sidebar: Entity<Sidebar>,
}

impl WindowView {
	pub fn run() {
		Application::new()
			.with_assets(Assets::new())
			.run(|cx: &mut App| {
				cx.open_window(WindowOptions::default(), |_, cx| {
					cx.new(|cx| WindowView {
						sidebar: cx.new(|_| Sidebar {
							active_tab: SidebarTab::Explore,
						}),
					})
				})
				.unwrap();
			});
	}
}

impl Render for WindowView {
	fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
		div()
			.size_full()
			.bg(rgb(Theme::BACKGROUND))
			.flex()
			.flex_col()
			.child()
	}
}

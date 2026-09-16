use gpui::*;

use crate::ui::assets::Assets;
use crate::ui::components::*;
use crate::ui::theme::Theme;

pub struct WindowView {
	sidebar: Entity<sidebar::Sidebar>,
	footer: Entity<footer::Footer>,
}

impl WindowView {
	pub fn run() {
		Application::new()
			.with_assets(Assets::new())
			.run(|cx: &mut App| {
				cx.open_window(
					WindowOptions {
						window_bounds: Some(WindowBounds::centered(
							size(px(800.0), px(500.0)),
							cx,
						)),
						..WindowOptions::default()
					},
					|_, cx| {
						cx.new(|cx| WindowView {
							sidebar: cx.new(|_| sidebar::Sidebar {
								active_tab: sidebar::SidebarTab::Library,
							}),
							footer: cx
								.new(|_| footer::Footer { current_task: None }),
						})
					},
				)
				.unwrap();
			});
	}
}

impl Render for WindowView {
	fn render(
		&mut self,
		_window: &mut Window,
		_cx: &mut Context<Self>,
	) -> impl IntoElement {
		div()
			.size_full()
			.bg(rgb(Theme::BACKGROUND))
			.flex()
			.flex_col()
			.child(
				// upper region
				div()
					.flex()
					.flex_row()
					.flex_grow()
					.child(self.sidebar.clone()) // sidebar component
					.child(
						// main content
						div()
							.flex_grow()
							.bg(rgb(Theme::BACKGROUND)),
					),
			)
			.child(self.footer.clone())
	}
}

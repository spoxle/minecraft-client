use gpui::*;

use crate::ui::assets::Assets;
use crate::ui::components::*;
use crate::ui::theme::Theme;

pub struct WindowView {
	sidebar: Entity<sidebar::Sidebar>,
	footer: Entity<footer::Footer>,
	page: Entity<page::Page>,
}

impl WindowView {
	pub fn new(cx: &mut Context<Self>) -> Self {
		let page = cx.new(page::Page::new);
		let sidebar = cx.new(|cx| sidebar::Sidebar::new(page.clone(), cx));

		Self {
			sidebar,
			footer: cx.new(|_|, footer::Footer::new()),
			page,
		}
	}

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
					|_, cx| cx.new(|cx| WindowView::new(cx)),
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
			.text_color(rgb(Theme::TEXT))
			.flex()
			.flex_col()
			.child(
				// upper region
				div()
					.flex()
					.flex_row()
					.flex_grow()
					.child(self.sidebar.clone())
					.child(self.page.clone()),
			)
			.child(self.footer.clone())
	}
}

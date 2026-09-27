use gpui::prelude::*;
use gpui::*;

use crate::ui::components::page::{Page, PageId};
use crate::ui::theme::Theme;

pub struct Sidebar {
	page: Entity<Page>,
}

impl Sidebar {
	pub fn new(page: Entity<Page>, cx: &mut Context<Self>) -> Self {
		cx.observe(&page, |_, _, cx| cx.notify())
			.detach();

		Self { page }
	}

	fn render_button(
		&self,
		tab: PageId,
		cx: &Context<Self>,
	) -> impl IntoElement {
		let is_active = self.page.read(cx).current() == tab;
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
				sidebar
					.page
					.update(cx, |page, cx| page.navigate(tab, cx));
			}))
			.p_2()
			.rounded_lg()
			.child(
				svg()
					.when(!is_active, |icon| {
						icon.group_hover(label, |style| {
							style.text_color(rgb(Theme::TEXT))
						})
					})
					.text_color(rgb(icon_color))
					.path(icon_path)
					.size_6(),
			)
	}
}

impl Render for Sidebar {
	fn render(
		&mut self,
		_window: &mut Window,
		cx: &mut Context<Self>,
	) -> impl IntoElement {
		div()
			.h_full()
			.bg(rgb(Theme::FOREGROUND))
			.flex()
			.flex_col()
			.gap_2()
			.p_2()
			.border_r_1()
			.border_color(rgb(Theme::BORDER))
			.children(
				PageId::ALL
					.iter()
					.copied()
					.map(|tab| self.render_button(tab, cx)),
			)
	}
}

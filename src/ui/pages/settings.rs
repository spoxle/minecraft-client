use gpui::*;

use crate::ui::theme::Theme;

pub struct Settings {}

impl Render for Settings {
	fn render(
		&mut self,
		_window: &mut Window,
		_cx: &mut Context<Self>,
	) -> impl IntoElement {
		div()
			.child("Settings Page")
			.text_color(rgb(Theme::TEXT))
	}
}

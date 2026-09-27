use gpui::*;

pub struct Library {}

impl Render for Library {
	fn render(
		&mut self,
		_window: &mut Window,
		_cx: &mut Context<Self>,
	) -> impl IntoElement {
		div().child("Library Page")
	}
}

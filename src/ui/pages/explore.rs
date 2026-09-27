use gpui::*;

pub struct Explore {}

impl Render for Explore {
	fn render(
		&mut self,
		_window: &mut Window,
		_cx: &mut Context<Self>,
	) -> impl IntoElement {
		div().child("Explore Page")
	}
}

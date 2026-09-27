use gpui::*;

pub struct Profile {}

impl Render for Profile {
	fn render(
		&mut self,
		_window: &mut Window,
		_cx: &mut Context<Self>,
	) -> impl IntoElement {
		div().child("Profile Page")
	}
}

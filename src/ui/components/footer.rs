use gpui::*;

use crate::ui::theme::Theme;

pub struct Footer {
	pub current_task: Option<SharedString>,
}

impl Footer {
	fn update_task(&mut self, task: String, cx: &mut Context<Self>) {
		self.current_task = Some(task.into());
		cx.notify()
	}
}

impl Render for Footer {
	fn render(
		&mut self,
		_window: &mut Window,
		_cx: &mut Context<Self>,
	) -> impl IntoElement {
		div()
			.p_1()
			.flex()
			.flex_row()
			.gap_1()
			.children(
				// task
				self.current_task
					.clone()
					.map(|task| {
						div().child(task).child(
							svg()
								.path("icons/progress.svg")
								.size_3p5()
								.text_color(rgb(Theme::TEXT)),
						)
					}),
			)
	}
}

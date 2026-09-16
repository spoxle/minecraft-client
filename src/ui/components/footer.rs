use gpui::*;

use crate::ui::theme::Theme;

pub struct Footer {
	pub current_task: Option<SharedString>,
	task_queue: Vec<SharedString>,
}

impl Footer {
	pub fn new() -> Self {
		Self {
			current_task: None,
			task_queue: Vec::new(),
		}
	}

	fn add_task(
		&mut self,
		task: impl Into<SharedString>,
		cx: &mut Context<Self>,
	) {
		let task = task.into();

		self.current_task = Some(task.clone());
		self.task_queue.push(task);
		// cx.notify()
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
						div()
							.child(format!(
								"{}{}",
								task,
								if self.task_queue.len() > 1 {
									format!("(+{})", self.task_queue.len() - 1)
								} else {
									String::new()
								}
							))
							.child(
								svg()
									.path("icons/progress.svg")
									.size_3p5()
									.text_color(rgb(Theme::TEXT)),
							)
					}),
			)
	}
}

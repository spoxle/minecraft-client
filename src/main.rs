pub mod ui;

fn main() {
	ui::window::run(|window, cx| {
		window.update(cx, |view, _window, cx| {
			view.set_task("what the jankus", cx)
		})
		.expect("failed to update window");
	});
}

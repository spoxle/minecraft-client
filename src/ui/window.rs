use std::{f32::consts::TAU, time::Duration};

use gpui::*;
use rust_embed::RustEmbed;

use crate::ui::components;
use crate::ui::pages;

#[derive(RustEmbed)]
#[folder = "assets/"]
struct AppAssets;

impl AssetSource for AppAssets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        match Self::get(path) {
            Some(file) => Ok(Some(file.data)),
            None => Ok(None),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let files = Self::iter()
            .filter(|f| f.starts_with(path))
            .map(|f| SharedString::from(f))
            .collect();
        Ok(files)
    }
}

pub enum Page {
	Instances,
	Explore,
	Settings,
	Profile,
}

pub struct WindowView {
	pub current_page: Page,
	pub current_task: Option<SharedString>,
}

impl WindowView {
	pub fn set_task(&mut self, text: impl Into<SharedString>, cx: &mut Context<'_, Self>) {
		let text = text.into();

		if self.current_task.as_ref() == Some(&text) {
			return;
		}

		self.current_task = Some(text);

		cx.notify();
	}

	fn set_current_page(&mut self, page: Page, cx: &mut Context<'_, Self>) {
		self.current_page = page;

		cx.notify();
	}

	fn render_current_page(&self) -> Div {
		match self.current_page {
			Page::Instances => pages::instances::instances_page(),
			Page::Explore => pages::explore::explore_page(),
			Page::Settings => pages::settings::settings_page(),
			Page::Profile => pages::profile::profile_page(),
		}
	}
}

impl Render for WindowView {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		div() // MAIN WINDOW FILL
			.size_full()
			.bg(rgb(0xff00ff))
			.flex()
			.flex_col()
			.size_full()
			.child( // UPPER REGION
				div()
					.flex()
					.flex_row()
					.flex_grow()
					.w_full()
					.child( // SIDEBAR
						div()
							.bg(rgb(0x010409))
							.border_r_1()
							.border_color(rgb(0x3d444d))
							.flex()
							.flex_col()
							.p_2()
							.child( // UPPER SIDEBAR
								div()
									.flex()
									.flex_col()
									.flex_grow()
									.gap(px(8.0))
									.child(
										components::sidebar_button::sidebar_button("instances")
											.on_click(cx.listener(|this, _event, _window, cx| {
												this.set_current_page(Page::Instances, cx);
											}))
									)
									.child(
										components::sidebar_button::sidebar_button("explore")
											.on_click(cx.listener(|this, _event, _window, cx| {
												this.set_current_page(Page::Explore, cx);
											}))
									)
							)
							.child( // LOWER SIDEBAR
								div()
									.flex()
									.flex_col()
									.gap(px(8.0))
									.child(
										components::sidebar_button::sidebar_button("settings")
											.on_click(cx.listener(|this, _event, _window, cx| {
												this.set_current_page(Page::Settings, cx);
											}))
									)
									.child(
										components::sidebar_button::sidebar_button("profile")
											.on_click(cx.listener(|this, _event, _window, cx| {
												this.set_current_page(Page::Profile, cx);
											}))
									)
							)
					)
					.child( // CONTENT
						div()
							.flex_grow()
							.bg(rgb(0x0d1117))
							.p_4()
							.child(self.render_current_page()) // MAIN PAGE SWAP LAYOUT
					)
			)
			.child( // FOOTER
				div()
					.bg(rgb(0x010409))
					.w_full()
					.border_t_1()
					.border_color(rgb(0x3d444d))
					.flex()
					.flex_row()
					.items_center()
					.p_1()
					.child( // LEFT FOOTER
						div()
							.flex()
							.flex_row()
							.flex_grow()
							.gap(px(4.0))
							.children(self.current_task.as_ref().map(|task| {
								div()
									.flex()
									.flex_row()
									.items_center()
									.gap(px(4.0))
									.p_1()
									.child(
										svg()
											.path("icons/spinner.svg")
											.size(px(12.0))
											.text_color(rgb(0xf0f6fc))
											.with_animation("spin_infinite", Animation::new(Duration::from_secs(1)).repeat(), |svg, delta| {
												let angle = Radians(delta * TAU);

												svg.with_transformation(Transformation::rotate(angle))
											})
									)
									.child(
										div()
											.text_color(rgb(0xf0f6fc))
											.text_size(px(12.0))
											.child(task.clone())
									)
							}))
					)
					.child( // RIGHT FOOTER
						div()
							.flex()
							.flex_row()
							.gap(px(4.0))
							.p_1()
							.child(
								svg()
									.path("icons/terminal.svg")
									.size(px(12.0))
									.text_color(rgb(0xf0f6fc))
							)
					)
			)
	}
}

pub fn run(on_start: impl FnOnce(WindowHandle<WindowView>, &mut App) + 'static) {
	Application::new()
		.with_assets(AppAssets)
		.run(|cx: &mut App| {
			let window = cx.open_window(
				WindowOptions {
					titlebar: Some(TitlebarOptions {
						title: Some("Minecraft & VPS Manager".into()),
						..Default::default()
					}),
					window_bounds: Some(WindowBounds::Windowed(
						Bounds::centered(None, size(px(800.0), px(600.0)), cx)
					)),
					window_min_size: Some(Size {
						width: px(600.0),
						height: px(400.0),
					}),
					..Default::default()
				},
				|_, cx| cx.new(|_| WindowView {
					current_page: Page::Instances,
					current_task: None,
				}),
			)
			.expect("failed to open application window");

			on_start(window, cx);
			cx.activate(true);
	});
}

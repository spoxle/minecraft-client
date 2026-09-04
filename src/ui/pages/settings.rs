use gpui::{ Div, rgb, div, prelude::* };

pub fn settings_page() -> Div {
	div()
		.text_color(rgb(0xffffff))
		.child("Settings page")
}

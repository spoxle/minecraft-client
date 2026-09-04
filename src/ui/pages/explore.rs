use gpui::{ Div, rgb, div, prelude::* };

pub fn explore_page() -> Div {
	div()
		.text_color(rgb(0xffffff))
		.child("Explore page")
}

use gpui::{ Div, rgb, div, prelude::* };

pub fn instances_page() -> Div {
	div()
		.text_color(rgb(0xffffff))
		.child("Instances page")
}

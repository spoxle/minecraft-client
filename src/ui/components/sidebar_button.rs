use gpui::{ Div, SharedString, Stateful, div, prelude::*, px, rgb, svg };

pub fn sidebar_button(icon: &str) -> Stateful<Div> {
	let icon_path = format!("icons/{}_large.svg", icon);
	let id_name = format!("sidebar-{}-button", icon);

	div()
		.p_2()
		.rounded(px(8.0))
		.id(SharedString::from(id_name))
		.hover(|style| {
			style.bg(rgb(0x15191f))
		})
		.child(
			svg()
				.path(icon_path)
				.text_color(rgb(0xf0f6fc))
				.size(px(24.0))
		)
}

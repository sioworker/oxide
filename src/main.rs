use eframe::egui;
use oxide::html;

fn main() -> eframe::Result {
	let src = std::env::args().nth(1).and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or("<html><body><h1>Hello, World!</h1></body></html>".into()); // file arg or def
	let t = html::parse(&src).iter().map(html::Node::text).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" ");
	eframe::run_ui_native("oxide", Default::default(), move |ui, _| {
		egui::ScrollArea::vertical().show(ui, |ui| ui.label(&t));
	})
}

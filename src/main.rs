fn main() -> eframe::Result {
	eframe::run_ui_native("oxide", Default::default(), |ui, _| {
		ui.centered_and_justified(|ui| ui.heading("Hello, World!"));
	})
}

trait View {
    fn draw() -> ();
}

pub struct TerminalView {

}

impl TerminalView {

}

impl View for TerminalView {
    fn draw() {}
}
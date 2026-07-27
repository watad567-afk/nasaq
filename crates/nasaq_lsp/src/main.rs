fn main() {
    if let Err(err) = nasaq_lsp::run_stdio() {
        eprintln!("nasaq-lsp error: {err}");
        std::process::exit(1);
    }
}

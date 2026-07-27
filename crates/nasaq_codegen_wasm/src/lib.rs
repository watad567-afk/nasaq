//! WebAssembly codegen for Nasaq — emits real i32 modules via wasm-encoder.

mod emit;

pub struct WasmOutput {
    pub bytes: Vec<u8>,
    pub wat: String,
}

pub fn compile_to_wasm(source: &str) -> WasmOutput {
    let parsed = nasaq_parser::parse_program(source);
    if parsed.diagnostics.has_errors() || parsed.program.is_none() {
        return emit::empty_module("nasaq_module");
    }
    let program = parsed.program.unwrap();
    let resolved = nasaq_resolver::resolve(&program);
    if resolved.diagnostics.has_errors() {
        return emit::empty_module(
            program
                .module
                .as_ref()
                .map(|m| m.node.name.node.as_str())
                .unwrap_or("nasaq_module"),
        );
    }
    let typed = nasaq_types::typecheck(&program);
    if typed.diagnostics.has_errors() {
        return emit::empty_module(
            program
                .module
                .as_ref()
                .map(|m| m.node.name.node.as_str())
                .unwrap_or("nasaq_module"),
        );
    }
    emit::compile_program(&program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_wasm_magic() {
        let out = compile_to_wasm(
            "module test\nexport fn main() -> Int {\n    return 42\n}\n",
        );
        assert_eq!(&out.bytes[0..4], b"\0asm");
        assert!(out.bytes.len() > 8);
        assert!(out.wat.contains("i32.const"));
    }

    #[test]
    fn compiles_add_fn() {
        let out = compile_to_wasm(
            r#"
            module math
            export fn add(a: Int, b: Int) -> Int {
                return a + b
            }
            "#,
        );
        assert!(out.wat.contains("add"));
        assert!(out.bytes.len() > 40);
    }
}

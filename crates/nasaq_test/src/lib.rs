//! Integration tests for the Nasaq compiler pipeline.

#[cfg(test)]
mod pipeline {
    use nasaq_codegen_js::{CodegenOptions, emit_module};
    use nasaq_hir::lower;
    use nasaq_parser::parse_program;
    use nasaq_syntax::SourceFile;

    fn compile(source: &str) -> String {
        let parsed = parse_program(source);
        assert!(
            !parsed.diagnostics.has_errors(),
            "{:?}",
            parsed.diagnostics.diagnostics
        );
        let hir = lower(parsed.program.unwrap());
        emit_module(
            &hir,
            &CodegenOptions {
                module_name: "test".into(),
                runtime_import: "./runtime/core.js".into(),
                dom_runtime_import: "./runtime/dom.js".into(),
                source_map: false,
                web_mount: None,
                hydrate: false,
            },
        )
        .js
    }

    #[test]
    fn golden_hello_world() {
        let js = compile(
            r#"
            extern fn println(value: String)
            export fn main() {
                println("Hello, Nasaq!")
            }
            "#,
        );
        assert!(js.contains("export function main"));
        assert!(js.contains("Hello, Nasaq!"));
    }

    #[test]
    fn golden_fibonacci() {
        let js = compile(
            r#"
            export fn fib(n: Int) -> Int {
                if n <= 1 {
                    return n
                }
                return fib(n - 1) + fib(n - 2)
            }
            "#,
        );
        assert!(js.contains("function fib"));
    }

    #[test]
    fn source_file_line_col() {
        let file = SourceFile::new("test.nasaq", "a\nbc");
        assert_eq!(file.line_col(0), (1, 1));
        assert_eq!(file.line_col(3), (2, 2));
    }
}

#[cfg(test)]
mod conformance {
    use std::fs;
    use std::path::{Path, PathBuf};

    use nasaq_loader::load_program;
    use nasaq_parser::parse_program;
    use nasaq_resolver::resolve;
    use nasaq_types::typecheck;
    use walkdir::WalkDir;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn conformance_root() -> PathBuf {
        repo_root().join("tests/conformance")
    }

    fn expect_from_source(source: &str) -> String {
        for line in source.lines().take(5) {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("// expect:") {
                return rest.trim().to_string();
            }
        }
        "ok".to_string()
    }

    fn check_source(source: &str) -> bool {
        let parsed = parse_program(source);
        if parsed.diagnostics.has_errors() {
            return false;
        }
        let Some(program) = parsed.program else {
            return false;
        };
        let resolved = resolve(&program);
        if resolved.diagnostics.has_errors() {
            return false;
        }
        let typed = typecheck(&program);
        !typed.diagnostics.has_errors()
    }

    fn check_file(path: &Path) -> bool {
        let loaded = load_program(path);
        if loaded.diagnostics.has_errors() {
            return false;
        }
        let resolved = resolve(&loaded.program);
        if resolved.diagnostics.has_errors() {
            return false;
        }
        let typed = typecheck(&loaded.program);
        !typed.diagnostics.has_errors()
    }

    #[test]
    fn conformance_files() {
        let root = conformance_root();
        if !root.is_dir() {
            return;
        }
        let mut count = 0usize;
        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("nq") {
                continue;
            }
            let source = fs::read_to_string(path).expect("read conformance file");
            let expect = expect_from_source(&source);
            let ok = check_file(path);
            assert_eq!(
                ok,
                expect == "ok",
                "{} expected {}, got {}",
                path.display(),
                expect,
                if ok { "ok" } else { "error" }
            );
            count += 1;
        }
        assert!(count >= 10, "expected at least 10 conformance files, found {count}");
    }

    #[test]
    fn conformance_inline_suite() {
        let cases: &[(&str, &str, &str)] = &[
            ("let immut", "export fn main() { let x = 1 x = 2 }", "error"),
            ("fn ret void", "export fn main() { return }", "ok"),
            ("if expr", "export fn main() { let x = if true { 1 } else { 2 } }", "ok"),
            ("match int", "export fn main() { let x = match 1 { 1 => 10, _ => 0, } }", "ok"),
            ("unary neg", "export fn main() { let x = -5 }", "ok"),
            ("unary not", "export fn main() { let x = !true }", "ok"),
            ("cmp lt", "export fn main() { let x = 1 < 2 }", "ok"),
            ("cmp eq", "export fn main() { let x = 1 == 1 }", "ok"),
            ("add float", "export fn main() { let x = 1.5 + 2.5 }", "ok"),
            ("string lit", "export fn main() { let x = \"hello\" }", "ok"),
            ("mut let", "export fn main() { let mut x = 0 x = 1 }", "ok"),
            ("call fn", "export fn id(n: Int) -> Int { return n } export fn main() { let x = id(3) }", "ok"),
            ("nested if", "export fn main() { let x = if 1 > 0 { if 2 > 1 { 1 } else { 0 } } else { 0 } }", "ok"),
            ("while break", "export fn main() { while false { break; } }", "ok"),
            ("struct field", "export struct S { a: Int } export fn main() { let s = S { a: 1 } }", "ok"),
            ("component state", "export component C() { state n: Int = 0 view { <span>{ n }</span> } } export fn main() {}", "ok"),
            ("view arabic", "export component C() { view { <p>نَسَق</p> } } export fn main() {}", "ok"),
            ("view style", "export component C() { view { <div></div> } style scoped { .x { color: red; } } } export fn main() {}", "ok"),
            ("bad assign", "export fn main() { let x: Int = true }", "error"),
            ("unknown fn", "export fn main() { missing(1) }", "error"),
        ];
        for (name, source, expect) in cases {
            let ok = check_source(source);
            assert_eq!(
                ok,
                *expect == "ok",
                "case `{name}` expected {expect}, got {}",
                if ok { "ok" } else { "error" }
            );
        }
        assert!(cases.len() >= 20);
    }

    #[test]
    fn conformance_bulk_generated() {
        for i in 0..80 {
            let source = format!(
                "export fn f{i}(n: Int) -> Int {{ return n + {i} }}\nexport fn main() {{ let x = f{i}(1) }}"
            );
            assert!(check_source(&source), "generated case f{i} failed");
        }
    }
}

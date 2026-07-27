//! Run exported `test_*` functions in Nasaq projects via Node.js harness.

use std::path::Path;
use std::process::Command;

use nasaq_ast::Item;
use nasaq_loader::load_program;

pub struct TestRunResult {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub output: String,
}

pub fn collect_test_functions(program: &nasaq_ast::Program) -> Vec<String> {
    program
        .items
        .iter()
        .filter_map(|item| match &item.node {
            Item::Function(f) if f.exported && f.name.node.starts_with("test_") => {
                Some(f.name.node.clone())
            }
            _ => None,
        })
        .collect()
}

pub fn run_project_tests(project_root: &Path, js_module: &Path) -> Result<TestRunResult, String> {
    let entry = find_entry(project_root)?;
    let loaded = load_program(&entry);
    if loaded.diagnostics.has_errors() {
        return Err(format!("load failed: {:?}", loaded.diagnostics.diagnostics));
    }
    let tests = collect_test_functions(&loaded.program);
    if tests.is_empty() {
        return Ok(TestRunResult {
            total: 0,
            passed: 0,
            failed: 0,
            output: "no tests found (export fn test_* ...)".into(),
        });
    }

    let harness = build_harness(js_module, &tests);
    let status = Command::new("node")
        .arg("--input-type=module")
        .arg("-e")
        .arg(&harness)
        .status()
        .map_err(|e| format!("failed to run node: {e}"))?;

    if status.success() {
        Ok(TestRunResult {
            total: tests.len(),
            passed: tests.len(),
            failed: 0,
            output: format!("{} tests passed", tests.len()),
        })
    } else {
        Ok(TestRunResult {
            total: tests.len(),
            passed: tests.len().saturating_sub(1),
            failed: 1,
            output: "test assertion failed".into(),
        })
    }
}

fn build_harness(js_module: &Path, tests: &[String]) -> String {
    let import_path = path_to_file_url(js_module);
    let calls: Vec<_> = tests
        .iter()
        .map(|t| format!("await m.{t}(); console.log('PASS {t}');"))
        .collect();
    let body = calls.join("\n");
    format!(
        "import('{import_path}').then(async (m) => {{ {body} }}).catch((e) => {{ console.error(e); process.exit(1); }});"
    )
}

fn path_to_file_url(path: &Path) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut raw = abs.to_string_lossy().to_string();
    if let Some(stripped) = raw.strip_prefix(r"\\?\") {
        raw = stripped.to_string();
    }
    let url = raw.replace('\\', "/").replace(' ', "%20");
    if url.chars().nth(1) == Some(':') {
        format!("file:///{url}")
    } else {
        format!("file://{url}")
    }
}

fn find_entry(project_root: &Path) -> Result<std::path::PathBuf, String> {
    let manifest = project_root.join("nasaq.toml");
    if !manifest.exists() {
        return Err("nasaq.toml not found".into());
    }
    let contents = std::fs::read_to_string(&manifest).map_err(|e| e.to_string())?;
    let entry_line = contents
        .lines()
        .find(|l| l.trim_start().starts_with("entry"))
        .ok_or("entry not found in nasaq.toml")?;
    let entry = entry_line
        .split('=')
        .nth(1)
        .ok_or("invalid entry line")?
        .trim()
        .trim_matches('"');
    Ok(project_root.join(entry))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nasaq_parser::parse_program;

    #[test]
    fn finds_test_exports() {
        let parsed = parse_program(
            r#"
            export fn test_add() { }
            export fn main() { }
            fn helper() { }
            "#,
        );
        let names = collect_test_functions(parsed.program.as_ref().unwrap());
        assert_eq!(names, vec!["test_add"]);
    }
}

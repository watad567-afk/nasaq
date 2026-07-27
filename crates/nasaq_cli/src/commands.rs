use std::fs;
use std::path::Path;
use std::process::Command;

use miette::{IntoDiagnostic, Result, WrapErr};
use nasaq_ast::Item;
use nasaq_cache::CacheStore;
use nasaq_loader::load_program;

use crate::compile::{compile_loaded, render_loaded_diagnostics};
use crate::config::{load_project, read_source};

pub fn check(path: &str) -> Result<()> {
    let root = Path::new(path);
    let config = load_project(root)?;
    let entry = crate::config::entry_file(root, &config);
    let loaded = load_program(&entry);
    let output = compile_loaded(
        &loaded,
        &config.package.name,
        &runtime_import(&config),
        web_mount(&config),
        false,
    );
    print_loaded(&loaded, &output.diagnostics);
    if output.diagnostics.has_errors() {
        miette::bail!("check failed");
    }
    println!("✓ {} — no issues found", entry.display());
    Ok(())
}

pub fn build(path: &str, out_dir: &str) -> Result<()> {
    let root = Path::new(path);
    let config = load_project(root)?;
    let entry = crate::config::entry_file(root, &config);
    let loaded = load_program(&entry);
    print_loaded(&loaded, &loaded.diagnostics);
    if loaded.diagnostics.has_errors() {
        miette::bail!("build failed during load");
    }

    let dist = root.join(out_dir);
    fs::create_dir_all(&dist)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to create output directory {}", dist.display()))?;
    let js_path = dist.join(nasaq_syntax::with_output_ext(&config.package.name));

    let mut cache = CacheStore::open(root);
    if cache.is_fresh(&loaded, &js_path) {
        println!("✓ cache hit — {}", js_path.display());
        return Ok(());
    }

    let output = compile_loaded(
        &loaded,
        &config.package.name,
        &runtime_import(&config),
        web_mount(&config),
        false,
    );
    print_loaded(&loaded, &output.diagnostics);
    if output.diagnostics.has_errors() {
        miette::bail!("build failed");
    }

    fs::write(&js_path, &output.js)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to write {}", js_path.display()))?;
    if let Some(map) = output.source_map {
        let map_path = dist.join(format!("{}.{}.map", config.package.name, nasaq_syntax::OUTPUT));
        fs::write(&map_path, map)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to write {}", map_path.display()))?;
    }
    copy_runtime(&dist)?;
    copy_static_assets(root, &dist)?;
    cache.record(&loaded, &js_path);
    println!("✓ built {}", js_path.display());
    Ok(())
}

pub fn run(path: &str) -> Result<()> {
    let root = Path::new(path);
    let out = config_out_dir(root)?;
    build(path, &out)?;
    let config = load_project(root)?;
    let module_path = output_module(root, &out, &config.package.name);
    let runner = root.join(&out).join("runtime").join(nasaq_syntax::with_runtime_ext("nq-run"));
    let status = Command::new("node")
        .arg(&runner)
        .arg(&module_path)
        .status()
        .into_diagnostic()
        .wrap_err("failed to spawn Node.js — install Node 18+ to run Nasaq programs")?;
    if !status.success() {
        miette::bail!("program exited with status {status}");
    }
    Ok(())
}

pub fn test(path: &str) -> Result<()> {
    let root = Path::new(path);
    let out = config_out_dir(root)?;
    build(path, &out)?;
    let config = load_project(root)?;
    let module_path = output_module(root, &out, &config.package.name);
    let result = nasaq_test_runner::run_project_tests(root, &module_path)
        .map_err(|e| miette::miette!(e))?;
    if result.failed > 0 {
        miette::bail!("{}", result.output);
    }
    println!("✓ {}", result.output);
    Ok(())
}

pub fn publish(path: &str) -> Result<()> {
    let root = Path::new(path);
    let out = config_out_dir(root)?;
    build(path, &out)?;
    let config = load_project(root)?;
    let pkg = serde_json::json!({
        "name": config.package.name,
        "version": config.package.version.clone().unwrap_or_else(|| "0.1.0".into()),
        "type": "module",
        "main": format!("./{}/{}", out, nasaq_syntax::with_output_ext(&config.package.name)),
        "files": [out, ".nasaq"],
        "nasaq": { "format": "nq", "runtime": "nqr" },
        "license": "Apache-2.0 OR MIT"
    });
    let pkg_path = root.join("package.json");
    fs::write(&pkg_path, serde_json::to_string_pretty(&pkg).unwrap())
        .into_diagnostic()
        .wrap_err("failed to write package.json")?;
    println!("✓ generated {}", pkg_path.display());
    Ok(())
}

pub fn playground(path: &str, out_dir: &str) -> Result<()> {
    let root = Path::new(path);
    let dist = root.join(out_dir);
    fs::create_dir_all(&dist).into_diagnostic()?;
    let html_path = dist.join("playground.html");
    fs::write(&html_path, nasaq_playground::playground_page())
        .into_diagnostic()?;
    println!("✓ wrote {}", html_path.display());
    Ok(())
}

pub fn wasm_build(path: &str, out_dir: &str) -> Result<()> {
    let root = Path::new(path);
    let config = load_project(root)?;
    let entry = crate::config::entry_file(root, &config);
    let source = read_source(&entry)?;
    let out = nasaq_codegen_wasm::compile_to_wasm(&source.contents);
    let dist = root.join(out_dir);
    fs::create_dir_all(&dist).into_diagnostic()?;
    fs::write(dist.join(format!("{}.wasm", config.package.name)), out.bytes)
        .into_diagnostic()?;
    fs::write(dist.join(format!("{}.wat", config.package.name)), out.wat)
        .into_diagnostic()?;
    println!("✓ wrote wasm artifacts to {}", dist.display());
    Ok(())
}

pub fn ssr(path: &str, out_dir: &str) -> Result<()> {
    let root = Path::new(path);
    let config = load_project(root)?;
    let out = config_out_dir(root)?;
    build(path, &out)?;

    let entry = crate::config::entry_file(root, &config);
    let loaded = load_program(&entry);
    print_loaded(&loaded, &loaded.diagnostics);
    if loaded.diagnostics.has_errors() {
        miette::bail!("ssr failed during load");
    }
    let component = loaded.program.items.iter().find_map(|item| match &item.node {
        Item::Component(c) if c.exported => Some(c),
        _ => None,
    });
    let Some(component) = component else {
        miette::bail!("no exported component found for SSR");
    };

    let dist = root.join(out_dir);
    fs::create_dir_all(&dist).into_diagnostic()?;

    let web = config.web.as_ref();
    let mount = web.map(|w| w.mount.clone()).unwrap_or_else(|| "#app".into());
    let body = nasaq_ssr::render_component_html(component);
    let page = nasaq_ssr::render_ssr_document(
        &config.package.name,
        &body,
        &config.package.name,
        &nasaq_ssr::SsrOptions {
            mount_selector: mount,
            hydrate: true,
        },
    );
    let html_path = dist.join("ssr.html");
    fs::write(&html_path, page)
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to write {}", html_path.display()))?;
    println!("✓ rendered {}", html_path.display());
    Ok(())
}

pub fn fmt(path: &str) -> Result<()> {
    let root = Path::new(path);
    let config = load_project(root)?;
    let entry = crate::config::entry_file(root, &config);
    let source = read_source(&entry)?;
    let formatted = nasaq_fmt::format_source(&source.contents);
    if formatted != source.contents {
        fs::write(&entry, formatted)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to write {}", entry.display()))?;
        println!("✓ formatted {}", entry.display());
    } else {
        println!("✓ {} already formatted", entry.display());
    }
    Ok(())
}

pub fn lint(path: &str) -> Result<()> {
    let root = Path::new(path);
    let config = load_project(root)?;
    let entry = crate::config::entry_file(root, &config);
    let source = read_source(&entry)?;
    let issues = nasaq_lint::lint_source(&source.contents);
    if issues.is_empty() {
        println!("✓ {} — no lint issues", entry.display());
        return Ok(());
    }
    let count = issues.len();
    for issue in issues {
        eprintln!("lint:{}:{}: {}", entry.display(), issue.line, issue.message);
    }
    miette::bail!("lint failed with {} issue(s)", count);
}

pub fn dev(path: &str, port: u16) -> Result<()> {
    let root = Path::new(path);
    build(path, &config_out_dir(root)?)?;
    let dist = root.join(config_out_dir(root)?);
    playground(path, "dist")?;
    println!("✓ Nasaq dev server at http://127.0.0.1:{port}/");
    println!("  playground → http://127.0.0.1:{port}/playground.html");
    println!("  compile API → POST /api/compile");
    serve_static(&dist, port)
}

pub fn bench() -> Result<()> {
    use std::time::Instant;
    let src = r#"
module bench
export fn fib(n: Int) -> Int {
    if n <= 1 { return n }
    return fib(n - 1) + fib(n - 2)
}
"#;
    let start = Instant::now();
    for _ in 0..100 {
        let _ = nasaq_parser::parse_program(src);
    }
    let parse_ms = start.elapsed().as_millis();
    let load_ms = if Path::new("examples/fibonacci/src/main.nq").exists() {
        let s = Instant::now();
        for _ in 0..20 {
            let _ = load_program(Path::new("examples/fibonacci/src/main.nq"));
        }
        s.elapsed().as_millis()
    } else {
        0
    };
    println!("Nasaq Benchmark");
    println!("  parse (x100): {parse_ms} ms");
    println!("  load  (x20):  {load_ms} ms");
    Ok(())
}

pub fn website(port: u16) -> Result<()> {
    let root = Path::new("website");
    if !root.join("nasaq.toml").exists() {
        miette::bail!("website/nasaq.toml not found — run from repo root");
    }
    println!("✓ موقع نَسَق — ملفات .nq فقط");
    dev("website", port)
}


fn web_mount(config: &crate::config::NasaqConfig) -> Option<(String, String)> {
    config
        .web
        .as_ref()
        .map(|w| (w.component.clone(), w.mount.clone()))
}

fn copy_static_assets(root: &Path, dist: &Path) -> Result<()> {
    for name in ["index.html", "favicon.ico"] {
        let src = root.join(name);
        if src.exists() {
            fs::copy(&src, dist.join(name))
                .into_diagnostic()
                .wrap_err_with(|| format!("failed to copy {}", src.display()))?;
        }
    }
    Ok(())
}

fn serve_static(root: &Path, port: u16) -> Result<()> {
    use std::net::TcpListener;
    let listener = TcpListener::bind(format!("127.0.0.1:{port}"))
        .into_diagnostic()
        .wrap_err_with(|| format!("failed to bind port {port}"))?;
    for mut stream in listener.incoming().flatten() {
        let _ = handle_http(&mut stream, root);
    }
    Ok(())
}

fn handle_http(stream: &mut std::net::TcpStream, root: &Path) -> std::io::Result<()> {
    use std::io::{Read, Write};
    let mut buf = vec![0u8; 8192];
    let n = stream.read(&mut buf)?;
    let req = String::from_utf8_lossy(&buf[..n]);
    let first_line = req.lines().next().unwrap_or_default();
    let method = first_line.split_whitespace().next().unwrap_or("GET");
    let path = first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/");

    if method == "POST" && path == "/api/compile" {
        return handle_compile_api(stream, &req);
    }

    let rel = if path == "/" {
        "index.html".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };
    let file_path = root.join(&rel);
    let (status, content_type, body) = if file_path.starts_with(root) && file_path.is_file() {
        let bytes = fs::read(&file_path).unwrap_or_default();
        let ct = content_type_for(&file_path);
        ("200 OK", ct, bytes)
    } else {
        ("404 Not Found", "text/plain", b"Not Found".to_vec())
    };
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(&body)?;
    Ok(())
}

fn handle_compile_api(stream: &mut std::net::TcpStream, req: &str) -> std::io::Result<()> {
    use std::io::Write;
    let body_start = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
    let body = &req[body_start..];
    let source = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|v| v.get("source").and_then(|s| s.as_str()).map(str::to_string))
        .unwrap_or_default();
    let json = nasaq_playground::compile_snippet_json(&source);
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        json.len()
    );
    stream.write_all(header.as_bytes())?;
    stream.write_all(json.as_bytes())?;
    Ok(())
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("nq") | Some("nqr") | Some("js") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json") => "application/json",
        Some("map") => "application/json",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}

fn runtime_import(config: &crate::config::NasaqConfig) -> String {
    config
        .build
        .as_ref()
        .and_then(|b| b.runtime.clone())
        .unwrap_or_else(|| format!("./runtime/{}", nasaq_syntax::with_runtime_ext("core")))
}

fn output_module(root: &Path, out: &str, name: &str) -> std::path::PathBuf {
    root.join(out).join(nasaq_syntax::with_output_ext(name))
}

fn config_out_dir(root: &Path) -> Result<String> {
    let config = load_project(root)?;
    Ok(config
        .build
        .as_ref()
        .and_then(|b| b.out_dir.clone())
        .unwrap_or_else(|| "dist".to_string()))
}

fn copy_runtime(dist: &Path) -> Result<()> {
    let runtime_dir = dist.join("runtime");
    fs::create_dir_all(&runtime_dir)
        .into_diagnostic()
        .wrap_err("failed to create runtime directory")?;
    fs::write(
        runtime_dir.join(nasaq_syntax::with_runtime_ext("core")),
        nasaq_runtime::RUNTIME_CORE,
    )
    .into_diagnostic()
    .wrap_err("failed to write runtime/core.nqr")?;
    fs::write(
        runtime_dir.join(nasaq_syntax::with_runtime_ext("dom")),
        nasaq_runtime::RUNTIME_DOM,
    )
    .into_diagnostic()
    .wrap_err("failed to write runtime/dom.nqr")?;
    fs::write(
        runtime_dir.join(nasaq_syntax::with_runtime_ext("router")),
        nasaq_runtime::RUNTIME_ROUTER,
    )
    .into_diagnostic()
    .wrap_err("failed to write runtime/router.nqr")?;
    fs::write(
        runtime_dir.join(nasaq_syntax::with_runtime_ext("nq-run")),
        nasaq_runtime::RUNTIME_RUNNER,
    )
    .into_diagnostic()
    .wrap_err("failed to write runtime/nq-run.nqr")?;
    Ok(())
}

fn print_loaded(loaded: &nasaq_loader::LoadedProgram, diagnostics: &nasaq_diagnostics::DiagnosticBag) {
    if !diagnostics.diagnostics.is_empty() {
        eprint!("{}", render_loaded_diagnostics(loaded, diagnostics));
    }
}

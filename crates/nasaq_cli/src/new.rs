//! Project scaffolding — `nasaq new` / `nasaq init`.

use std::fs;
use std::path::Path;

use miette::{IntoDiagnostic, Result, WrapErr};
use nasaq_syntax::{with_output_ext, with_runtime_ext, with_source_ext, SOURCE};

pub fn new_project(name: &str, template: &str) -> Result<()> {
    let root = Path::new(name);
    if root.exists() {
        miette::bail!("directory `{}` already exists", name);
    }
    match template {
        "web" => scaffold_web(root, name),
        "lib" => scaffold_lib(root, name),
        _ => scaffold_app(root, name),
    }
}

pub fn init_project(template: &str) -> Result<()> {
    let root = Path::new(".");
    if root.join("nasaq.toml").exists() {
        miette::bail!("nasaq.toml already exists in current directory");
    }
    let name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "my_app".into());
    match template {
        "web" => scaffold_web(root, &name),
        "lib" => scaffold_lib(root, &name),
        _ => scaffold_app(root, &name),
    }
}

fn scaffold_app(root: &Path, name: &str) -> Result<()> {
    let slug = slug(name);
    write_tree(root, &[
        ("nasaq.toml", &manifest_app(&slug)),
        (&format!("src/{}", with_source_ext("main")), &main_app(&slug)),
        ("README.md", &readme(&slug, "app")),
    ])?;
    println!("✓ Nasaq project `{slug}` — source .{SOURCE} → output .{SOURCE}");
    println!("  cd {name}");
    println!("  nasaq run .");
    Ok(())
}

fn scaffold_web(root: &Path, name: &str) -> Result<()> {
    let slug = slug(name);
    fs::create_dir_all(root.join("src/components")).into_diagnostic()?;
    write_tree(root, &[
        ("nasaq.toml", &manifest_web(&slug)),
        ("index.html", &index_html(&slug)),
        (&format!("src/{}", with_source_ext("main")), &main_web(&slug)),
        (&format!("src/{}", with_source_ext("App")), &app_component(&slug)),
        (
            &format!("src/components/{}", with_source_ext("Counter")),
            &counter_component(),
        ),
        ("README.md", &readme(&slug, "web")),
    ])?;
    println!("✓ Nasaq web `{slug}` — .{SOURCE} only (no .js)");
    println!("  cd {name}");
    println!("  nasaq dev .");
    Ok(())
}

fn scaffold_lib(root: &Path, name: &str) -> Result<()> {
    let slug = slug(name);
    write_tree(root, &[
        ("nasaq.toml", &manifest_lib(&slug)),
        (&format!("src/{}", with_source_ext("lib")), &lib_source(&slug)),
        ("README.md", &readme(&slug, "lib")),
    ])?;
    println!("✓ Nasaq library `{slug}`");
    Ok(())
}

fn write_tree(root: &Path, files: &[(&str, &str)]) -> Result<()> {
    for (rel, contents) in files {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).into_diagnostic()?;
        }
        fs::write(&path, contents)
            .into_diagnostic()
            .wrap_err_with(|| format!("failed to write {}", path.display()))?;
    }
    Ok(())
}

fn slug(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn manifest_app(slug: &str) -> String {
    format!(
        r#"# Nasaq project manifest — نَسَق
[package]
name = "{slug}"
version = "0.1.0"
entry = "src/{main}"

[build]
out_dir = "dist"
"#,
        slug = slug,
        main = with_source_ext("main")
    )
}

fn manifest_web(slug: &str) -> String {
    format!(
        r##"# Nasaq project manifest — نَسَق
[package]
name = "{slug}"
version = "0.1.0"
entry = "src/{main}"

[build]
out_dir = "dist"
runtime = "./runtime/{core}"

[web]
mount = "#app"
component = "App"
"##,
        slug = slug,
        main = with_source_ext("main"),
        core = with_runtime_ext("core")
    )
}

fn manifest_lib(slug: &str) -> String {
    format!(
        r#"# Nasaq library manifest — نَسَق
[package]
name = "{slug}"
version = "0.1.0"
entry = "src/{lib}"

[build]
out_dir = "dist"
"#,
        slug = slug,
        lib = with_source_ext("lib")
    )
}

fn main_app(slug: &str) -> String {
    format!(
        r#"module {slug}

extern fn println(value: String)

export fn main() {{
    println("Hello from Nasaq — .nq")
}}
"#
    )
}

fn main_web(slug: &str) -> String {
    format!(
        r#"module {slug}

import "./App";

export fn main() {{
}}
"#
    )
}

fn app_component(_slug: &str) -> String {
    r#"import "./components/Counter";

export component App() {
    view {
        <main class="app">
            <h1>نَسَق Nasaq</h1>
            <p>ملفات المصدر: .nq — المترجم: .nq — Runtime: .nqr</p>
            <Counter start={0} />
        </main>
    }

    style scoped {
        .app {
            font-family: "Segoe UI", Tahoma, "Noto Sans Arabic", sans-serif;
            padding: 2rem;
            max-width: 40rem;
        }
        h1 { color: rgb(37, 99, 235); }
    }
}
"#
    .into()
}

fn counter_component() -> String {
    r#"export component Counter(start: Int = 0) {
    state count: Int = start

    view {
        <section class="counter">
            <span>{ count }</span>
            <button on:click={ count = count + 1 }>+</button>
        </section>
    }

    style scoped {
        .counter { display: flex; gap: 1rem; align-items: center; }
        button { padding: 0.4rem 0.8rem; cursor: pointer; }
    }
}
"#
    .into()
}

fn lib_source(slug: &str) -> String {
    format!(
        r#"module {slug}

export fn double(n: Int) -> Int {{
    return n + n
}}
"#
    )
}

fn index_html(slug: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ar" dir="rtl">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{slug} — نَسَق</title>
</head>
<body>
  <div id="app"></div>
  <script type="module" src="./{out}"></script>
</body>
</html>
"#,
        slug = slug,
        out = with_output_ext(slug)
    )
}

fn readme(slug: &str, kind: &str) -> String {
    format!(
        r#"# {slug} — نَسَق

| Extension | Use |
|-----------|-----|
| `.nq` | Source + compiled module |
| `.nqr` | Runtime (core, dom, router) |
| `nasaq.toml` | Project manifest |

```bash
nasaq check .
nasaq build .   # → dist/{slug}.nq
nasaq dev .
```

Template: `{kind}`
"#
    )
}

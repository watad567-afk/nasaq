//! Minimal LSP server over stdio (JSON-RPC with Content-Length framing).

use std::io::{self, Read, Write};

use nasaq_diagnostics::Severity;
use nasaq_syntax::SourceFile;
use serde_json::{json, Value};

use crate::analyze::analyze_source;
use crate::completion::{completion_items, prefix_at_position};
use crate::docs;

pub fn run_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();

    loop {
        let msg = match read_message(&mut stdin)? {
            Some(msg) => msg,
            None => break,
        };

        let id = msg.get("id").cloned();
        let method = msg
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or_default();

        match method {
            "initialize" => {
                write_message(
                    &mut stdout,
                    json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": {
                            "capabilities": {
                                "textDocumentSync": { "openClose": true, "change": 1 },
                                "completionProvider": {
                                    "triggerCharacters": [".", "\"", "/"]
                                }
                            },
                            "serverInfo": {
                                "name": "nasaq-lsp",
                                "version": crate::analyze::VERSION
                            }
                        }
                    }),
                )?;
            }
            "initialized" => {}
            "shutdown" => {
                write_message(
                    &mut stdout,
                    json!({ "jsonrpc": "2.0", "id": id, "result": null }),
                )?;
            }
            "exit" => break,
            "textDocument/didOpen" | "textDocument/didChange" | "textDocument/didSave" => {
                if let Some(params) = msg.get("params") {
                    cache_document(params);
                    publish_diagnostics(&mut stdout, params)?;
                }
            }
            "textDocument/completion" => {
                if let Some(params) = msg.get("params") {
                    let result = handle_completion(params);
                    write_message(
                        &mut stdout,
                        json!({ "jsonrpc": "2.0", "id": id, "result": result }),
                    )?;
                }
            }
            _ => {
                if id.is_some() {
                    write_message(
                        &mut stdout,
                        json!({ "jsonrpc": "2.0", "id": id, "result": {} }),
                    )?;
                }
            }
        }
        stdout.flush()?;
    }
    Ok(())
}

fn read_message(reader: &mut impl Read) -> io::Result<Option<Value>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let mut byte = [0u8; 1];
        while reader.read(&mut byte)? > 0 {
            if byte[0] == b'\n' {
                break;
            }
            if byte[0] != b'\r' {
                line.push(byte[0] as char);
            }
        }
        if line.is_empty() {
            break;
        }
        if let Some(len) = line.strip_prefix("Content-Length:") {
            content_length = len.trim().parse().ok();
        }
    }

    let Some(len) = content_length else {
        return Ok(None);
    };

    let mut body = vec![0u8; len];
    reader.read_exact(&mut body)?;
    let msg: Value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    if msg.is_null() {
        Ok(None)
    } else {
        Ok(Some(msg))
    }
}

fn write_message(stdout: &mut impl Write, value: Value) -> io::Result<()> {
    let body = value.to_string();
    write!(stdout, "Content-Length: {}\r\n\r\n{}", body.len(), body)
}

fn cache_document(params: &Value) {
    let uri = params
        .pointer("/textDocument/uri")
        .and_then(|u| u.as_str())
        .unwrap_or("");
    let text = params
        .pointer("/textDocument/text")
        .and_then(|t| t.as_str())
        .or_else(|| {
            params
                .pointer("/contentChanges/0/text")
                .and_then(|t| t.as_str())
        })
        .unwrap_or("");
    if !uri.is_empty() {
        docs::update(uri, text);
    }
}

fn handle_completion(params: &Value) -> Value {
    let uri = params
        .pointer("/textDocument/uri")
        .and_then(|u| u.as_str())
        .unwrap_or("");
    let line = params
        .pointer("/position/line")
        .and_then(|l| l.as_u64())
        .unwrap_or(0) as u32;
    let character = params
        .pointer("/position/character")
        .and_then(|c| c.as_u64())
        .unwrap_or(0) as u32;
    let source = docs::get(uri).unwrap_or_default();
    let prefix = prefix_at_position(&source, line, character);
    let items = completion_items(&source, &prefix);
    json!({ "isIncomplete": false, "items": items })
}

fn publish_diagnostics(stdout: &mut impl Write, params: &Value) -> io::Result<()> {
    let uri = params
        .pointer("/textDocument/uri")
        .and_then(|u| u.as_str())
        .unwrap_or("file:///unknown");
    let text = params
        .pointer("/textDocument/text")
        .and_then(|t| t.as_str())
        .or_else(|| {
            params
                .pointer("/contentChanges/0/text")
                .and_then(|t| t.as_str())
        })
        .unwrap_or("");

    let source = SourceFile::new(uri, text);
    let result = analyze_source(text);
    let diags: Vec<Value> = result
        .diagnostics
        .diagnostics
        .iter()
        .map(|d| {
            let (start_line, start_col) = source.line_col(d.span.start);
            let (end_line, end_col) = source.line_col(d.span.end.max(d.span.start + 1));
            json!({
                "range": {
                    "start": {
                        "line": start_line.saturating_sub(1),
                        "character": start_col.saturating_sub(1)
                    },
                    "end": {
                        "line": end_line.saturating_sub(1),
                        "character": end_col.saturating_sub(1)
                    }
                },
                "severity": match d.severity {
                    Severity::Error => 1,
                    Severity::Warning => 2,
                    Severity::Info => 3,
                    Severity::Hint => 4,
                },
                "source": "nasaq",
                "message": d.message
            })
        })
        .collect();

    write_message(
        stdout,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {
                "uri": uri,
                "diagnostics": diags
            }
        }),
    )
}

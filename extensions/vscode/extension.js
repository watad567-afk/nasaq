const vscode = require("vscode");
const path = require("path");
const { LanguageClient, TransportKind } = require("vscode-languageclient/node");

/** @type {LanguageClient | undefined} */
let client;

function findLspServer(context) {
  const configured = vscode.workspace
    .getConfiguration("nasaq")
    .get("lspPath");
  if (configured && configured.length > 0) {
    return configured;
  }

  const candidates = [
    path.join(context.extensionPath, "..", "..", "target", "release", "nasaq-lsp.exe"),
    path.join(context.extensionPath, "..", "..", "target", "release", "nasaq-lsp"),
    "nasaq-lsp",
  ];

  for (const candidate of candidates) {
    if (candidate === "nasaq-lsp") {
      return candidate;
    }
    try {
      require("fs").accessSync(candidate);
      return candidate;
    } catch {
      // try next candidate
    }
  }
  return "nasaq-lsp";
}

function activate(context) {
  const serverPath = findLspServer(context);
  client = new LanguageClient(
    "nasaqLanguageServer",
    "Nasaq Language Server",
    {
      run: { command: serverPath, transport: TransportKind.stdio },
      debug: { command: serverPath, transport: TransportKind.stdio },
    },
    {
      documentSelector: [{ scheme: "file", language: "nasaq" }],
    }
  );

  context.subscriptions.push(
    client.start(),
    vscode.commands.registerCommand("nasaq.checkFile", async () => {
      const doc = vscode.window.activeTextEditor?.document;
      if (!doc || doc.languageId !== "nasaq") {
        vscode.window.showWarningMessage("Open a .nq file first.");
        return;
      }
      vscode.window.showInformationMessage(
        "Diagnostics update automatically via nasaq-lsp."
      );
    })
  );
}

function deactivate() {
  return client?.stop();
}

module.exports = { activate, deactivate };

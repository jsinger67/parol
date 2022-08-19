/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as vscode from "vscode";
import * as lc from "vscode-languageclient/node";
import { spawnSync } from "child_process";
import { TransportKind } from "vscode-languageclient/node";

let client: lc.LanguageClient;

export interface ParolLsExtensionApi {
  client: lc.LanguageClient;
}

export const log = new (class {
  private enabled = true;
  private readonly output = vscode.window.createOutputChannel(
    "Parol Language Client"
  );

  setEnabled(yes: boolean): void {
    log.enabled = yes;
  }

  // Hint: the type [T, ...T[]] means a non-empty array
  debug(...msg: [unknown, ...unknown[]]): void {
    if (!log.enabled) return;
    log.write("DEBUG", ...msg);
  }

  info(...msg: [unknown, ...unknown[]]): void {
    log.write("INFO", ...msg);
  }

  warn(...msg: [unknown, ...unknown[]]): void {
    debugger;
    log.write("WARN", ...msg);
  }

  error(...msg: [unknown, ...unknown[]]): void {
    debugger;
    log.write("ERROR", ...msg);
    log.output.show(true);
  }

  private write(label: string, ...messageParts: unknown[]): void {
    const message = messageParts.join(" ");
    const dateTime = new Date().toLocaleString();
    log.output.appendLine(`${label} [${dateTime}]: ${message}`);
  }
})();

export type ParolDocument = vscode.TextDocument & { languageId: "parol" };

export function isParolDocument(
  document: vscode.TextDocument
): document is ParolDocument {
  return document.languageId === "parol" && document.uri.scheme === "file";
}

export async function activate(_context: vscode.ExtensionContext) {
  // VS Code doesn't show a notification when an extension fails to activate
  // so we do it ourselves.
  return await tryActivate().catch((err) => {
    void vscode.window.showErrorMessage(
      `Cannot activate parol-ls: ${err.message}`
    );
    throw err;
  });
}

async function tryActivate(): Promise<ParolLsExtensionApi> {
  const serverPath = await bootstrap().catch((err) => {
    let message = "bootstrap error. ";

    message +=
      'See the logs in "OUTPUT > Parol Language Server Client" (should open automatically). ';
    message +=
      'To enable verbose logs use { "parol-ls.trace.extension": true }';

    log.error("Bootstrap error", err);
    throw new Error(message);
  });

  // If the extension is launched in debug mode then the debug server options are used
  // Otherwise the run options are used
  let serverOptions: lc.ServerOptions = {
    run: {
      command: serverPath,
      transport: { kind: TransportKind.socket, port: 7061 },
    },
    debug: {
      command: serverPath,
      transport: { kind: TransportKind.socket, port: 7061 },
    },
  };

  // Options to control the language client
  const clientOptions: lc.LanguageClientOptions = {
    // Register the server for plain text documents
    documentSelector: [{ scheme: "file", language: "parol" }],
    synchronize: {
      // Notify the server about file changes to '.par files contained in the workspace
      fileEvents: vscode.workspace.createFileSystemWatcher("**/.par"),
    },
  };

  // Create the language client and start the client.
  client = new lc.LanguageClient(
    "parolLanguageServer",
    "Parol Language Server",
    serverOptions,
    clientOptions
  );

  // Start the client. This will also launch the server
  client.start();

  return {
    client: client,
  };
}

export function isValidExecutable(path: string): boolean {
  log.debug("Checking availability of a binary at", path);

  const res = spawnSync(path, ["--version"], { encoding: "utf8" });

  const printOutput =
    res.error && (res.error as any).code !== "ENOENT" ? log.warn : log.debug;
  if (Array.isArray(res.output)) {
    for (let line of res.output) {
      if (typeof line === "string" && line.includes("parol-ls")) {
        if (line.startsWith(",")) {
          line = line.substring(1);
        }
        printOutput(line);
      }
    }
  } else {
    printOutput(typeof res.output);
  }

  return res.status === 0;
}

async function bootstrap(): Promise<string> {
  const path = "parol-ls";

  log.info("Using server binary at", path);

  if (!isValidExecutable(path)) {
    throw new Error(`Failed to execute ${path} --version`);
  }

  return path;
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}

// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// JS-side adapter for the AffineScript stdlib/Vscode + stdlib/VscodeLanguageClient
// extern bindings. Vendored from hyperpolymath/affinescript:packages/affine-vscode/mod.js
// because the upstream adapter isn't yet published as an npm package.
//
// Update protocol: when affinescript:packages/affine-vscode/mod.js changes,
// copy the file verbatim and adjust the export style at the bottom from
// ESM to CJS if needed. Keep a short note in the PR linking back to the
// upstream commit so drift is visible.

"use strict";

module.exports = function makeVscodeBindings(vscode, lcModule, hostShim) {
  const reg = (obj) => hostShim._registerHandle(obj);
  const get = (h) => hostShim._getHandle(h);
  const getInstance = () => hostShim._instance;

  function readString(ptr) {
    const inst = getInstance();
    if (!inst || !inst.exports.memory) return "";
    const dv = new DataView(inst.exports.memory.buffer);
    const len = dv.getUint32(ptr, true);
    const bytes = new Uint8Array(inst.exports.memory.buffer, ptr + 4, len);
    return new TextDecoder("utf-8").decode(bytes);
  }

  function wrapHandler(idx) {
    return () => {
      const inst = getInstance();
      const tbl = inst && inst.exports && inst.exports.__indirect_function_table;
      if (!tbl) return;
      const fn = tbl.get(idx);
      if (typeof fn === "function") fn();
    };
  }

  const Vscode = {
    registerCommand: (namePtr, handlerIdx) => {
      const name = readString(namePtr);
      const handler = wrapHandler(handlerIdx);
      return reg(vscode.commands.registerCommand(name, handler));
    },

    getConfiguration: (sectionPtr) =>
      reg(vscode.workspace.getConfiguration(readString(sectionPtr))),

    workspaceConfigGetString: (cfgHandle, keyPtr, defPtr) => {
      const cfg = get(cfgHandle);
      const result = cfg.get(readString(keyPtr), readString(defPtr));
      return reg(String(result));
    },

    createFileSystemWatcher: (globPtr) =>
      reg(vscode.workspace.createFileSystemWatcher(readString(globPtr))),

    activeTextEditor: () => {
      const ed = vscode.window.activeTextEditor;
      return ed ? reg(ed) : 0;
    },
    showErrorMessage:       (msgPtr) => reg(vscode.window.showErrorMessage(readString(msgPtr))),
    showWarningMessage:     (msgPtr) => reg(vscode.window.showWarningMessage(readString(msgPtr))),
    showInformationMessage: (msgPtr) => reg(vscode.window.showInformationMessage(readString(msgPtr))),

    createTerminal: (namePtr) =>
      reg(vscode.window.createTerminal(readString(namePtr))),
    terminalShow: (tHandle) => { const t = get(tHandle); if (t) t.show(); return 0; },
    terminalSendText: (tHandle, textPtr) => {
      const t = get(tHandle); if (t) t.sendText(readString(textPtr)); return 0;
    },

    pushSubscription: (ctxHandle, dHandle) => {
      const ctx = get(ctxHandle);
      const d = get(dHandle);
      if (ctx && d) ctx.subscriptions.push(d);
      return 0;
    },

    editorActiveFilePath: () => {
      const ed = vscode.window.activeTextEditor;
      return ed ? reg(ed.document.uri.fsPath) : reg("");
    },
    editorActiveLanguageId: () => {
      const ed = vscode.window.activeTextEditor;
      return ed ? reg(ed.document.languageId) : reg("");
    },

    workspaceConfigGetBool: (cfgHandle, keyPtr, defVal) => {
      const cfg = get(cfgHandle);
      if (!cfg) return defVal;
      return cfg.get(readString(keyPtr), defVal !== 0) ? 1 : 0;
    },

    consoleLog: (msgPtr) => { console.log(readString(msgPtr)); return 0; },
    execSync: (cmdPtr) => {
      try {
        require("child_process").execSync(readString(cmdPtr), { stdio: "ignore" });
        return 0;
      } catch (e) {
        return e.status ?? 1;
      }
    },

    stringConcat: (aPtr, bPtr) => reg(readString(aPtr) + readString(bPtr)),
    stringEndsWith: (sPtr, suffixPtr) =>
      readString(sPtr).endsWith(readString(suffixPtr)) ? 1 : 0,
    stringReplaceSuffix: (sPtr, suffixPtr, replacementPtr) => {
      const s = readString(sPtr);
      const suffix = readString(suffixPtr);
      const replacement = readString(replacementPtr);
      const out = s.endsWith(suffix) ? s.slice(0, -suffix.length) + replacement : s;
      return reg(out);
    },
  };

  const VscodeLanguageClient = {
    newLanguageClient: (idPtr, namePtr, cmdPtr, argsNlPtr, transportKind) => {
      const id = readString(idPtr);
      const name = readString(namePtr);
      const command = readString(cmdPtr);
      const args = readString(argsNlPtr).split("\n").filter(s => s.length > 0);
      const transport = transportKind === 1 ? lcModule.TransportKind.ipc : lcModule.TransportKind.stdio;
      const serverOptions = { run: { command, args, transport }, debug: { command, args, transport } };
      const clientOptions = { documentSelector: [{ scheme: "file" }] };
      return reg(new lcModule.LanguageClient(id, name, serverOptions, clientOptions));
    },
    languageClientStart: (cHandle) => {
      const c = get(cHandle);
      if (c) c.start();
      return 0;
    },
    languageClientStop: (cHandle) => {
      const c = get(cHandle);
      if (c) c.stop();
      return 0;
    },
  };

  return { Vscode, VscodeLanguageClient };
};

// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
// Runtime entry point — what VS Code loads via package.json `main`.
//
// Pipeline:
//   src/extension.affine   ──affinescript compile──>   out/extension.cjs   (wasm shim)
//   src/index.cjs          ──this file──>   exports.{activate,deactivate}
//
// This wrapper bridges the compiled wasm shim to the live vscode/lc modules
// by installing `extraImports` on the shim before activation runs.

"use strict";

const shim = require("../out/extension.cjs");
const makeVscodeBindings = require("./affine-vscode-adapter.cjs");

// Install the Phase-2 binding hook. `_buildImports()` inside the shim calls
// this after the host invokes `exports.activate`, just before
// `WebAssembly.instantiate`. The shim's own `exports._instance` is set
// during init, so the adapter's lazy `getInstance()` resolves correctly
// for callbacks that fire after activation.
shim.extraImports = function extraImports() {
  return makeVscodeBindings(
    require("vscode"),
    require("vscode-languageclient/node"),
    shim
  );
};

exports.activate = shim.activate;
exports.deactivate = shim.deactivate;

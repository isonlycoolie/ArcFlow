"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const serverClient_1 = require("./client/serverClient");
const webviewPanel_1 = require("./graph/webviewPanel");
const traceTimelinePanel_1 = require("./trace/traceTimelinePanel");
function activate(context) {
    context.subscriptions.push(vscode.commands.registerCommand("arcflow.visualizeGraph", () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || !(0, webviewPanel_1.isArcflowWorkflowDocument)(editor.document)) {
            void vscode.window.showWarningMessage("Open an *.arcflow.json workflow file to visualize the graph.");
            return;
        }
        (0, webviewPanel_1.openGraphPanel)(context, editor.document);
    }), vscode.commands.registerCommand("arcflow.viewTraceTimeline", () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || !(0, traceTimelinePanel_1.isArcflowTraceDocument)(editor.document)) {
            void vscode.window.showWarningMessage("Open an *.arcflow.trace.json file to view the trace timeline.");
            return;
        }
        (0, traceTimelinePanel_1.openTraceTimelinePanel)(context, editor.document);
    }), vscode.commands.registerCommand("arcflow.connectServer", () => {
        void (0, serverClient_1.connectToLocalServer)();
    }), vscode.workspace.onDidOpenTextDocument((document) => {
        const config = vscode.workspace.getConfiguration("arcflow");
        if ((0, webviewPanel_1.isArcflowWorkflowDocument)(document) && config.get("autoOpenGraph", true)) {
            (0, webviewPanel_1.openGraphPanel)(context, document);
        }
        if ((0, traceTimelinePanel_1.isArcflowTraceDocument)(document) && config.get("autoOpenTrace", true)) {
            (0, traceTimelinePanel_1.openTraceTimelinePanel)(context, document);
        }
    }));
}
function deactivate() {
    // Panels dispose themselves via onDidDispose handlers.
}
//# sourceMappingURL=extension.js.map
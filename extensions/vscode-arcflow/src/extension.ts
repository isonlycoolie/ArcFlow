import * as vscode from "vscode";
import { connectToLocalServer } from "./client/serverClient";
import {
  isArcflowWorkflowDocument,
  openGraphPanel,
} from "./graph/webviewPanel";
import {
  isArcflowTraceDocument,
  openTraceTimelinePanel,
} from "./trace/traceTimelinePanel";

export function activate(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    vscode.commands.registerCommand("arcflow.visualizeGraph", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || !isArcflowWorkflowDocument(editor.document)) {
        void vscode.window.showWarningMessage(
          "Open an *.arcflow.json workflow file to visualize the graph.",
        );
        return;
      }
      openGraphPanel(context, editor.document);
    }),

    vscode.commands.registerCommand("arcflow.viewTraceTimeline", () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || !isArcflowTraceDocument(editor.document)) {
        void vscode.window.showWarningMessage(
          "Open an *.arcflow.trace.json file to view the trace timeline.",
        );
        return;
      }
      openTraceTimelinePanel(context, editor.document);
    }),

    vscode.commands.registerCommand("arcflow.connectServer", () => {
      void connectToLocalServer();
    }),

    vscode.workspace.onDidOpenTextDocument((document) => {
      const config = vscode.workspace.getConfiguration("arcflow");
      if (isArcflowWorkflowDocument(document) && config.get("autoOpenGraph", true)) {
        openGraphPanel(context, document);
      }
      if (isArcflowTraceDocument(document) && config.get("autoOpenTrace", true)) {
        openTraceTimelinePanel(context, document);
      }
    }),
  );
}

export function deactivate(): void {
  // Panels dispose themselves via onDidDispose handlers.
}

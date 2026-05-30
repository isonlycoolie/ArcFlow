import * as vscode from "vscode";
import { connectToLocalServer, ServerClient } from "./client/serverClient";
import {
  BreakpointManager,
  DebugAdapter,
  openDebugStatePanel,
} from "./debug/debugAdapter";
import {
  isArcflowWorkflowDocument,
  openGraphPanel,
} from "./graph/webviewPanel";
import {
  isArcflowTraceDocument,
  openTraceTimelinePanel,
} from "./trace/traceTimelinePanel";

const breakpoints = new BreakpointManager();

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

    vscode.commands.registerCommand("arcflow.toggleBreakpoint", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || !isArcflowWorkflowDocument(editor.document)) {
        void vscode.window.showWarningMessage(
          "Open an *.arcflow.json workflow file to set breakpoints.",
        );
        return;
      }
      const stepId = await vscode.window.showInputBox({
        prompt: "Step or node id to break on",
        placeHolder: "step-uuid",
      });
      if (!stepId) {
        return;
      }
      const enabled = breakpoints.toggle(stepId);
      void vscode.window.showInformationMessage(
        enabled
          ? `ArcFlow: breakpoint set on ${stepId}`
          : `ArcFlow: breakpoint removed from ${stepId}`,
      );
    }),

    vscode.commands.registerCommand("arcflow.startDebugRun", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || !isArcflowWorkflowDocument(editor.document)) {
        void vscode.window.showWarningMessage(
          "Open an *.arcflow.json workflow file to start a debug run.",
        );
        return;
      }
      const workflow = JSON.parse(editor.document.getText()) as {
        agents?: unknown[];
      };
      const adapter = new DebugAdapter(ServerClient.fromConfig());
      const runId = await adapter.startRun({
        workflow: JSON.parse(editor.document.getText()),
        agents: workflow.agents ?? [],
        input: "debug",
        breakpoints: breakpoints.list(),
      });
      if (!runId) {
        void vscode.window.showErrorMessage(
          "ArcFlow: debug run failed. Ensure ARCFLOW_DEBUG=true on the server.",
        );
        return;
      }
      void vscode.window.showInformationMessage(`ArcFlow: debug run ${runId} started`);
      const state = await adapter.fetchState(runId);
      if (state) {
        await openDebugStatePanel(context, state);
      }
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
  breakpoints.clear();
}

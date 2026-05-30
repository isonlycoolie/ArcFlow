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
exports.sidecarPath = sidecarPath;
exports.loadLayoutSidecar = loadLayoutSidecar;
exports.saveLayoutSidecar = saveLayoutSidecar;
exports.mergeSidecarPositions = mergeSidecarPositions;
exports.layoutToSidecar = layoutToSidecar;
exports.resolveWorkflowPath = resolveWorkflowPath;
const fs = __importStar(require("fs"));
const path = __importStar(require("path"));
function sidecarPath(workflowFile) {
    return workflowFile.replace(/\.arcflow\.json$/i, ".arcflow.layout.json");
}
function loadLayoutSidecar(workflowFile) {
    const file = sidecarPath(workflowFile);
    if (!fs.existsSync(file)) {
        return undefined;
    }
    try {
        return JSON.parse(fs.readFileSync(file, "utf8"));
    }
    catch {
        return undefined;
    }
}
function saveLayoutSidecar(workflowFile, sidecar) {
    const file = sidecarPath(workflowFile);
    fs.writeFileSync(file, JSON.stringify(sidecar, null, 2), "utf8");
}
function mergeSidecarPositions(layout, sidecar) {
    if (!sidecar?.positions) {
        return layout;
    }
    const nodes = layout.nodes.map((node) => {
        const saved = sidecar.positions[node.id];
        if (!saved) {
            return node;
        }
        return { ...node, x: saved.x, y: saved.y };
    });
    return { ...layout, nodes };
}
function layoutToSidecar(layout) {
    const positions = {};
    for (const node of layout.nodes) {
        positions[node.id] = { x: node.x, y: node.y };
    }
    return { workflowName: layout.workflowName, positions };
}
function resolveWorkflowPath(document) {
    return path.normalize(document.uri.fsPath);
}
//# sourceMappingURL=layoutSidecar.js.map
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
exports.ServerClient = void 0;
exports.connectToLocalServer = connectToLocalServer;
const vscode = __importStar(require("vscode"));
const DEFAULT_URL = "http://127.0.0.1:8080";
/** Local-only HTTP client stub for arcflow-server (Month 5 preview). */
class ServerClient {
    baseUrl;
    constructor(baseUrl) {
        this.baseUrl = baseUrl ?? DEFAULT_URL;
    }
    static fromConfig() {
        const config = vscode.workspace.getConfiguration("arcflow");
        const url = config.get("serverUrl", DEFAULT_URL);
        return new ServerClient(url);
    }
    getBaseUrl() {
        return this.baseUrl;
    }
    /** Validates localhost binding before any request. */
    assertLocalhost() {
        let parsed;
        try {
            parsed = new URL(this.baseUrl);
        }
        catch {
            throw new Error(`Invalid server URL: ${this.baseUrl}`);
        }
        const host = parsed.hostname;
        if (host !== "127.0.0.1" && host !== "localhost" && host !== "::1") {
            throw new Error("ArcFlow debug endpoints must use localhost (127.0.0.1). Remote URLs are not permitted.");
        }
    }
    /** Ping health endpoint; returns false when server is unreachable. */
    async ping() {
        this.assertLocalhost();
        try {
            const response = await fetch(`${this.baseUrl}/health`, {
                method: "GET",
                signal: AbortSignal.timeout(3000),
            });
            return response.ok;
        }
        catch {
            return false;
        }
    }
    /** Fetch execution trace for a run id (stub — wired in stable release). */
    async fetchRunTrace(runId) {
        this.assertLocalhost();
        try {
            const response = await fetch(`${this.baseUrl}/v1/runs/${runId}/trace`, {
                method: "GET",
                signal: AbortSignal.timeout(5000),
            });
            if (!response.ok) {
                return undefined;
            }
            return response.json();
        }
        catch {

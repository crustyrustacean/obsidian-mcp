/**
 * Obsidian MCP extension for pi
 *
 * Spawns the obsidian-mcp binary in stdio mode and registers all 16 tools.
 * The binary is expected to be on PATH or at OBSIDIAN_MCP_BINARY.
 */

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { defineTool } from "@mariozechner/pi-coding-agent";
import { Type, StringEnum } from "@mariozechner/pi-ai";
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";

let mcpProcess: ReturnType<typeof spawn> | null = null;
let requestId = 0;
const pending = new Map<number, { resolve: (value: any) => void; reject: (err: Error) => void }>();

/** Send a JSON-RPC request and return the response. */
function mcpCall(method: string, params?: Record<string, unknown>): Promise<any> {
    return new Promise((resolve, reject) => {
        if (!mcpProcess || !mcpProcess.stdin) {
            reject(new Error("obsidian-mcp process not running"));
            return;
        }

        const id = ++requestId;
        const msg = JSON.stringify({ jsonrpc: "2.0", id, method, params: params ?? null }) + "\n";

        pending.set(id, { resolve, reject });
        mcpProcess.stdin.write(msg);

        setTimeout(() => {
            if (pending.delete(id)) {
                reject(new Error(`MCP call timed out: ${method}`));
            }
        }, 30_000);
    });
}

/** Call an Obsidian tool by name with arguments. */
async function callTool(name: string, args: Record<string, unknown>): Promise<string> {
    const resp = await mcpCall("tools/call", { name, arguments: args });
    if (resp.error) {
        throw new Error(`obsidian_${name}: ${resp.error.message}`);
    }
    const content = resp.result?.content;
    if (Array.isArray(content)) {
        return content.map((c: any) => c.text ?? "").join("\n");
    }
    return JSON.stringify(resp.result);
}

export default function (pi: ExtensionAPI) {
    /** Spawn the obsidian-mcp binary in stdio mode. */
    function startMcp() {
        const bin = process.env.OBSIDIAN_MCP_BINARY || "obsidian-mcp";

        mcpProcess = spawn(bin, ["--transport", "stdio"], {
            env: {
                ...process.env,
                RUST_LOG: process.env.RUST_LOG ?? "info",
            },
            stdio: ["pipe", "pipe", "pipe"],
        });

        const rl = createInterface({ input: mcpProcess.stdout! });

        rl.on("line", (line: string) => {
            try {
                const msg = JSON.parse(line);
                if (msg.id != null && pending.has(msg.id)) {
                    const { resolve, reject } = pending.get(msg.id)!;
                    pending.delete(msg.id);
                    if (msg.error) {
                        reject(new Error(msg.error.message || "MCP error"));
                    } else {
                        resolve(msg);
                    }
                }
            } catch {
                // Ignore non-JSON lines
            }
        });

        mcpProcess.stderr!.on("data", (data: Buffer) => {
            const text = data.toString().trim();
            if (text && (text.includes("WARN") || text.includes("ERROR"))) {
                pi.exec("echo", [text], { timeout: 1000 }).catch(() => {});
            }
        });

        mcpProcess.on("exit", (code) => {
            if (code !== 0 && code !== null) {
                pi.sendUserMessage?.(`[obsidian-mcp] process exited with code ${code}`, { deliverAs: "followUp" });
            }
            mcpProcess = null;
        });

        // Send initialize handshake
        mcpCall("initialize").then(() => {
            if (mcpProcess?.stdin) {
                mcpProcess.stdin.write(JSON.stringify({
                    jsonrpc: "2.0",
                    method: "notifications/initialized",
                }) + "\n");
            }
        }).catch((err) => {
            pi.sendUserMessage?.(`[obsidian-mcp] initialization failed: ${err.message}`, { deliverAs: "followUp" });
        });
    }

    pi.on("session_start", async (_event, ctx) => {
        try {
            startMcp();
            ctx.ui.notify("Obsidian MCP connected", "info");
        } catch (err: any) {
            ctx.ui.notify(`Obsidian MCP failed: ${err.message}`, "error");
        }
    });

    pi.on("session_shutdown", async () => {
        if (mcpProcess) {
            mcpProcess.kill();
            mcpProcess = null;
        }
    });

    // Register all 16 tools using defineTool.
    // NOTE: callTool names MUST match the MCP server's tool names exactly
    // (i.e., with the "obsidian_" prefix) — the server registers tools as
    // "obsidian_read_note", not "read_note".
    const tools = [
        defineTool({
            name: "obsidian_read_note",
            label: "Read note",
            description: "Read a note from the Obsidian vault as markdown",
            parameters: Type.Object({ path: Type.String({ description: "Path to the note (e.g., 'notes/my-note.md')" }) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_read_note", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_read_note_metadata",
            label: "Read note metadata",
            description: "Read a note's metadata (frontmatter) as JSON",
            parameters: Type.Object({ path: Type.String({ description: "Path to the note" }) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_read_note_metadata", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_write_note",
            label: "Write note",
            description: "Write (create or replace) a note in the Obsidian vault. Write is verified by read-back.",
            parameters: Type.Object({
                path: Type.String({ description: "Path to the note" }),
                content: Type.String({ description: "Markdown content to write" }),
            }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_write_note", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_append_note",
            label: "Append note",
            description: "Append content to an existing note in the Obsidian vault",
            parameters: Type.Object({
                path: Type.String({ description: "Path to the note" }),
                content: Type.String({ description: "Markdown content to append" }),
            }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_append_note", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_patch_note",
            label: "Patch note",
            description: "Patch (update) a specific heading within a note",
            parameters: Type.Object({
                path: Type.String({ description: "Path to the note" }),
                heading: Type.String({ description: "Heading to target (e.g., '## Section')" }),
                content: Type.String({ description: "Content to place under the heading" }),
            }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_patch_note", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_delete_note",
            label: "Delete note",
            description: "Delete a note from the Obsidian vault. Requires confirm=true.",
            parameters: Type.Object({
                path: Type.String({ description: "Path to the note" }),
                confirm: Type.Boolean({ description: "Must be true to confirm deletion" }),
            }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_delete_note", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_search",
            label: "Search vault",
            description: "Full-text search across the Obsidian vault",
            parameters: Type.Object({ query: Type.String({ description: "Search query (max 1000 chars)" }) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_search", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_dataview_query",
            label: "Dataview query",
            description: "Execute a Dataview DQL query against the Obsidian vault",
            parameters: Type.Object({ dql: Type.String({ description: "Dataview DQL query string (max 1000 chars)" }) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_dataview_query", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_jsonlogic_query",
            label: "JSONLogic query",
            description: "Execute a JSONLogic search query against the Obsidian vault",
            parameters: Type.Object({ logic: Type.Object({}, { description: "JSONLogic query object" }) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_jsonlogic_query", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_batch_read",
            label: "Batch read",
            description: "Read multiple notes in parallel. Max 20 paths. Partial failures reported per-path.",
            parameters: Type.Object({ paths: Type.Array(Type.String(), { description: "Array of note paths (max 20)" }) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_batch_read", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_periodic_note",
            label: "Periodic note",
            description: "Get a periodic note (daily, weekly, or monthly)",
            parameters: Type.Object({ period: StringEnum(["daily", "weekly", "monthly"] as const, { description: "Period type" }) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_periodic_note", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_recent_changes",
            label: "Recent changes",
            description: "Get recently changed notes sorted by modification time",
            parameters: Type.Object({ limit: Type.Optional(Type.Number({ description: "Max results (1-100, default 10)" })) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_recent_changes", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_list_directory",
            label: "List directory",
            description: "List the contents of a directory in the Obsidian vault",
            parameters: Type.Object({ path: Type.Optional(Type.String({ description: "Directory path (empty for root)" })) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_list_directory", params) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_get_tags",
            label: "Get tags",
            description: "Get the tag hierarchy with counts from the Obsidian vault",
            parameters: Type.Object({}),
            async execute(_id, _params) {
                return { content: [{ type: "text", text: await callTool("obsidian_get_tags", {}) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_server_status",
            label: "Server status",
            description: "Health check — test connectivity to the Obsidian Local REST API",
            parameters: Type.Object({}),
            async execute(_id, _params) {
                return { content: [{ type: "text", text: await callTool("obsidian_server_status", {}) }], details: {} };
            },
        }),
        defineTool({
            name: "obsidian_open_note",
            label: "Open note",
            description: "Trigger the Obsidian UI to open a specific note",
            parameters: Type.Object({ path: Type.String({ description: "Path to the note to open" }) }),
            async execute(_id, params) {
                return { content: [{ type: "text", text: await callTool("obsidian_open_note", params) }], details: {} };
            },
        }),
    ];

    for (const tool of tools) {
        pi.registerTool(tool);
    }
}

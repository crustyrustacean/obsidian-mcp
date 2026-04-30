/**
 * Obsidian MCP extension for pi
 *
 * Spawns the obsidian-mcp binary in stdio mode and registers all 16 tools.
 * The binary is expected to be on PATH or at OBSIDIAN_MCP_BINARY.
 *
 * Setup:
 *   1. Build: cargo build --release
 *   2. Copy target/release/obsidian-mcp to somewhere on PATH
 *   3. Set OBSIDIAN_API_KEY in your environment
 *   4. Start pi — the extension auto-loads
 */

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { Type } from "typebox";
import { ChildProcess, spawn } from "node:child_process";
import { createInterface } from "node:readline";

let mcpProcess: ChildProcess | null = null;
let requestId = 0;
const pending = new Map<number, { resolve: (value: any) => void; reject: (err: Error) => void }>();

/** Send a JSON-RPC request and return the response result. */
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

        // Timeout after 30s
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
    // MCP tool results are in content[].text
    const content = resp.result?.content;
    if (Array.isArray(content)) {
        return content.map((c: any) => c.text ?? "").join("\n");
    }
    return JSON.stringify(resp.result);
}

/** Tool definitions — mirrors the 16 tools from obsidian-mcp. */
const toolDefs = [
    {
        name: "obsidian_read_note",
        description: "Read a note from the Obsidian vault as markdown",
        schema: Type.Object({ path: Type.String({ description: "Path to the note (e.g., 'notes/my-note.md')" }) }),
        call: (args: any) => callTool("read_note", args),
    },
    {
        name: "obsidian_read_note_metadata",
        description: "Read a note's metadata (frontmatter) as JSON",
        schema: Type.Object({ path: Type.String({ description: "Path to the note" }) }),
        call: (args: any) => callTool("read_note_metadata", args),
    },
    {
        name: "obsidian_write_note",
        description: "Write (create or replace) a note in the Obsidian vault. Write is verified by read-back.",
        schema: Type.Object({
            path: Type.String({ description: "Path to the note" }),
            content: Type.String({ description: "Markdown content to write" }),
        }),
        call: (args: any) => callTool("write_note", args),
    },
    {
        name: "obsidian_append_note",
        description: "Append content to an existing note in the Obsidian vault",
        schema: Type.Object({
            path: Type.String({ description: "Path to the note" }),
            content: Type.String({ description: "Markdown content to append" }),
        }),
        call: (args: any) => callTool("append_note", args),
    },
    {
        name: "obsidian_patch_note",
        description: "Patch (update) a specific heading within a note",
        schema: Type.Object({
            path: Type.String({ description: "Path to the note" }),
            heading: Type.String({ description: "Heading to target (e.g., '## Section')" }),
            content: Type.String({ description: "Content to place under the heading" }),
        }),
        call: (args: any) => callTool("patch_note", args),
    },
    {
        name: "obsidian_delete_note",
        description: "Delete a note from the Obsidian vault. Requires confirm=true.",
        schema: Type.Object({
            path: Type.String({ description: "Path to the note" }),
            confirm: Type.Boolean({ description: "Must be true to confirm deletion" }),
        }),
        call: (args: any) => callTool("delete_note", args),
    },
    {
        name: "obsidian_search",
        description: "Full-text search across the Obsidian vault",
        schema: Type.Object({ query: Type.String({ description: "Search query (max 1000 chars)" }) }),
        call: (args: any) => callTool("search", args),
    },
    {
        name: "obsidian_dataview_query",
        description: "Execute a Dataview DQL query against the Obsidian vault",
        schema: Type.Object({ dql: Type.String({ description: "Dataview DQL query string (max 1000 chars)" }) }),
        call: (args: any) => callTool("dataview_query", args),
    },
    {
        name: "obsidian_jsonlogic_query",
        description: "Execute a JSONLogic search query against the Obsidian vault",
        schema: Type.Object({ logic: Type.Object({}, { description: "JSONLogic query object" }) }),
        call: (args: any) => callTool("jsonlogic_query", args),
    },
    {
        name: "obsidian_batch_read",
        description: "Read multiple notes in parallel. Max 20 paths. Partial failures reported per-path.",
        schema: Type.Object({ paths: Type.Array(Type.String(), { description: "Array of note paths (max 20)" }) }),
        call: (args: any) => callTool("batch_read", args),
    },
    {
        name: "obsidian_periodic_note",
        description: "Get a periodic note (daily, weekly, or monthly)",
        schema: Type.Object({ period: Type.Union([Type.Literal("daily"), Type.Literal("weekly"), Type.Literal("monthly")], { description: "Period type" }) }),
        call: (args: any) => callTool("periodic_note", args),
    },
    {
        name: "obsidian_recent_changes",
        description: "Get recently changed notes sorted by modification time",
        schema: Type.Object({ limit: Type.Optional(Type.Number({ description: "Max results (1-100, default 10)" })) }),
        call: (args: any) => callTool("recent_changes", args),
    },
    {
        name: "obsidian_list_directory",
        description: "List the contents of a directory in the Obsidian vault",
        schema: Type.Object({ path: Type.Optional(Type.String({ description: "Directory path (empty for root)" })) }),
        call: (args: any) => callTool("list_directory", args),
    },
    {
        name: "obsidian_get_tags",
        description: "Get the tag hierarchy with counts from the Obsidian vault",
        schema: Type.Object({}),
        call: (args: any) => callTool("get_tags", args),
    },
    {
        name: "obsidian_server_status",
        description: "Health check — test connectivity to the Obsidian Local REST API",
        schema: Type.Object({}),
        call: (args: any) => callTool("server_status", args),
    },
    {
        name: "obsidian_open_note",
        description: "Trigger the Obsidian UI to open a specific note",
        schema: Type.Object({ path: Type.String({ description: "Path to the note to open" }) }),
        call: (args: any) => callTool("open_note", args),
    },
];

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
                // Ignore non-JSON lines (shouldn't happen with RUST_LOG to stderr)
            }
        });

        mcpProcess.stderr!.on("data", (data: Buffer) => {
            // Log obsidian-mcp stderr to pi's notification area
            const text = data.toString().trim();
            if (text) {
                // Only log warnings and errors, not every INFO line
                if (text.includes("WARN") || text.includes("ERROR")) {
                    pi.exec("echo", [text], { timeout: 1000 }).catch(() => {});
                }
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
            // Send initialized notification (no id = notification)
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

    // Start on session start
    pi.on("session_start", async (_event, ctx) => {
        try {
            startMcp();
            ctx.ui.notify("Obsidian MCP connected", "info");
        } catch (err: any) {
            ctx.ui.notify(`Obsidian MCP failed: ${err.message}`, "error");
        }
    });

    // Clean up on shutdown
    pi.on("session_shutdown", async () => {
        if (mcpProcess) {
            mcpProcess.kill();
            mcpProcess = null;
        }
    });

    // Register all 16 tools
    for (const def of toolDefs) {
        pi.registerTool({
            name: def.name,
            label: def.name.replace("obsidian_", "").replace(/_/g, " "),
            description: def.description,
            parameters: def.schema,
            async execute(_toolCallId, params, _signal, _onUpdate, _ctx) {
                const result = await def.call(params);
                return {
                    content: [{ type: "text", text: result }],
                    details: {},
                };
            },
        });
    }
}

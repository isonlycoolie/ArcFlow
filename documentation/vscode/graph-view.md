
# Graph view (VS Code)

Visualize graph-mode workflow definitions as a DAG inside VS Code. Helps catch missing edges, orphan nodes, and join misconfiguration before runs hit the server.

## Open graph view

1. Save workflow as `something.arcflow.json` (language id `arcflow-workflow`).
2. Ensure `execution_mode` is `"graph"` with `nodes` and `edges` arrays per the workflow specification.
3. Run command palette: **ArcFlow: Visualize Graph** (`arcflow.visualizeGraph`).
4. Or click the graph icon in the editor title when the file matches `*.arcflow.json`.

## Example workflow fragment

```json
{
 "id": "00000000-0000-4000-8000-000000000099",
 "name": "support_router",
 "execution_mode": "graph",
 "entry_node": "00000000-0000-4000-8000-000000000010",
 "nodes": [
 {
 "id": "00000000-0000-4000-8000-000000000010",
 "agent_id": "00000000-0000-4000-8000-000000000020",
 "kind": "agent"
 },
 {
 "id": "00000000-0000-4000-8000-000000000011",
 "agent_id": "00000000-0000-4000-8000-000000000021",
 "kind": "agent"
 }
 ],
 "edges": [
 {
 "from": "00000000-0000-4000-8000-000000000010",
 "to": "00000000-0000-4000-8000-000000000011",
 "condition": null
 }
 ],
 "max_iterations": 50
}
```

## ASCII preview (documentation diagram)

Graph view renders interactively in the extension webview. Logical layout resembles:

```text
 [ classify ]
 |
 +-------+-------+
 | |
 [ billing ] [ technical ]
 | |
 +-------+-------+
 |
 [ merge ]
```

Nodes show agent names when resolvable from bundled `agents` array in the same file or workspace context.

## Navigation tips

| Action | Result |
|--------|--------|
| Click node | Highlight definition in JSON source |
| Conditional edge | Label shows condition expression id |
| `to: null` edge | Branch termination (supported by runtime) |
| Join node | Multiple inbound edges converge |

## Graph recovery note (Graph recovery resume)

Graph **execution** is production-ready. **Resume from checkpoint after failure** is partial. Use graph view for authoring and testing happy path; plan linear recovery patterns for critical SLAs until Graph recovery resume closes.

## Related authoring docs

- [guides/workflows/graph-workflows.md](../guides/workflows/graph-workflows.md)
- [guides/workflows/workflow-registry.md](../guides/workflows/workflow-registry.md)

## Related pages

- [vscode/overview.md](overview.md)
- [trace-timeline.md](trace-timeline.md)

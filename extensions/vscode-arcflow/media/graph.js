(function () {
  const NODE_W = 140;
  const NODE_H = 48;

  const svg = document.getElementById("graph-canvas");
  const nameEl = document.getElementById("workflow-name");
  const badgeEl = document.getElementById("mode-badge");
  const warningsEl = document.getElementById("warnings");

  function nodeCenter(layout, id) {
    const node = layout.nodes.find(function (n) { return n.id === id; });
    if (!node) return { x: 0, y: 0 };
    return { x: node.x + NODE_W / 2, y: node.y + NODE_H / 2 };
  }

  function render(payload) {
    const layout = payload.layout;
    const dims = payload.dims;

    nameEl.textContent = layout.workflowName || "Workflow";
    badgeEl.textContent = layout.mode;

    warningsEl.innerHTML = "";
    (layout.warnings || []).forEach(function (msg) {
      const p = document.createElement("p");
      p.className = "warn";
      p.textContent = msg;
      warningsEl.appendChild(p);
    });

    svg.setAttribute("width", String(dims.width));
    svg.setAttribute("height", String(dims.height));
    svg.innerHTML = "";

    const defs = document.createElementNS("http://www.w3.org/2000/svg", "defs");
    const marker = document.createElementNS("http://www.w3.org/2000/svg", "marker");
    marker.setAttribute("id", "arrow");
    marker.setAttribute("markerWidth", "8");
    marker.setAttribute("markerHeight", "8");
    marker.setAttribute("refX", "6");
    marker.setAttribute("refY", "3");
    marker.setAttribute("orient", "auto");
    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
    path.setAttribute("d", "M0,0 L0,6 L6,3 z");
    path.setAttribute("fill", "currentColor");
    marker.appendChild(path);
    defs.appendChild(marker);
    svg.appendChild(defs);

    const edgesGroup = document.createElementNS("http://www.w3.org/2000/svg", "g");
    edgesGroup.setAttribute("class", "edges");
    layout.edges.forEach(function (edge) {
      const from = nodeCenter(layout, edge.from);
      const to = nodeCenter(layout, edge.to);
      const g = document.createElementNS("http://www.w3.org/2000/svg", "g");
      g.setAttribute("class", "edge");
      const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
      line.setAttribute("x1", String(from.x));
      line.setAttribute("y1", String(from.y));
      line.setAttribute("x2", String(to.x));
      line.setAttribute("y2", String(to.y));
      g.appendChild(line);
      if (edge.label) {
        const label = document.createElementNS("http://www.w3.org/2000/svg", "text");
        label.setAttribute("class", "edge-label");
        label.setAttribute("x", String((from.x + to.x) / 2));
        label.setAttribute("y", String((from.y + to.y) / 2 - 4));
        label.setAttribute("text-anchor", "middle");
        label.textContent = edge.label;
        g.appendChild(label);
      }
      edgesGroup.appendChild(g);
    });
    svg.appendChild(edgesGroup);

    const nodesGroup = document.createElementNS("http://www.w3.org/2000/svg", "g");
    nodesGroup.setAttribute("class", "nodes");
    layout.nodes.forEach(function (node) {
      const g = document.createElementNS("http://www.w3.org/2000/svg", "g");
      g.setAttribute("class", "node" +
        (node.isEntry ? " entry" : "") +
        (node.isJoin ? " join" : ""));
      g.setAttribute("transform", "translate(" + node.x + "," + node.y + ")");

      const rect = document.createElementNS("http://www.w3.org/2000/svg", "rect");
      rect.setAttribute("width", String(NODE_W));
      rect.setAttribute("height", String(NODE_H));
      g.appendChild(rect);

      const text = document.createElementNS("http://www.w3.org/2000/svg", "text");
      text.setAttribute("x", String(NODE_W / 2));
      text.setAttribute("y", String(NODE_H / 2 + 4));
      text.textContent = node.label;
      g.appendChild(text);


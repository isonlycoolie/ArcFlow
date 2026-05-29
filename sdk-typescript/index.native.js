const { existsSync } = require("node:fs");
const { join } = require("node:path");

const root = __dirname;
const local = join(root, "arcflow-node.node");
const debug = join(root, "..", "target", "debug", "arcflow_node.dll");
const release = join(root, "..", "target", "release", "arcflow_node.dll");

let bindingPath = local;
if (!existsSync(bindingPath)) {
  bindingPath = existsSync(release) ? release : debug;
}

module.exports = require(bindingPath);

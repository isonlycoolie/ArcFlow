import * as assert from "assert";
import { BreakpointManager } from "../debug/debugAdapter";

suite("ArcFlow debug adapter", () => {
  test("breakpoint manager toggles step ids", () => {
    const mgr = new BreakpointManager();
    assert.strictEqual(mgr.list().length, 0);
    assert.strictEqual(mgr.toggle("step-1"), true);
    assert.deepStrictEqual(mgr.list(), ["step-1"]);
    assert.strictEqual(mgr.toggle("step-1"), false);
    assert.deepStrictEqual(mgr.list(), []);
  });
});

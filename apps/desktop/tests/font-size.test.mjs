import test from "node:test";
import assert from "node:assert/strict";

test("font sizes are rounded and kept between 10 and 50 pixels", async () => {
  const { clampFontSize } = await import("../src/shell/font-size.ts");
  assert.equal(clampFontSize(14.4), 14);
  assert.equal(clampFontSize(4), 10);
  assert.equal(clampFontSize(80), 50);
  assert.equal(clampFontSize(Number.NaN), 12);
  assert.equal(clampFontSize(0), 12);
});

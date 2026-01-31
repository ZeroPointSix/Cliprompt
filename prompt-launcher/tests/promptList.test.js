import assert from "node:assert/strict";
import test from "node:test";
import {
  buildRecentList,
  buildTopTags,
  getTagSuggestions
} from "../src/lib/promptList.js";

/**
 * @param {string} id
 * @param {string[]} [tags]
 */
const makePrompt = (id, tags = []) => ({
  id,
  title: id,
  body: "",
  preview: "",
  tags,
  path: `${id}.txt`
});

test("buildRecentList keeps recent order and original index", () => {
  const prompts = [
    makePrompt("a"),
    makePrompt("b"),
    makePrompt("c")
  ];
  const recent = ["c", "a", "missing"];
  const result = buildRecentList(prompts, recent);
  assert.deepEqual(
    result.map((item) => [item.prompt.id, item.index]),
    [
      ["c", 2],
      ["a", 0]
    ]
  );
});

test("buildTopTags sorts by count then tag name and respects limit", () => {
  const prompts = [
    makePrompt("a", ["beta", "alpha"]),
    makePrompt("b", ["beta", "gamma"]),
    makePrompt("c", ["beta"]),
    makePrompt("d")
  ];
  const result = buildTopTags(prompts, 2);
  assert.deepEqual(result, [
    { tag: "beta", count: 3 },
    { tag: "alpha", count: 1 }
  ]);
});

test("getTagSuggestions returns matches for the last tag token", () => {
  const tags = ["alpha", "beta", "Gamma", "gizmo"];
  const result = getTagSuggestions("hello #g", tags);
  assert.deepEqual(result, ["Gamma", "gizmo"]);
});

test("getTagSuggestions returns empty when last token is not a tag", () => {
  const tags = ["alpha", "beta"];
  const result = getTagSuggestions("#alpha hello", tags);
  assert.deepEqual(result, []);
});

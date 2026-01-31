import assert from "node:assert/strict";
import test from "node:test";
import {
  applyTagSuggestion,
  clearTagFilters,
  hasAnyFilters,
  hasTagFilters,
  isTagActive,
  normalizeTagToken,
  toggleTagFilter
} from "../src/lib/launcherFilters.js";

test("normalizeTagToken lowercases and prefixes", () => {
  assert.equal(normalizeTagToken("Hello"), "#hello");
});

test("hasTagFilters detects tag tokens", () => {
  assert.equal(hasTagFilters("hello #tag"), true);
  assert.equal(hasTagFilters("hello world"), false);
});

test("hasAnyFilters considers query and flags", () => {
  assert.equal(
    hasAnyFilters({ query: " ", showFavorites: false, showRecent: false }),
    false
  );
  assert.equal(
    hasAnyFilters({ query: "#tag", showFavorites: false, showRecent: false }),
    true
  );
  assert.equal(
    hasAnyFilters({ query: " ", showFavorites: true, showRecent: false }),
    true
  );
});

test("applyTagSuggestion replaces the last token", () => {
  assert.equal(applyTagSuggestion("", "alpha"), "#alpha");
  assert.equal(applyTagSuggestion("hello #b", "beta"), "hello #beta");
});

test("clearTagFilters removes tag tokens only", () => {
  assert.equal(clearTagFilters("hello #a world #b"), "hello world");
});

test("toggleTagFilter adds or removes tokens", () => {
  assert.equal(toggleTagFilter("hello", "alpha"), "hello #alpha");
  assert.equal(toggleTagFilter("hello #alpha", "alpha"), "hello");
});

test("isTagActive matches case-insensitively", () => {
  assert.equal(isTagActive("hello #Alpha", "alpha"), true);
  assert.equal(isTagActive("hello world", "alpha"), false);
});

/**
 * @typedef {import("./types").PromptEntry} PromptEntry
 */

/**
 * @param {PromptEntry[]} prompts
 * @param {string[]} recentIds
 * @returns {{ prompt: PromptEntry; index: number }[]}
 */
export function buildRecentList(prompts, recentIds) {
  const map = new Map(
    prompts.map((prompt, index) => [prompt.id, { prompt, index }])
  );
  const results = [];
  for (const id of recentIds) {
    const entry = map.get(id);
    if (entry) {
      results.push(entry);
    }
  }
  return results;
}

/**
 * @param {string} value
 * @param {string[]} tags
 * @returns {string[]}
 */
export function getTagSuggestions(value, tags) {
  const parts = value.trim().split(/\s+/);
  const last = parts[parts.length - 1] ?? "";
  if (!last.startsWith("#")) {
    return [];
  }
  const keyword = last.slice(1).toLowerCase();
  return tags
    .filter((tag) => tag.toLowerCase().startsWith(keyword))
    .slice(0, 8);
}

/**
 * @param {PromptEntry[]} prompts
 * @param {number} limit
 * @returns {{ tag: string; count: number }[]}
 */
export function buildTopTags(prompts, limit) {
  const counts = new Map();
  prompts.forEach((prompt) => {
    prompt.tags?.forEach((tag) => {
      counts.set(tag, (counts.get(tag) ?? 0) + 1);
    });
  });
  return Array.from(counts.entries())
    .sort((a, b) => {
      if (b[1] !== a[1]) {
        return b[1] - a[1];
      }
      return a[0].localeCompare(b[0]);
    })
    .slice(0, limit)
    .map(([tag, count]) => ({ tag, count }));
}

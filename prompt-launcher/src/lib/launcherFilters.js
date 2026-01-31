/**
 * @param {string} tag
 * @returns {string}
 */
export function normalizeTagToken(tag) {
  return `#${tag.toLowerCase()}`;
}

/**
 * @param {string} query
 * @returns {boolean}
 */
export function hasQuery(query) {
  return query.trim().length > 0;
}

/**
 * @param {string} query
 * @returns {boolean}
 */
export function hasTagFilters(query) {
  return query
    .split(/\s+/)
    .some((part) => part.startsWith("#") && part.length > 1);
}

/**
 * @param {{ query: string; showFavorites: boolean; showRecent: boolean }} options
 * @returns {boolean}
 */
export function hasAnyFilters({ query, showFavorites, showRecent }) {
  return hasQuery(query) || showFavorites || showRecent;
}

/**
 * @param {string} query
 * @param {string} tag
 * @returns {boolean}
 */
export function isTagActive(query, tag) {
  const token = normalizeTagToken(tag);
  return query
    .split(/\s+/)
    .some((part) => part.toLowerCase() === token);
}

/**
 * @param {string} query
 * @param {string} tag
 * @returns {string}
 */
export function applyTagSuggestion(query, tag) {
  const parts = query.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) {
    return `#${tag}`;
  }
  parts[parts.length - 1] = `#${tag}`;
  return parts.join(" ").trim();
}

/**
 * @param {string} query
 * @returns {string}
 */
export function clearTagFilters(query) {
  const parts = query.split(/\s+/).filter(Boolean);
  const remaining = parts.filter((part) => !part.startsWith("#"));
  return remaining.join(" ").trim();
}

/**
 * @param {string} query
 * @param {string} tag
 * @returns {string}
 */
export function toggleTagFilter(query, tag) {
  const token = normalizeTagToken(tag);
  const parts = query.split(/\s+/).filter(Boolean);
  const hasTag = parts.some((part) => part.toLowerCase() === token);
  const filtered = parts.filter((part) => part.toLowerCase() !== token);
  const next = hasTag ? filtered : [...filtered, token];
  return next.join(" ").trim();
}

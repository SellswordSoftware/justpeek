/**
 * Fuzzy match query against text.
 * Returns `{ match: true, score }` or `null`.
 *
 * @param {string} query
 * @param {string} text
 * @returns {{ match: true, score: number } | null}
 */
export function fuzzyMatch(query, text) {
  if (!query || query.length === 0) {
    return { match: true, score: 1 };
  }

  const normalizedQuery = query.toLowerCase();
  const normalizedText = text.toLowerCase();

  let position = 0;
  let score = 0;
  let streak = 0;

  for (let index = 0; index < normalizedQuery.length; index += 1) {
    const next = normalizedText.indexOf(normalizedQuery[index], position);
    if (next === -1) {
      return null;
    }

    if (next === position) {
      streak += 1;
    } else {
      streak = 0;
    }

    score += streak;
    position = next + 1;
  }

  return { match: true, score };
}

/**
 * Filter items using fuzzy match across multiple fields.
 *
 * @template T
 * @param {T[]} items
 * @param {string} query
 * @param {(item: T) => string[]} getKey
 * @returns {T[]}
 */
export function fuzzyFilter(items, query, getKey) {
  if (!query || query.length === 0) {
    return items;
  }

  const scored = [];

  for (const item of items) {
    const searchable = getKey(item);
    let bestScore = -1;

    for (const value of searchable) {
      const result = fuzzyMatch(query, value);
      if (result && result.score > bestScore) {
        bestScore = result.score;
      }
    }

    if (bestScore >= 0) {
      scored.push({ item, score: bestScore });
    }
  }

  scored.sort((left, right) => right.score - left.score);
  return scored.map((entry) => entry.item);
}

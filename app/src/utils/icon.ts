export function getIconUrlPath(symbol: string, size: 24 | 40) {
  if (!symbol || !size) return;
  return new URL(`../img/ic_${symbol.toLowerCase()}_${size}.svg`, import.meta.url).href;
}

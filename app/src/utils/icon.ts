const IC_ICONS = [
  "btc",
  "eth",
  "gmx",
  "sol",
  "usdc",
  "usdg",
  "wsol",
];

export function getIconUrlPath(symbol: string, size: 24 | 40) {
  if (!symbol || !size) return;
  const lowerCaseSymbol = symbol.toLocaleLowerCase();
  const icPath = new URL(`../img/ic_${lowerCaseSymbol}_${size}.svg`, import.meta.url).href;
  return IC_ICONS.includes(lowerCaseSymbol) ? icPath : `/icons/${lowerCaseSymbol}.svg`;
}

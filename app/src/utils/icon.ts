import manifest from "cryptocurrency-icons/manifest.json";

const IC_ICONS = [
  "btc",
  "eth",
  "gmx",
  "sol",
  "usdc",
  "usdg",
  "wsol",
  "pepe",
];

function symbolExists(symbol: string): boolean {
  return manifest.some(icon => icon.symbol.toLocaleLowerCase() === symbol.toLowerCase());
}

export function getIconUrlPath(symbol: string, size: 24 | 40) {
  if (!symbol || !size) return;
  const lowerCaseSymbol = symbol.toLocaleLowerCase();
  const icPath = new URL(`../img/ic_${lowerCaseSymbol}_${size}.svg`, import.meta.url).href;
  return IC_ICONS.includes(lowerCaseSymbol) ? icPath : symbolExists(lowerCaseSymbol) ? `/icons/${lowerCaseSymbol}.svg` : `/icons/generic.svg`;
}

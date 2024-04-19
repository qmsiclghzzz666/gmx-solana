import * as fs from "fs/promises";

interface GMSOLDeployment {
  store: string,
  oracle: string,
  markets: Market[],
}

interface Market {
  name: string,
  market_token: string,
  index_token: string,
  long_token: string,
  short_token: string,
}

export const loadGMSOLDeployment = async (path?: string) => {
  if (path) {
    const content = await fs.readFile(path, 'utf-8');
    return JSON.parse(content) as GMSOLDeployment;
  }
};

import { parseMode, parseOperation } from "@/components/GmSwap/utils";

export const earnLoader = ({ request }: { request: Request }) => {
  const url = new URL(request.url);
  return {
    market: url.searchParams.get("market"),
    operation: parseOperation(url.searchParams.get("operation")),
    mode: parseMode(url.searchParams.get("mode")),
  };
};

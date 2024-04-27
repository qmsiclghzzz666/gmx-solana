import { FormEventHandler, useCallback } from "react";

export function TradeForm() {
  const handleSubmit: FormEventHandler<HTMLFormElement> = useCallback((e) => {
    e.preventDefault();
  }, []);

  return (
    <form onSubmit={handleSubmit}>
    </form>
  );
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export async function action({ request }: { request: Request }) {
  console.log("performing gm action", request);
  await sleep(3000);
  console.log("gm action done", request);
  return {

  };
}

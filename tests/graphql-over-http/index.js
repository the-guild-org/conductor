import { serverAudits } from "graphql-http";
import { it } from "node:test";

for (const audit of serverAudits({
  url: process.env.SERVER_URL || "http://127.0.0.1:9000/graphql",
  fetchFn: fetch,
})) {
  it(audit.name, async () => {
    const result = await audit.fn();
    if (result.status === "error") {
      throw result.reason;
    }
    if (result.status === "warn") {
      console.warn(`⚠️ warning: ${result.reason}`); // or throw if you want full compliance (warnings are not requirements)
    }
  });
}

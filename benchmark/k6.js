// @ts-check

// @ts-expect-error - TS doesn't know this import
import { textSummary } from "https://jslib.k6.io/k6-summary/0.0.1/index.js";
// @ts-expect-error - TS doesn"t know this import
import { githubComment } from "https://raw.githubusercontent.com/dotansimha/k6-github-pr-comment/master/lib.js";
import http from "k6/http";
import { Rate } from "k6/metrics";

const VUS = 100;
const DURATION = "60s";

export const validGraphQLResponse = new Rate("valid_graphql_response");
export const validHttpCode = new Rate("valid_http_code");

export const options = {
  stages: [
    { duration: "10s", target: 100 }, // warm up
    { duration: DURATION, target: VUS }, // ramp up
    { duration: "10s", target: 0 }, // cool down
  ],
  thresholds: {
    http_req_duration: ["avg<=30"], // request duration should be less than the value specified
    http_req_failed: ["rate==0"], // no failed requests
    [validGraphQLResponse.name]: ["rate==1"],
    [validHttpCode.name]: ["rate==1"],
  },
};

export function handleSummary(data) {
  if (__ENV.GITHUB_TOKEN) {
    githubComment(data, {
      token: __ENV.GITHUB_TOKEN,
      commit: __ENV.GITHUB_SHA,
      pr: __ENV.GITHUB_PR,
      org: "the-guild-org",
      repo: "conductor-t2",
      renderTitle({ passes }) {
        return passes ? "✅ Benchmark Results" : "❌ Benchmark Failed";
      },
      renderMessage({ passes, checks, thresholds }) {
        const result = [];

        if (thresholds.failures) {
          result.push(
            `**Performance regression detected**: it seems like your Pull Request adds some extra latency to Conductor hot paths.`
          );
        }

        if (checks.failures) {
          result.push("**Failed assertions detected**");
        }

        if (!passes) {
          result.push(
            `> If the performance regression is expected, please increase the failing threshold.`
          );
        }

        return result.join("\n");
      },
    });
  }
  return {
    stdout: textSummary(data, { indent: " ", enableColors: true }),
  };
}

export default function () {
  const res = http.post(
    "http://127.0.0.1:9000/graphql",
    JSON.stringify({
      query: /* GraphQL */ `
        query authors {
          authors {
            id
            name
            company
            books {
              id
              name
              numPages
            }
          }
        }
      `,
    }),
    {
      headers: { "Content-Type": "application/json" },
    }
  );

  if (res.status !== 200) {
    printOnce(
      "http_code",
      `‼️ Failed to run HTTP request, here's a sample response:`,
      res
    );
  } else {
    validHttpCode.add(1);
  }

  const json = res.json();
  // @ts-expect-error
  const hasGraphQLErrors = json && json.errors && json.errors.length > 0;

  if (hasGraphQLErrors) {
    printOnce(
      "graphql_errors",
      `‼️ Got GraphQL errors, here's a sample:`,
      res.body
    );
  } else {
    validGraphQLResponse.add(1);
  }
}

let identifiersMap = {};
function printOnce(identifier, ...args) {
  if (identifiersMap[identifier]) {
    return;
  }

  console.log(...args);
  identifiersMap[identifier] = true;
}

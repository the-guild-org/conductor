// @ts-check

// @ts-expect-error - TS doesn't know this import
import { textSummary } from "https://jslib.k6.io/k6-summary/0.0.1/index.js";
// @ts-expect-error - TS doesn"t know this import
import { githubComment } from "https://raw.githubusercontent.com/dotansimha/k6-github-pr-comment/master/lib.js";
import http from "k6/http";
import { Rate } from "k6/metrics";
// @ts-expect-error - TS doesn't know this import
import { tagWithCurrentStageProfile } from "https://jslib.k6.io/k6-utils/1.3.0/index.js";

const VUS = 100;
const DURATION = "60s";

export const validGraphQLResponse = new Rate("valid_graphql_response");
export const validHttpCode = new Rate("valid_http_code");

export const options = {
  stages: [
    { duration: "10s", target: VUS },
    { duration: DURATION, target: VUS },
    { duration: "10s", target: 0 },
  ],
  thresholds: {
    "http_req_duration{stage_profile:steady}": ["avg<=30"], // request duration should be less than the value specified
    "http_req_failed{stage_profile:steady}": ["rate==0"], // no failed requests
    [`${validGraphQLResponse.name}{stage_profile:steady}`]: ["rate==1"],
    [`${validHttpCode.name}{stage_profile:steady}`]: ["rate==1"],
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
  tagWithCurrentStageProfile();

  const res = http.post(
    "http://127.0.0.1:9000/graphql",
    JSON.stringify({
      query: /* GraphQL */ `
        query authors {
          authors {
            id
            name
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

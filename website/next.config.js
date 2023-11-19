import { withGuildDocs } from "@theguild/components/next.config";

/** @type {import("next").Config} */
export default withGuildDocs({
  eslint: {
    ignoreDuringBuilds: true,
  },
});

/* eslint sort-keys: error */
import { useRouter } from "next/router";
import { defineConfig, FooterExtended } from "@theguild/components";

export default defineConfig({
  docsRepositoryBase:
    "https://github.com/the-guild-org/conductor-t2/tree/master/website", // base URL for the docs repository
  logoLink: "/docs",
  siteName: "CONDUCTOR",
});

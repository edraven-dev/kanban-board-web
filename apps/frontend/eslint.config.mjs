import { nextJsConfig } from "@repo/eslint-config/next-js";

/** @type {import("eslint").Linter.Config[]} */
const config = [...nextJsConfig, { ignores: ["coverage/**"] }];

export default config;

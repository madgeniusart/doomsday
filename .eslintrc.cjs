/* eslint-env node */
module.exports = {
  root: true,
  env: { browser: true, es2021: true, node: true },
  extends: [
    "eslint:recommended",
    "plugin:@typescript-eslint/recommended",
    "plugin:react/recommended",
    "plugin:react/jsx-runtime",
    "plugin:react-hooks/recommended",
  ],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
    ecmaFeatures: { jsx: true },
  },
  plugins: ["@typescript-eslint", "react", "react-hooks"],
  ignorePatterns: [
    "dist",
    "node_modules",
    "src-tauri",
    "browser-extension",
    "*.config.js",
    "*.config.ts",
    "*.cjs",
  ],
  rules: {
    // TypeScript already checks these; avoid duplicate/noisy errors
    "no-undef": "off",
    "no-unused-vars": "off",
    "@typescript-eslint/no-unused-vars": [
      "warn",
      { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
    ],
    "@typescript-eslint/no-explicit-any": "warn",
    "@typescript-eslint/explicit-module-boundary-types": "off",
    "@typescript-eslint/no-non-null-assertion": "off",

    // TypeScript types replace PropTypes entirely in this project
    "react/prop-types": "off",
    // React 17+ JSX transform (no `import React` needed per-file)
    "react/react-in-jsx-scope": "off",
  },
  settings: {
    react: { version: "detect" },
  },
};

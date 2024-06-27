// @ts-check
import eslint from "@eslint/js";
import prettier from "eslint-config-prettier";
import react from "eslint-plugin-react";
import reacthooks from "eslint-plugin-react-hooks";
import reactRecommended from "eslint-plugin-react/configs/jsx-runtime";
import tseslint from "typescript-eslint";

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  { ...reactRecommended, plugins: { react } },
  {
    plugins: { "react-hooks": reacthooks },
    rules: { "react-hooks/rules-of-hooks": "error", "react-hooks/exhaustive-deps": "warn" },
  },
  prettier,
  {
    languageOptions: {
      parserOptions: {
        project: ["./tsconfig.json"],
        tsconfigRootDir: import.meta.dirname,
      },
    },
    settings: { react: { version: "detect" } },
    rules: {
      "@typescript-eslint/no-unused-vars": ["error", { varsIgnorePattern: "^_", argsIgnorePattern: "^_" }],
      "no-console": "error",
      "react/no-children-prop": ["error", { allowFunctions: true }],
    },
  },
  { ignores: ["src/components/ui/*.tsx", "src/api.d.ts", "*.config.js"] },
);

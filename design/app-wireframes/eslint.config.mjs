import { plugin as shadcn } from "@shadcn/lint"
import tsParser from "@typescript-eslint/parser"
import { defineConfig } from "eslint/config"

const note = "Eva design system: use tokens from src/theme.css and components from @/ds. See DESIGN.md."

export default defineConfig([
  {
    files: ["src/**/*.{js,jsx,ts,tsx}"],
    languageOptions: { parser: tsParser, parserOptions: { ecmaFeatures: { jsx: true } } },
    plugins: { shadcn },
    settings: { shadcn: { ui: "@/ds", note } },
    rules: {
      "shadcn/no-raw-colors": "error",
      "shadcn/no-arbitrary-values": "error",
      "shadcn/no-inline-styles": "error",
      "shadcn/no-unknown-classes": "error",
      "shadcn/require-static-classes": "error",
      "shadcn/no-restyle": ["error", {
        allow: ["layout", "spacing"],
        contracts: [
          { pattern: "^(CardTitle|Eyebrow|Headline)$", allow: ["layout", "spacing", "typography"] },
          { pattern: "^Button$", allow: ["layout", "spacing"] },
          { pattern: "^Card$", allow: ["layout", "spacing"] },
        ],
      }],
    },
  },
  {
    // the design system itself defines the styling
    files: ["src/ds/**/*.{js,jsx,ts,tsx}"],
    rules: { "shadcn/no-restyle": "off" },
  },
])

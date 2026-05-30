/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        sans: ["Aptos", "Segoe UI", "sans-serif"],
        mono: ["JetBrains Mono", "Cascadia Code", "monospace"]
      }
    }
  },
  plugins: []
};

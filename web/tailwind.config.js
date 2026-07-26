/** @type {import('tailwindcss').Config} */
import { heroui } from "@heroui/theme";

export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
    "./node_modules/@heroui/theme/dist/components/(button|ripple|spinner|input|snippet|code|divider|modal|toggle|select|dropdown).js",
  ],
  theme: {
    extend: {},
  },
  darkMode: "class",
  plugins: [heroui()],
};

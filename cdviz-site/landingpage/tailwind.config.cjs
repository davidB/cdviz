const defaultTheme = require("tailwindcss/defaultTheme");
// const colors = require("tailwindcss/colors");

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      spacing: {
        // see [Perfect Ratios & How to Find Them - YouTube](https://www.youtube.com/watch?v=r1DANFZYJDw)
        "2xs": "0.3rem",
        xs: "0.486rem",
        sm: "0.7862rem",
        md: "1.272rem", // "1rem", // the base
        lg: "2.058rem",
        xl: "3.330rem",
        "2xl": "5.388rem",
        "4xl": "8.778rem",
      },
      // theme build with:
      // - https://www.realtimecolors.com/?colors=0e151b-c9d2de-f29107-a294c2-725190&fonts=DroidSans-DroidSans
      // - alternative: https://www.realtimecolors.com/?colors=0e0d0b-f9f8f6-fbbd23-abbfb9-8d9daa&fonts=DroidSans-DroidSans
      // - manually set `DEFAULT` when export in Tailwind CSS + Themes + Shades
      // - export in Tailwind CSS + Themes  in HSL (to be able to use opacity `/5`)
      // - in source of realtime the number suffixing the color is the opacity

      // fontSize: {
      //   sm: "0.750rem",
      //   base: "1rem",
      //   xl: "1.333rem",
      //   "2xl": "1.777rem",
      //   "3xl": "2.369rem",
      //   "4xl": "3.158rem",
      //   "5xl": "4.210rem",
      // },
      fontFamily: {
        sans: ["Droid Sans", ...defaultTheme.fontFamily.sans],
        heading: "Droid Sans",
        body: "Droid Sans",
      },
      fontWeight: {
        normal: "400",
        bold: "700",
      },
      colors: {
        // text: colors.slate[50],
        // background: colors.slate[700],
        // primary: colors.amber[200],
        // secondary: colors.purple[600],
        // accent: colors.violet[500],

        text: "hsl(var(--text))",
        background: "hsl(var(--background))",
        primary: "hsl(var(--primary))",
        secondary: "hsl(var(--secondary))",
        accent: "hsl(var(--accent))",
      },
    },
  },
  plugins: [],
  darkMode: "selector",
};

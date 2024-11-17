const defaultTheme = require("tailwindcss/defaultTheme");

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
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
        // text: "#031026",
        // background: "#e7ecf3",
        // primary: "#f39208", //"#ff6600",
        // secondary: "#685ff2",
        // accent: "#3d12d9",
        text: "var(--text)",
        background: "var(--background)",
        primary: "var(--primary)",
        secondary: "var(--secondary)",
        accent: "var(--accent)",
      },
    },
  },
  plugins: [require("@tailwindcss/forms")],
  darkMode: "selector",
};

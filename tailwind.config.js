/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        cyber: {
          bg: "#0D1117",
          surface: "#161B22",
          card: "#1C2333",
          border: "#30363D",
          green: "#00E5A0",
          cyan: "#00C9DB",
          blue: "#0A84FF",
          orange: "#F0883E",
          red: "#F85149",
          text: "#E6EDF3",
          muted: "#8B949E",
        },
      },
      fontFamily: {
        sans: ["Noto Sans", "Inter", "system-ui", "sans-serif"],
        mono: ["JetBrains Mono", "Fira Code", "monospace"],
      },
      backgroundImage: {
        "glow-green": "linear-gradient(135deg, #00E5A0 0%, #00C9DB 100%)",
        "glow-blue": "linear-gradient(135deg, #00C9DB 0%, #0A84FF 100%)",
        "glow-purple": "linear-gradient(135deg, #0A84FF 0%, #A855F7 100%)",
        "glow-orange": "linear-gradient(135deg, #F0883E 0%, #F85149 100%)",
      },
      boxShadow: {
        neon: "0 0 20px rgba(0, 229, 160, 0.15)",
        "neon-cyan": "0 0 20px rgba(0, 201, 219, 0.15)",
        "neon-blue": "0 0 20px rgba(10, 132, 255, 0.15)",
        glass: "0 8px 32px rgba(0, 0, 0, 0.3)",
      },
      backdropBlur: {
        xs: "2px",
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "glow": "glow 2s ease-in-out infinite alternate",
      },
      keyframes: {
        glow: {
          "0%": { boxShadow: "0 0 5px rgba(0, 229, 160, 0.2)" },
          "100%": { boxShadow: "0 0 20px rgba(0, 229, 160, 0.4)" },
        },
      },
    },
  },
  plugins: [
    require("tailwindcss-animate"),
    require("@tailwindcss/forms"),
  ],
};

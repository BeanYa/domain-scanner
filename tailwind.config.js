/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        cyber: {
          bg: "#000000",
          "bg-elevated": "#030303",
          surface: "#0c0c0c",
          card: "#111111",
          "card-hover": "#1a1a1a",
          border: "#27272a",
          "border-light": "#3f3f3f",
          // Accent palette
          green: "#e8ece8",
          "green-dim": "#c9d6cc",
          cyan: "#c9ccd1",
          "cyan-bright": "#f4f5f7",
          blue: "#dce2ea",
          "blue-soft": "#e9ecf2",
          orange: "#d8c7a4",
          "orange-warm": "#eadbbb",
          red: "#e2aaa6",
          "red-dim": "#c6827e",
          purple: "#d0d4d4",
          pink: "#c9ccd1",
          // Text hierarchy
          text: "#ffffff",
          "text-secondary": "#c9ccd1",
          muted: "#767d88",
          "muted-dim": "#565d66",
        },
      },
      fontFamily: {
        sans: ['"abcNormal"', '"Aptos"', '"Segoe UI"', "system-ui", "sans-serif"],
        mono: ['"JetBrains Mono"', '"Cascadia Mono"', "Consolas", "monospace"],
        display: ['"abcNormal"', '"Aptos"', '"Segoe UI"', "sans-serif"],
      },
      fontSize: {
        "2xs": ["0.625rem", { lineHeight: "0.875rem" }],
        "xs": ["0.75rem", { lineHeight: "1.125rem" }],
        sm: ["0.8125rem", { lineHeight: "1.375rem" }],
        base: ["0.875rem", { lineHeight: "1.5rem" }],
        lg: ["0.9375rem", { lineHeight: "1.5rem" }],
      },
      backgroundImage: {
        "glow-green": "linear-gradient(90deg, #ffffff 0%, #ffffff 100%)",
        "glow-blue": "linear-gradient(90deg, #c9ccd1 0%, #c9ccd1 100%)",
        "glow-purple": "linear-gradient(90deg, #d0d4d4 0%, #d0d4d4 100%)",
        "glow-orange": "linear-gradient(90deg, #d8c7a4 0%, #d8c7a4 100%)",
        "glow-pink": "linear-gradient(90deg, #c9ccd1 0%, #c9ccd1 100%)",
        "mesh-grid": "linear-gradient(90deg, transparent 0%, transparent 100%)",
        "noise-texture": "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)' opacity='0.04'/%3E%3C/svg%3E\")",
        "gradient-radial": "radial-gradient(var(--tw-gradient-stops))",
        "gradient-conic":
          "conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))",
      },
      boxShadow: {
        neon: "none",
        "neon-cyan": "none",
        "neon-blue": "none",
        "neon-orange": "none",
        "neon-red": "none",
        glow: "none",
        "glow-lg": "none",
        glass: "none",
        "glass-sm": "none",
        elevated: "none",
        "inner-glow": "none",
      },
      backdropBlur: {
        xs: "2px",
      },
      animation: {
        "pulse-slow": "pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite",
        "glow": "glow 2s ease-in-out infinite alternate",
        "shimmer": "shimmer 2s linear infinite",
        "fade-in": "fadeIn 0.3s ease-out",
        "fade-up": "fadeUp 0.4s ease-out",
        "slide-in-right": "slideInRight 0.3s ease-out",
        "scale-in": "scaleIn 0.2s ease-out",
        float: "float 6s ease-in-out infinite",
        "spin-slow": "spin 8s linear infinite",
      },
      keyframes: {
        glow: {
          "0%": { boxShadow: "0 0 5px rgba(0, 229, 160, 0.15)" },
          "100%": { boxShadow: "0 0 25px rgba(0, 229, 160, 0.35)" },
        },
        shimmer: {
          "0%": { backgroundPosition: "-200% 0" },
          "100%": { backgroundPosition: "200% 0" },
        },
        fadeIn: {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
        fadeUp: {
          from: { opacity: "0", transform: "translateY(8px)" },
          to: { opacity: "1", transform: "translateY(0)" },
        },
        slideInRight: {
          from: { opacity: "0", transform: "translateX(12px)" },
          to: { opacity: "1", transform: "translateX(0)" },
        },
        scaleIn: {
          from: { opacity: "0", transform: "scale(0.96)" },
          to: { opacity: "1", transform: "scale(1)" },
        },
        float: {
          "0%, 100%": { transform: "translateY(0)" },
          "50%": { transform: "translateY(-6px)" },
        },
      },
      borderRadius: {
        "4xl": "1.25rem",
        "5xl": "1.5rem",
      },
    },
    plugins: [
      require("tailwindcss-animate"),
      require("@tailwindcss/forms"),
    ],
  },
};

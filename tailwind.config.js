/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        cyber: {
          bg: "#0A0E14",
          "bg-elevated": "#0D1117",
          surface: "#131920",
          card: "#1A222D",
          "card-hover": "#1F2937",
          border: "#252D3A",
          "border-light": "#2F3B4C",
          // Accent palette
          green: "#00E5A0",
          "green-dim": "#00B87A",
          cyan: "#00C9DB",
          "cyan-bright": "#38D8ED",
          blue: "#0A84FF",
          "blue-soft": "#5BA8FF",
          orange: "#F0883E",
          "orange-warm": "#FF9940",
          red: "#FF5F57",
          "red-dim": "#F85149",
          purple: "#A78BFA",
          pink: "#F472B6",
          // Text hierarchy
          text: "#EEF1F6",
          "text-secondary": "#C9D1D9",
          muted: "#7D8590",
          "muted-dim": "#545D68",
        },
      },
      fontFamily: {
        sans: ['"Space Grotesk"', "Inter", "system-ui", "sans-serif"],
        mono: ['"JetBrains Mono"', "Fira Code", "monospace"],
        display: ['"Space Grotesk"', "sans-serif"],
      },
      fontSize: {
        "2xs": ["0.625rem", { lineHeight: "0.875rem" }],
        "xs": ["0.75rem", { lineHeight: "1.125rem" }],
        sm: ["0.8125rem", { lineHeight: "1.375rem" }],
        base: ["0.875rem", { lineHeight: "1.5rem" }],
        lg: ["0.9375rem", { lineHeight: "1.5rem" }],
      },
      backgroundImage: {
        "glow-green": "linear-gradient(135deg, #00E5A0 0%, #00C9DB 100%)",
        "glow-blue": "linear-gradient(135deg, #00C9DB 0%, #0A84FF 100%)",
        "glow-purple": "linear-gradient(135deg, #0A84FF 0%, #A78BFA 100%)",
        "glow-orange": "linear-gradient(135deg, #F0883E 0%, #FF5F57 100%)",
        "glow-pink": "linear-gradient(135deg, #F472B6 0%, #A78BFA 100%)",
        "mesh-grid":
          "radial-gradient(circle at 1px 1px, rgba(255,255,255,0.03) 1px, transparent 0)",
        "noise-texture": "url(\"data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)' opacity='0.04'/%3E%3C/svg%3E\")",
        "gradient-radial": "radial-gradient(var(--tw-gradient-stops))",
        "gradient-conic":
          "conic-gradient(from 180deg at 50% 50%, var(--tw-gradient-stops))",
      },
      boxShadow: {
        neon: "0 0 20px rgba(0, 229, 160, 0.12), 0 0 60px rgba(0, 229, 160, 0.05)",
        "neon-cyan": "0 0 20px rgba(0, 201, 219, 0.12), 0 0 60px rgba(0, 201, 219, 0.05)",
        "neon-blue": "0 0 20px rgba(10, 132, 255, 0.12), 0 0 60px rgba(10, 132, 255, 0.05)",
        "neon-orange": "0 0 20px rgba(240, 136, 62, 0.15)",
        "neon-red": "0 0 20px rgba(255, 95, 87, 0.15)",
        glow: "0 0 40px rgba(0, 229, 160, 0.08)",
        "glow-lg": "0 0 80px rgba(0, 229, 160, 0.06)",
        glass: "0 8px 32px rgba(0, 0, 0, 0.4), inset 0 1px 0 rgba(255, 255, 255, 0.04)",
        "glass-sm": "0 2px 16px rgba(0, 0, 0, 0.3)",
        elevated: "0 4px 24px rgba(0, 0, 0, 0.3), 0 0 0 1px rgba(255, 255, 255, 0.04)",
        "inner-glow": "inset 0 1px 0 rgba(255, 255, 255, 0.05), inset 0 -1px 0 rgba(0, 0, 0, 0.2)",
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

/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./src/**/*.{js,jsx,ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        // Dark theme colors
        'primary': '#0F172A',    // Deep slate
        'secondary': '#1E293B',  // Darker slate
        'cta': '#22C55E',        // Success green
        'background': '#020617', // Near black (OLED)
        'text': '#F8FAFC',       // Light text
        'accent': '#FACC15',     // Yellow-300 for highlights
      },
    },
  },
  plugins: [],
}
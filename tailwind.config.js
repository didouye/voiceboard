/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{html,ts}",
  ],
  theme: {
    extend: {
      colors: {
        background: '#0a0a0f',
        surface: {
          DEFAULT: '#12121a',
          hover: '#1a1a25',
        },
        border: {
          DEFAULT: '#2a2a3a',
        },
        accent: {
          DEFAULT: '#9d4edd',
          glow: '#bf5af2',
          hot: '#ff00ff',
        },
        text: {
          primary: '#ffffff',
          secondary: '#888899',
          muted: '#555566',
        },
        status: {
          success: '#22c55e',
          warning: '#eab308',
          error: '#ef4444',
          info: '#00d4ff',
        },
      },
      animation: {
        'glow-pulse': 'glow-pulse 1s ease-in-out infinite',
        'bounce-click': 'bounce-click 200ms ease-out',
        'slide-in': 'slide-in 150ms ease-out',
        'fade-in': 'fade-in 150ms ease-out',
        'scale-in': 'scale-in 150ms ease-out',
      },
      keyframes: {
        'glow-pulse': {
          '0%, 100%': { boxShadow: '0 0 20px rgba(157, 78, 221, 0.5)' },
          '50%': { boxShadow: '0 0 40px rgba(157, 78, 221, 0.8)' },
        },
        'bounce-click': {
          '0%': { transform: 'scale(0.95)' },
          '100%': { transform: 'scale(1)' },
        },
        'slide-in': {
          '0%': { transform: 'translateX(-100%)', opacity: '0' },
          '100%': { transform: 'translateX(0)', opacity: '1' },
        },
        'fade-in': {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        'scale-in': {
          '0%': { transform: 'scale(0.95)', opacity: '0' },
          '100%': { transform: 'scale(1)', opacity: '1' },
        },
      },
    },
  },
  plugins: [],
}

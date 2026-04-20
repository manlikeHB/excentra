import type { Config } from 'tailwindcss'

const config: Config = {
  content: [
    './src/pages/**/*.{js,ts,jsx,tsx,mdx}',
    './src/components/**/*.{js,ts,jsx,tsx,mdx}',
    './src/app/**/*.{js,ts,jsx,tsx,mdx}',
  ],
  theme: {
    extend: {
      colors: {
        'bg-base': '#0A0B0D',
        'bg-surface': '#111318',
        'bg-elevated': '#1A1D24',
        'bg-border': '#1F232C',
        accent: '#3B82F6',
        'accent-hover': '#2563EB',
        buy: '#10B981',
        sell: '#EF4444',
        'text-primary': '#F1F5F9',
        'text-secondary': '#94A3B8',
        'text-muted': '#475569',
      },
      fontFamily: {
        sans: ['DM Sans', 'system-ui', 'sans-serif'],
        mono: ['DM Mono', 'monospace'],
      },
      animation: {
        shimmer: 'shimmer 2s linear infinite',
        'fade-in': 'fadeIn 150ms ease-out',
        'scale-in': 'scaleIn 150ms ease-out',
      },
      keyframes: {
        shimmer: {
          '0%': { backgroundPosition: '-200% 0' },
          '100%': { backgroundPosition: '200% 0' },
        },
        fadeIn: {
          '0%': { opacity: '0' },
          '100%': { opacity: '1' },
        },
        scaleIn: {
          '0%': { opacity: '0', transform: 'scale(0.95)' },
          '100%': { opacity: '1', transform: 'scale(1)' },
        },
      },
    },
  },
  plugins: [],
}

export default config

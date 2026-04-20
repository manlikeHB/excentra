import type { Metadata } from 'next'
import '@/styles/globals.css'
import { Providers } from '@/lib/context'
import { QueryProvider } from './providers'
import { Toaster } from 'sonner'

export const metadata: Metadata = {
  title: 'Excentra Exchange',
  description: 'Professional spot cryptocurrency trading',
}

export default function RootLayout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en" className="dark">
      <body>
        <QueryProvider>
          <Providers>
            {children}
          </Providers>
        </QueryProvider>
        <Toaster
          position="top-right"
          theme="dark"
          toastOptions={{
            style: {
              background: '#111318',
              border: '1px solid #1F232C',
              color: '#F1F5F9',
              fontSize: '13px',
            },
          }}
        />
      </body>
    </html>
  )
}

import type { Metadata } from 'next';
import './globals.css';

export const metadata: Metadata = {
  title: 'Distronomicon Desktop',
  description: 'Interface simples para configurar e operar o Distronomicon',
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return <html lang="pt-BR"><body>{children}</body></html>;
}

'use client';

import { useMemo, useState } from 'react';

type Action = 'check' | 'version' | 'update';

export default function Home() {
  const [app, setApp] = useState('meu-app');
  const [repo, setRepo] = useState('owner/repository');
  const [action, setAction] = useState<Action>('check');
  const [copied, setCopied] = useState(false);

  const command = useMemo(() => {
    const safeApp = app.trim() || 'meu-app';
    const safeRepo = repo.trim() || 'owner/repository';
    if (action === 'version') return `distronomicon --app ${safeApp} version`;
    if (action === 'update') return `distronomicon --app ${safeApp} update --repo ${safeRepo}`;
    return `distronomicon --app ${safeApp} check --repo ${safeRepo}`;
  }, [app, repo, action]);

  async function copyCommand() {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    window.setTimeout(() => setCopied(false), 1600);
  }

  return (
    <main className="shell">
      <header className="topbar">
        <div className="brand"><span className="logo">D</span><div><strong>Distronomicon</strong><small>Desktop</small></div></div>
        <span className="status"><i /> Interface online</span>
      </header>

      <section className="hero">
        <p className="eyebrow">PAINEL SIMPLES</p>
        <h1>Atualizações sem<br/><span>complicação.</span></h1>
        <p className="lead">Uma interface amigável para montar os comandos do Distronomicon e gerenciar releases do GitHub.</p>
      </section>

      <section className="grid">
        <div className="card formCard">
          <div className="cardTitle"><span>01</span><div><h2>Aplicação</h2><p>Informe o app e o repositório que deseja acompanhar.</p></div></div>
          <label>Nome da aplicação<input value={app} onChange={e => setApp(e.target.value)} placeholder="meu-app" /></label>
          <label>Repositório GitHub<input value={repo} onChange={e => setRepo(e.target.value)} placeholder="owner/repository" /></label>
        </div>

        <div className="card">
          <div className="cardTitle"><span>02</span><div><h2>Ação</h2><p>Escolha o que deseja fazer.</p></div></div>
          <div className="actions">
            <button className={action === 'check' ? 'selected' : ''} onClick={() => setAction('check')}><b>Verificar</b><small>Procura uma nova release sem instalar.</small></button>
            <button className={action === 'version' ? 'selected' : ''} onClick={() => setAction('version')}><b>Versão</b><small>Mostra a versão atualmente instalada.</small></button>
            <button className={action === 'update' ? 'selected danger' : ''} onClick={() => setAction('update')}><b>Atualizar</b><small>Baixa e inicia o processo de atualização.</small></button>
          </div>
        </div>
      </section>

      <section className="terminalCard">
        <div className="terminalHead"><div><span className="dot red"/><span className="dot yellow"/><span className="dot green"/></div><small>COMANDO GERADO</small></div>
        <code><span>$</span> {command}</code>
        <button onClick={copyCommand}>{copied ? 'Copiado ✓' : 'Copiar comando'}</button>
      </section>

      <section className="notice"><strong>Como funciona?</strong><p>Por segurança, um site hospedado no Vercel não pode executar diretamente comandos no seu Ubuntu/WSL. Este painel gera o comando correto para você copiar e executar no terminal onde o Distronomicon está instalado.</p></section>

      <footer>Distronomicon Desktop · interface para <a href="https://github.com/jtdowney/distronomicon" target="_blank" rel="noreferrer">jtdowney/distronomicon</a></footer>
    </main>
  );
}

# Distronomicon Desktop — Roadmap

Este documento registra ideias futuras que ainda não fazem parte da v1.4.0.

## Próximas ideias

1. **Fila de atualização de vários perfis** — verificar e atualizar vários aplicativos em sequência, com resultado individual.
2. **Modo Canary** — validar uma atualização em um perfil piloto antes de continuar com os demais.
3. **Restauração gráfica de backups** — listar e restaurar backups pré-update diretamente pela interface.
4. **Progresso avançado de download** — mostrar porcentagem, velocidade, bytes transferidos e ETA.
5. **Pacote de diagnóstico para suporte** — gerar um ZIP técnico com estado, histórico, Doctor e logs sanitizados.
6. **Baixar sem instalar** — pré-carregar uma release e aplicá-la depois.
7. **Janelas de manutenção** — definir dias e horários permitidos para updates automáticos.
8. **Proxy corporativo** — suporte explícito a proxy HTTP/HTTPS e configurações do Windows.
9. **Atualização delta** — usar patches menores quando o fornecedor disponibilizar esse formato, com fallback para pacote completo.
10. **Rotação e retenção de logs** — limitar tamanho, quantidade e idade dos logs técnicos.
11. **Comparador de release notes** — comparar versão atual e alvo e destacar mudanças relevantes.
12. **Painel de saúde dos perfis** — dashboard de versão, canal, arquitetura, última verificação, automação e espaço usado.
13. **Sigstore / cosign** — verificação opcional adicional de assinatura de artefatos.
14. **Rollback automático por crash pós-update** — restaurar a versão anterior se a nova versão falhar repetidamente após a atualização.

## Princípios

- Continuar nativo e portátil para Windows.
- Nenhuma dependência de WSL, Python, Node ou terminal para uso normal.
- A instalação ativa nunca deve ser corrompida por download ou update incompleto.
- Mudanças importantes devem ter recuperação ou rollback definido.
- Recursos novos devem passar por formatação, análise estática, testes e build Windows no CI.

## Status

A lista acima é planejamento futuro e ainda não está implementada na v1.4.0.

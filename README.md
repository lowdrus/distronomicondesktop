# Distronomicon Desktop

Versão desktop nativa e portátil para Windows inspirada no [Distronomicon](https://github.com/jtdowney/distronomicon), de John Downey.

## PT-BR

### Objetivo

O Distronomicon Desktop adapta para Windows o fluxo principal do Distronomicon original sem exigir WSL, terminal ou instalação. O usuário final utiliza apenas a interface gráfica e o executável portátil `DistronomiconDesktop.exe`.

### Recursos

- Interface gráfica leve e minimalista em Rust/egui
- Português do Brasil (PT-BR) e English (EN)
- Verificar atualização
- Atualizar/instalar releases
- Consultar versão instalada
- Desbloqueio forçado de atualização
- GitHub Releases
- ETag e Last-Modified para consultas condicionais
- Suporte opcional a GitHub token e repositórios privados
- GitHub Enterprise por host configurável
- Prereleases opcionais
- Seleção de asset por expressão regular
- Verificação SHA-256 por arquivo de checksums
- Download por streaming para evitar carregar releases inteiras na memória
- Lock exclusivo contra atualizações simultâneas
- Staging antes da instalação
- Troca segura do diretório `bin`
- Controle da versão ativa por `current.txt`
- Retenção configurável de versões anteriores; a versão atual nunca é removida
- Comando pós-atualização opcional, executado sem abrir terminal
- Proteções de extração ZIP contra path traversal, symlinks e arquivos excessivamente grandes

### Uso

1. Baixe o artefato `DistronomiconDesktop-Windows-Portable` quando o build estiver disponível.
2. Extraia o ZIP em qualquer pasta.
3. Abra `DistronomiconDesktop.exe`.
4. Informe o nome da aplicação e o repositório no formato `owner/repository`.
5. Use **Verificar**, **Atualizar** ou **Versão**.

Nenhum comando, WSL, Node.js, Python ou Rust é necessário para o usuário final.

### Estrutura portátil

Por padrão, os dados ficam próximos ao executável:

```text
DistronomiconDesktop.exe
managed/
  <app>/
    bin/
    releases/
    staging/
    current.txt
.distronomicon/
  <app>/
    state.json
    lock
    downloads/
```

### Opções avançadas

As opções avançadas ficam recolhidas por padrão para manter a interface simples. Elas permitem alterar diretórios, regex do asset/checksum, retenção, prereleases, token GitHub, host da API e comando pós-atualização.

A verificação SHA-256 fica habilitada por padrão. Desativá-la não é recomendado.

## English

Distronomicon Desktop is a lightweight native portable Windows release manager inspired by the original Distronomicon project. It provides a GUI for checking, installing and updating GitHub releases without WSL, terminals, or an installer.

It supports PT-BR/EN, conditional GitHub requests, private repositories through an optional token, GitHub Enterprise, prereleases, asset regex matching, SHA-256 verification, streaming downloads, exclusive update locking, staging, safe version switching, release retention and optional hidden post-update commands.

### End-user usage

Download the portable artifact, extract it, run `DistronomiconDesktop.exe`, enter an application name and `owner/repository`, then use **Check**, **Update**, or **Version**. No Rust/Cargo commands are required for end users.

## CI/CD

The repository contains `.github/workflows/build-windows.yml`, configured to build the native Windows portable executable and upload `DistronomiconDesktop-Windows-Portable`.

At the time of this revision, GitHub is creating workflow runs for this private repository but is terminating the hosted job before any step is assigned to a runner (`steps: []`, `runner_id: 0`). This is an account/runner provisioning condition rather than a build-step failure. The project source remains ready for the workflow once GitHub hosted runners are available to the repository.

## Compatibility note

The original Distronomicon is Linux-oriented and includes Unix-specific executable permissions and symlinks. The Windows port preserves the update semantics while replacing those mechanisms with native Windows file operations and an atomic-style `bin` directory swap. ZIP and raw Windows release assets are intentionally prioritized to keep the portable desktop small.

## Credits and license

Original project: [jtdowney/distronomicon](https://github.com/jtdowney/distronomicon), licensed under the MIT License.

See `THIRD_PARTY_NOTICES.md` for attribution.

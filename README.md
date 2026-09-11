# Distronomicon Desktop

Aplicativo gráfico nativo e portátil para Windows inspirado no projeto original [jtdowney/distronomicon](https://github.com/jtdowney/distronomicon).

## Objetivo

Oferecer a mesma ideia central do Distronomicon em uma interface Windows leve, sem WSL e sem terminal para o usuário final.

O executável final é `DistronomiconDesktop.exe` e não exige Rust, Cargo, Python, Node ou instalação para uso.

## Idiomas

- Português do Brasil (PT-BR)
- English (EN)

## Funções

- Check / Verificar atualização
- Update / Atualizar
- Version / Versão
- Unlock / Desbloquear
- GitHub Releases
- GitHub token opcional
- GitHub Enterprise/API host configurável
- ETag e Last-Modified
- Prereleases
- Seleção de asset por Regex
- SHA-256
- Download por streaming com retry/backoff para falhas transitórias
- Staging antes da instalação
- Troca segura do diretório `bin`
- Estado persistente em `state.json`
- Retenção configurável de releases antigas (padrão 3)
- A versão atual nunca é removida pelo pruning
- Restart opcional sem abrir terminal visível
- Lock para impedir duas atualizações simultâneas
- Proteções contra path traversal e archives maliciosos

## Formatos de archive

- `.zip`
- `.tar.gz`
- `.tgz`
- `.tar.bz2`
- `.tbz2`
- `.tar.xz`
- `.txz`
- `.tar.zst`

Assets não compactados, como `.exe`, também podem ser gerenciados diretamente.

## Layout portátil

Por padrão, os dados ficam ao lado do próprio executável:

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

## Uso

1. Abra `DistronomiconDesktop.exe`.
2. Informe o nome da aplicação.
3. Informe o repositório no formato `owner/repository`.
4. Clique em **Verificar** ou **Atualizar**.
5. Use **Opções avançadas** somente quando precisar configurar Regex, checksum, token, prerelease, retenção, pastas ou restart.

Não é necessário abrir PowerShell, CMD, WSL ou terminal.

## Cargo / Rust

Cargo e Rust são usados apenas pelo CI/CD e pelos desenvolvedores para transformar o código-fonte em `DistronomiconDesktop.exe`. Eles **não fazem parte dos requisitos do usuário final**.

## CI/CD

O workflow `.github/workflows/build-windows.yml` utiliza um runner Windows nativo para:

1. preparar Rust no ambiente temporário do GitHub Actions;
2. formatar o código;
3. executar Clippy;
4. executar testes;
5. compilar o build `release`;
6. gerar `DistronomiconDesktop.exe`;
7. criar `DistronomiconDesktop-Windows-Portable.zip`;
8. publicar o pacote como artifact do workflow.

Todo o uso de Cargo acontece dentro do runner de CI, não no computador do usuário.

## Projeto original

Distronomicon por jtdowney:
https://github.com/jtdowney/distronomicon

O projeto original é licenciado sob MIT. Consulte `THIRD_PARTY_NOTICES.md`.

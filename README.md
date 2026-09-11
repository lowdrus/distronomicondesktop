# Distronomicon Desktop

Aplicativo gráfico nativo e portátil para Windows inspirado no projeto original [jtdowney/distronomicon](https://github.com/jtdowney/distronomicon).

[![Windows CI and Portable Build](https://github.com/lowdrus/distronomicondesktop/actions/workflows/build-windows.yml/badge.svg)](https://github.com/lowdrus/distronomicondesktop/actions/workflows/build-windows.yml)
[![Latest Release](https://img.shields.io/github/v/release/lowdrus/distronomicondesktop?display_name=tag)](https://github.com/lowdrus/distronomicondesktop/releases/latest)

## Download

- **Última versão:** https://github.com/lowdrus/distronomicondesktop/releases/latest
- **Todas as versões:** https://github.com/lowdrus/distronomicondesktop/releases
- **Builds de CI:** https://github.com/lowdrus/distronomicondesktop/actions/workflows/build-windows.yml

Arquivos publicados em cada release:

- `DistronomiconDesktop-Windows-Portable.zip`
- `DistronomiconDesktop.exe`
- `SHA256SUMS.txt`

## Status atual

Versão atual do código: **v1.2.0**.

O executável final é um aplicativo gráfico Windows x86-64 compilado com CRT estático. Para uso normal não é necessário instalar Rust, Cargo, Python, Node, WSL, Ubuntu ou Visual C++ Redistributable adicional.

## Paridade com o Distronomicon original

O Desktop implementa a mesma finalidade central do projeto original: consultar releases no GitHub, selecionar assets, verificar checksum, instalar de forma segura, registrar a versão atual e manter releases anteriores.

| Recurso | Original Linux | Desktop Windows |
|---|---:|---:|
| Consultar última release | ✅ | ✅ |
| Atualizar para release mais recente | ✅ | ✅ |
| Mostrar versão instalada | ✅ | ✅ |
| Seleção de asset por Regex | ✅ | ✅ |
| SHA-256 | ✅ | ✅ |
| GitHub token | ✅ | ✅ |
| GitHub Enterprise/API host | ✅ | ✅ |
| Prereleases | ✅ | ✅ |
| ETag / Last-Modified | ✅ | ✅ |
| Retenção de releases antigas | ✅ | ✅ |
| Restart pós-update | ✅ | ✅ |
| Instalação atômica/staging | ✅ | ✅ |
| Lock contra updates simultâneos | ✅ | ✅ |
| Systemd timer | ✅ | Não aplicável ao Windows |
| CLI Linux | ✅ | Substituída por GUI Windows |
| Rollback | Planejado no upstream | ✅ |
| Doctor/diagnóstico | Planejado no upstream | ✅ |

Portanto, o Desktop já cobre o fluxo principal do Distronomicon, mas não é uma cópia byte-a-byte do programa Linux: integrações exclusivas do Linux, como `systemd`, foram substituídas por comportamento nativo de Windows.

## Como funciona

```text
GitHub Release
     ↓
Verificar release e asset
     ↓
Baixar para staging
     ↓
Verificar SHA-256
     ↓
Extrair de forma segura
     ↓
Criar release versionada
     ↓
Atualizar bin/current.txt
     ↓
Prunar versões antigas
     ↓
Restart opcional
```

## Uso passo a passo

### 1. Baixe

Abra a página de releases:

https://github.com/lowdrus/distronomicondesktop/releases/latest

Baixe `DistronomiconDesktop-Windows-Portable.zip`.

### 2. Extraia

Extraia o ZIP para qualquer pasta em que você tenha permissão de escrita, por exemplo:

```text
C:\DistronomiconDesktop\
```

### 3. Abra

Execute:

```text
DistronomiconDesktop.exe
```

Não é necessário abrir PowerShell, CMD, terminal ou WSL.

### 4. Informe a aplicação

No campo **Aplicação**, use um identificador simples, por exemplo:

```text
meu-app
```

Esse nome é usado para organizar as pastas locais de releases e estado.

### 5. Informe o repositório

No campo **Repositório GitHub**, use:

```text
owner/repository
```

Exemplo:

```text
lowdrus/distronomicondesktop
```

### 6. Clique em Verificar

**Verificar / Check** consulta a release mais recente sem instalar nada.

Resultados possíveis:

- atualizado;
- atualização disponível;
- instalação disponível;
- erro de autenticação/repositório.

### 7. Clique em Atualizar

**Atualizar / Update** executa o fluxo completo:

1. consulta a release;
2. seleciona o asset pelo Regex configurado;
3. baixa o arquivo;
4. valida SHA-256, salvo quando a verificação foi explicitamente desativada;
5. extrai para staging;
6. promove para `releases/<tag>`;
7. atualiza `bin` e `current.txt`;
8. remove releases antigas conforme a retenção;
9. executa restart opcional.

## Função por função

### Verificar / Check

Consulta o GitHub e informa se existe uma versão mais recente. Não modifica a instalação.

### Atualizar / Update

Baixa e instala a release mais recente de forma versionada e segura.

### Versão / Version

Mostra a release atualmente ativa.

### Rollback

Volta para a release instalada imediatamente anterior sem precisar baixar novamente. O `bin` e o `current.txt` são atualizados para a versão restaurada.

### Diagnóstico / Doctor

Verifica a integridade básica da instalação local:

- `current.txt`;
- diretório da release atual;
- diretório `bin`;
- divergência entre `state.json` e `current.txt`;
- quantidade de releases instaladas.

### Forçar desbloqueio / Force unlock

Remove o arquivo de lock quando uma execução anterior foi interrompida e deixou o lock preso.

## Opções avançadas

### Pasta de instalação

Diretório onde ficam as releases gerenciadas.

### Pasta de estado

Diretório usado para `state.json`, lock e downloads temporários.

### Padrão do asset

Regex usado para selecionar o asset correto da GitHub Release.

Padrão inicial:

```text
(?i).*\.(zip|exe)$
```

### Padrão do checksum

Regex usado para localizar o arquivo SHA-256 da release.

### Pular verificação SHA-256

Desativa a validação de checksum. Deve ser usado somente quando a release não fornece checksum confiável.

### Permitir prereleases

Inclui releases marcadas como prerelease.

### Manter releases recentes

Define quantas releases versionadas ficam armazenadas. A versão atual nunca é removida pelo pruning.

### GitHub API host

Permite usar GitHub Enterprise ou endpoint compatível.

### GitHub token

Opcional para repositórios públicos. Útil para repositórios privados ou maior limite de API.

### Comando pós-update/rollback

Executa um comando opcional após ativar a versão nova ou restaurada.

## Estrutura de arquivos

```text
DistronomiconDesktop.exe
managed/
  <app>/
    bin/
    releases/
      <tag>/
    staging/
    current.txt
.distronomicon/
  <app>/
    state.json
    lock
    downloads/
```

## Formatos suportados

- `.zip`
- `.tar.gz`
- `.tgz`
- `.tar.bz2`
- `.tbz2`
- `.tar.xz`
- `.txz`
- `.tar.zst`
- assets não compactados, como `.exe`

A extração possui proteções contra path traversal, links simbólicos indevidos, quantidade excessiva de arquivos, arquivos gigantes e razão de descompressão abusiva.

## Prints / guia visual

Os prints oficiais devem corresponder exatamente à versão publicada. A cada mudança visual relevante, os arquivos em `docs/screenshots/` devem ser atualizados junto com o README antes de publicar a release.

Fluxo visual esperado:

1. tela inicial;
2. preenchimento de Aplicação e Repositório;
3. resultado de Verificar;
4. execução de Atualizar;
5. tela/status após atualização;
6. Rollback;
7. Doctor;
8. Opções avançadas.

> Observação: screenshots reais serão adicionados ao repositório somente quando capturados da build correspondente, para evitar documentação visual falsa ou divergente do executável.

## CI/CD e validação

O workflow `.github/workflows/build-windows.yml` executa em Windows nativo:

1. checkout;
2. Rust stable;
3. `cargo fmt`;
4. Clippy com `-D warnings`;
5. testes;
6. build release com CRT estático;
7. criação do ZIP portátil;
8. validação da assinatura `MZ` do executável;
9. geração de SHA-256;
10. upload de artifact;
11. publicação de GitHub Release em tags `v*`.

Cargo e Rust existem apenas no ambiente de desenvolvimento/CI. O usuário final recebe somente o executável e arquivos de documentação.

## Política de atualização do README

Sempre que o Distronomicon Desktop for alterado:

1. atualizar a versão quando apropriado;
2. atualizar este README;
3. revisar a tabela de paridade;
4. atualizar o tutorial e a lista de funções;
5. atualizar screenshots reais quando a interface mudar;
6. manter os links de download apontando para `releases/latest` e `releases`;
7. validar `main` e `native-windows-port` antes da publicação.

## Projeto original

Distronomicon por jtdowney:

https://github.com/jtdowney/distronomicon

O projeto original é licenciado sob MIT. Consulte `THIRD_PARTY_NOTICES.md`.

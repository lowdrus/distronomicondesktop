# Distronomicon Desktop

Gerenciador gráfico nativo e portátil para Windows que monitora GitHub Releases, baixa e valida atualizações por SHA-256, instala releases de forma versionada e segura, oferece rollback, diagnóstico, retenção, prereleases, GitHub Enterprise, perfis, automação, histórico, health check e gerenciamento sem terminal ou WSL.

[![Windows CI](https://github.com/lowdrus/distronomicondesktop/actions/workflows/build-windows.yml/badge.svg)](https://github.com/lowdrus/distronomicondesktop/actions/workflows/build-windows.yml)
[![Latest Release](https://img.shields.io/github/v/release/lowdrus/distronomicondesktop?display_name=tag)](https://github.com/lowdrus/distronomicondesktop/releases/latest)

### Downloads rápidos

- **[DistronomiconDesktop-Latest](https://github.com/lowdrus/distronomicondesktop/releases/latest)**
- **[Todas Versões - Releases](https://github.com/lowdrus/distronomicondesktop/releases)**
- **[DistronomiconDesktop BUILDS DE CI](https://github.com/lowdrus/distronomicondesktop/actions/workflows/build-windows.yml)**

As releases usam título curto no GitHub: `DISTROD-vX.Y.Z`.

Arquivos publicados em cada release:

- `DistronomiconDesktop-Windows-Portable.zip`
- `DistronomiconDesktop.exe`
- `SHA256SUMS.txt`

## Status

Versão desta revisão: **v1.3.1**.

A v1.3.0 introduziu as 10 grandes melhorias funcionais. A v1.3.1 é uma revisão de estabilidade: reforça recuperação de metadados, perfis e histórico, melhora Dry Run e version pinning e incorpora o screenshot real na documentação.

O executável final é Windows x86-64 com CRT estático. O usuário não precisa instalar Rust, Cargo, Python, Node, WSL, Ubuntu ou Visual C++ Redistributable adicional.

A interface possui **PT-BR / EN**, tema **Dark / Light** com acento dourado inspirado no macOS Golden Gate e ícone próprio no `.exe`.

## Screenshot real

![Distronomicon Desktop](assets/distronomicon-desktop-example.png)

A imagem acima é um screenshot real da aplicação Windows. O arquivo SVG continua no repositório apenas como ilustração vetorial auxiliar; o PNG é a referência visual principal.

## O que o Distronomicon Desktop faz

O Distronomicon Desktop consulta releases de qualquer repositório GitHub válido, escolhe o asset mais adequado ao Windows/x64 dentro do filtro configurado, baixa com retry e retomada, verifica SHA-256, extrai com proteções de segurança, instala em diretórios versionados, ativa a nova versão, mantém versões anteriores conforme a retenção, permite rollback e registra estado e histórico.

O campo **Repositório GitHub** aceita:

```text
owner/repository
```

ou uma URL completa:

```text
https://github.com/owner/repository
```

Também aceita `github.com/owner/repository`, barra final e sufixo `.git`. A entrada é normalizada internamente para `owner/repository` antes da chamada à API. Não há vínculo com `lowdrus`: qualquer repositório GitHub válido pode ser usado.

## Função por função

### Aplicação

É um nome local escolhido pelo usuário para identificar o programa gerenciado. Exemplos:

```text
distronomicondesktop
cinetwitch
maginarium
meu-servidor
```

Esse nome cria uma área separada em `managed/<aplicacao>/` e `.distronomicon/<aplicacao>/`. Nomes inválidos para Windows, nomes reservados como `CON`, `AUX`, `NUL`, `COM1` e caracteres proibidos são rejeitados antes da instalação.

### Perfil

Salva persistentemente as configurações de uma aplicação. É possível cadastrar vários programas e alternar entre eles pelo ComboBox sem redigitar repositório, Regex, pastas, retenção, pinning, health check ou automação. Tokens GitHub não são gravados no perfil.

Na v1.3.1, `profiles.json` também usa recuperação pelo arquivo de backup se o principal estiver ausente ou corrompido.

### Verificar / Check

Consulta a release alvo e informa se existe atualização. Não instala nada.

### Prévia / Dry Run

Mostra sem alterar o disco:

- versão atual;
- versão alvo;
- versão fixada, se houver;
- asset selecionado;
- arquivo de checksum;
- destino da instalação;
- versões que a política de retenção removerá depois da atualização;
- se nenhuma instalação será necessária porque a versão alvo já está ativa.

### Atualizar / Update

Executa o fluxo completo:

1. obtém a release;
2. respeita prerelease/version pinning;
3. escolhe o asset Windows adequado;
4. baixa com retry e retomada HTTP Range;
5. verifica SHA-256;
6. extrai para staging;
7. promove a release para `releases/<tag>`;
8. troca `bin` com rollback de filesystem em caso de falha;
9. atualiza `current.txt` e `state.json` com persistência reforçada;
10. executa restart opcional;
11. executa health check opcional;
12. volta automaticamente à versão anterior se o health check falhar;
13. aplica retenção;
14. grava histórico.

### Versão / Version

Mostra a release atualmente ativa. Se `current.txt` estiver ausente ou inválido, a v1.3.1 tenta recuperar a tag pelo `current.txt.bak` e valida se a release correspondente existe.

### Reverter / Rollback

Volta para a release instalada anterior sem novo download e registra a operação no histórico.

### Diagnóstico / Doctor

Verifica coerência entre `current.txt`, `state.json`, release ativa, diretório `bin` e releases armazenadas.

### Histórico / History

Exibe as operações recentes, incluindo update/rollback, versão anterior/nova, asset, SHA-256 e resultado. Na v1.3.1, `history.json` também recupera o backup quando necessário.

### Forçar desbloqueio / Force unlock

O lock impede duas atualizações simultâneas na mesma aplicação. Use **Forçar desbloqueio** somente se uma atualização anterior foi interrompida e deixou o lock preso. No uso normal não é necessário.

### PT-BR / EN

O seletor troca a interface entre Português do Brasil e Inglês. Botões, rótulos e mensagens operacionais da v1.3 usam o mesmo sistema bilíngue.

### Dark / Light

O controle `◐` alterna os temas mantendo a identidade visual dourada.

## As 10 melhorias implementadas na v1.3

### 1. Perfis persistentes

Cada perfil armazena aplicação, repositório, Regex, diretórios, prerelease, retenção, restart, health check, pin de versão e automação. Os perfis ficam próximos ao executável em `.distronomicon/profiles.json` e usam escrita protegida por temporário, `sync_all`, backup e rename.

### 2. Auto Check / Auto Update Windows

A GUI registra uma tarefa no **Windows Task Scheduler** sem abrir terminal visível. O perfil pode executar:

- `check`: apenas verificar;
- `update`: verificar e atualizar;
- intervalo configurável em minutos.

A execução agendada reutiliza o mesmo `.exe` em modo silencioso e grava log em `.distronomicon/scheduled/`.

### 3. Dry Run

O botão **Prévia** permite revisar o plano antes da atualização e, na v1.3.1, também mostra o efeito previsto da retenção.

### 4. Health Check + rollback automático

Depois da atualização, um comando opcional pode validar o aplicativo. Se falhar, o Desktop reativa a versão anterior, restaura o estado e executa novamente o restart quando configurado.

### 5. Version Pinning

O perfil pode permanecer em `latest` ou apontar para uma tag específica. A v1.3.1 aceita, por exemplo:

```text
1.3.0
v1.3.0
release/1.3
```

Quando o usuário informa `1.3.0`, o Desktop tenta também `v1.3.0`. Tags com caracteres como `/` são codificadas corretamente para a API.

### 6. Histórico de updates/rollbacks

Cada aplicação mantém `history.json`, limitado às entradas mais recentes, com versão anterior/nova, asset, SHA-256, resultado e timestamp. O arquivo possui recuperação via backup.

### 7. Seleção inteligente de asset

Depois de aplicar a Regex configurada, os candidatos recebem prioridade para nomes que indiquem:

- `windows`, `win64` ou `win-`;
- `x86_64`, `x64` ou `amd64`;
- `.zip` / `.exe`;
- `portable`.

Assets Linux/macOS e arquiteturas incompatíveis recebem penalidade. Uma Regex explícita do usuário continua sendo o filtro principal.

### 8. Auto-update do Distronomicon Desktop

O aplicativo consulta a própria release, baixa `DistronomiconDesktop.exe`, valida `SHA256SUMS.txt`, prepara a troca e reinicia sem janela de terminal visível. O mecanismo impede downgrade automático para uma versão mais antiga.

### 9. Downloads retomáveis

Arquivos parciais são retomados com HTTP `Range` quando o servidor devolve `206 Partial Content`; se a retomada não for aceita, o download recomeça do zero com segurança.

### 10. Persistência reforçada

`state.json`, `current.txt`, perfis e histórico usam escrita temporária + `sync_all` + backup + rename. A leitura tenta o backup quando o arquivo principal estiver ausente/corrompido. A tag recuperada de `current.txt.bak` só é aceita se a release correspondente realmente existir.

## Paridade com o Distronomicon Linux

O projeto linux DISTRONOMICON é uma ferramenta Linux que consulta GitHub Releases e executa atualizações atômicas. O Distronomicon Desktop preserva esse núcleo e substitui integrações exclusivas de Linux por equivalentes apropriados para Windows.

| Recurso | Distronomicon Linux | Desktop Windows |
|---|---:|---:|
| Consultar GitHub Releases | ✅ | ✅ |
| Check / Update / Version / Unlock | ✅ | ✅ |
| Regex de asset | ✅ | ✅ |
| SHA-256 | ✅ | ✅ |
| GitHub token | ✅ | ✅ |
| GitHub Enterprise/API host | ✅ | ✅ |
| Prereleases e drafts | ✅ | ✅ |
| ETag / Last-Modified | ✅ | ✅ |
| Retry/backoff | ✅ | ✅ |
| Timeout HTTP | ✅ | ✅ |
| Lock + timeout | ✅ | ✅ |
| Staging | ✅ | ✅ |
| Retenção | ✅ | ✅ |
| Restart | ✅ | ✅ |
| ZIP/TAR comprimidos | ✅ | ✅ |
| Proteções de extração | ✅ | ✅ |
| Persistência com recuperação | ✅ | ✅ |
| Version pinning | — | ✅ |
| Download retomável | — | ✅ |
| Perfis | — | ✅ |
| Histórico | — | ✅ |
| Health check + auto rollback | — | ✅ |
| Auto-update do próprio app | — | ✅ |
| GUI PT-BR / EN | ❌ | ✅ |
| Dark / Light | ❌ | ✅ |
| EXE portátil | ❌ | ✅ |
| systemd timer | ✅ | Task Scheduler |
| CLI Linux | ✅ | GUI + modo agendado interno |

### O que não tem pariedade literalmente

- `systemd/`: no Windows é substituído pelo Task Scheduler opcional;
- symlinks/permissões POSIX: o Desktop usa releases versionadas, `current.txt` e espelhamento de `bin` por hard links/cópia;
- CLI Linux: o uso normal é totalmente gráfico; argumentos internos existem apenas para automação agendada.

## Segurança da instalação

A extração suporta:

- `.zip`
- `.tar.gz` / `.tgz`
- `.tar.bz2` / `.tbz2`
- `.tar.xz` / `.txz`
- `.tar.zst`
- assets não compactados, como `.exe`

Há proteção contra path traversal, symlinks indevidos, excesso de arquivos, tamanho excessivo e razão de descompressão abusiva. Archives com uma única pasta raiz são normalizados.

## Estrutura portátil

```text
DistronomiconDesktop.exe
managed/
  <app>/
    bin/
    releases/
      <tag>/
    staging/
    current.txt
    current.txt.bak
.distronomicon/
  profiles.json
  profiles.json.bak
  <app>/
    state.json
    state.json.bak
    history.json
    history.json.bak
    lock
    downloads/
  scheduled/
  self-update/
```

## Uso passo a passo

1. Baixe **DistronomiconDesktop-Windows-Portable.zip** na última release.
2. Extraia para uma pasta com permissão de escrita.
3. Execute `DistronomiconDesktop.exe`.
4. Escolha ou crie um **Perfil**.
5. Informe **Aplicação**.
6. Cole `owner/repository` ou a URL completa do GitHub.
7. Clique **Verificar**.
8. Use **Prévia** para revisar a atualização e o efeito da retenção.
9. Clique **Atualizar**.
10. Confirme em **Versão** e **Diagnóstico**.
11. Se necessário, use **Reverter**.

Nenhum desses passos exige CMD, PowerShell, WSL ou Cargo.

## CI/CD e Releases

O workflow Windows valida:

1. `Cargo.lock`;
2. `rustfmt`;
3. Clippy com `-D warnings`;
4. testes;
5. build MSVC release com CRT estático;
6. pacote ZIP;
7. assinatura `MZ` do executável;
8. ícone embutido;
9. artifact do CI;
10. `SHA256SUMS.txt`;
11. GitHub Release.

A release publicada recebe título curto `DISTROD-vX.Y.Z`.

## Sobre commits antigos com erro

Commits históricos com check vermelho representam estados antigos e não quebram a release atual. O critério de distribuição é: head atual verde, testes/build verdes e release produzida desse estado. Reescrever todo o histórico apenas para esconder falhas antigas não é recomendado.

## Próximas ideias

Além das 10 melhorias já implementadas, fazem sentido para versões futuras:

- notificações nativas do Windows;
- assinatura Authenticode/Sigstore quando aplicável;
- retenção também por espaço em disco ou dias;
- canais Stable/Beta/Nightly por perfil;
- exportar/importar perfis;
- abrir pasta da versão ativa e página da release;
- botão de cancelamento de download;
- estimativa de espaço necessário antes do update;
- proteção contra downgrade manual acidental;
- release notes dentro da GUI;
- mirrors/fontes adicionais além do GitHub;
- ARM64 quando houver demanda;
- teste de conectividade antes de operações longas;
- reconstrução guiada de estado no Doctor;
- notificações de update concluído/falhou na central do Windows.

## Projeto de Linux Para Nativo

**by: lowdrus, canadian192, lincolhalles.**

O Distronomicon Desktop é uma adaptação Windows nativa e portátil construída para preservar os conceitos centrais do projeto Linux e oferecer uma experiência gráfica sem terminal ou WSL.

O projeto original é licenciado sob MIT. A atribuição jurídica e o texto da licença permanecem em [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

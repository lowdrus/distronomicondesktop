# Distronomicon Desktop

Aplicativo gráfico nativo e portátil, mais informações abaixo...

[![Windows CI](https://github.com/lowdrus/distronomicondesktop/actions/workflows/build-windows.yml/badge.svg)](https://github.com/lowdrus/distronomicondesktop/actions/workflows/build-windows.yml)
[![Latest Release](https://img.shields.io/github/v/release/lowdrus/distronomicondesktop?display_name=tag)](https://github.com/lowdrus/distronomicondesktop/releases/latest)

### Downloads rápidos

- **[DistronomiconDesktop-Latest](https://github.com/lowdrus/distronomicondesktop/releases/latest)**
- **[Todas Versões - Releases](https://github.com/lowdrus/distronomicondesktop/releases)**
- **[DistronomiconDesktop BUILDS DE CI](https://github.com/lowdrus/distronomicondesktop/actions/workflows/build-windows.yml)**

Arquivos publicados em cada release:

- `DistronomiconDesktop-Windows-Portable.zip`
- `DistronomiconDesktop.exe`
- `SHA256SUMS.txt`

## Status atual

Versão atual do código: **v1.2.0**.

O executável é um aplicativo gráfico Windows x86-64 compilado com CRT estático. Para uso normal não é necessário instalar Rust, Cargo, Python, Node, WSL, Ubuntu ou Visual C++ Redistributable adicional.

A interface possui **PT-BR / EN**, modo **Dark / Light** com acento visual dourado inspirado na estética macOS Golden Gate e ícone próprio embutido no `.exe`.

Os builds de desenvolvimento são reproduzíveis com `Cargo.lock` versionado; o usuário final continua recebendo apenas o `.exe`/ZIP portátil.

## Paridade com o Distronomicon original

O projeto original [`jtdowney/distronomicon`](https://github.com/jtdowney/distronomicon) é uma ferramenta Linux que consulta GitHub Releases e executa atualizações atômicas. O Distronomicon Desktop preserva esse núcleo e substitui integrações exclusivas de Linux por equivalentes apropriados para Windows.

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
| Retry/backoff em download | ✅ | ✅ |
| Retenção de releases antigas | ✅ | ✅ |
| Restart pós-update | ✅ | ✅ |
| Staging antes da ativação | ✅ | ✅ |
| Estado persistente | ✅ | ✅ |
| Lock contra updates simultâneos | ✅ | ✅ |
| ZIP/TAR e formatos comprimidos | ✅ | ✅ |
| Proteções de extração | ✅ | ✅ |
| Systemd timer | ✅ | Não se aplica diretamente ao Windows |
| CLI Linux | ✅ | Substituída por GUI Windows |
| Rollback | Planejado no upstream | ✅ |
| Doctor/diagnóstico | Planejado no upstream | ✅ |
| GUI PT-BR / EN | ❌ | ✅ |
| Dark / Light | ❌ | ✅ |
| EXE portátil | ❌ | ✅ |

### O que não deve ser copiado literalmente

- `systemd/`: é específico de Linux; no Windows a evolução planejada é um agendamento nativo opcional.
- symlinks POSIX e permissões Unix: o Desktop usa `current.txt`, releases versionadas e troca segura do diretório `bin`.
- CLI/variáveis de ambiente: a configuração principal foi movida para a interface gráfica.

## Como funciona

```text
GitHub Release
     ↓
Verificar release + ETag/Last-Modified
     ↓
Selecionar asset por Regex
     ↓
Download com retry/backoff
     ↓
Verificar SHA-256
     ↓
Extrair para staging com proteções
     ↓
Promover para releases/<tag>
     ↓
Atualizar bin + current.txt
     ↓
Salvar state.json
     ↓
Remover versões antigas conforme retenção
     ↓
Restart opcional
```

## Uso passo a passo

### 1. Baixe

Abra **[DistronomiconDesktop-Latest](https://github.com/lowdrus/distronomicondesktop/releases/latest)** e baixe `DistronomiconDesktop-Windows-Portable.zip`.

### 2. Extraia

Extraia o ZIP para qualquer pasta em que você tenha permissão de escrita, por exemplo:

```text
C:\DistronomiconDesktop\
```

### 3. Abra

Execute `DistronomiconDesktop.exe`. Não é necessário abrir PowerShell, CMD, terminal ou WSL.

### 4. Aplicação — o que colocar em `myapp` / `meu-app`?

**Aplicação é apenas um nome local que você escolhe para identificar o programa que será gerenciado.**

Exemplos:

```text
cinetwitch
meu-servidor
app-financeiro
maginarium
```

Esse nome **não precisa ser o nome exato do repositório GitHub**. Ele serve para o Distronomicon Desktop organizar separadamente:

```text
managed/<aplicacao>/releases/
managed/<aplicacao>/bin/
managed/<aplicacao>/current.txt
.distronomicon/<aplicacao>/state.json
```

Exemplo prático:

```text
Aplicação: cinetwitch
Repositório: lowdrus/watch-monitor
```

### 5. Repositório GitHub

Use o formato:

```text
owner/repository
```

Exemplo:

```text
lowdrus/distronomicondesktop
```

### 6. Verificar

**Verificar / Check** consulta a última release sem instalar nada.

### 7. Atualizar

**Atualizar / Update**:

1. consulta a release;
2. seleciona o asset;
3. baixa com retry;
4. valida SHA-256;
5. extrai para staging;
6. promove a release;
7. atualiza `bin` e `current.txt`;
8. salva estado;
9. aplica retenção;
10. executa restart opcional.

## Função por função

### Verificar / Check
Consulta o GitHub e informa se existe uma versão mais recente. Não modifica a instalação.

### Atualizar / Update
Baixa, verifica e instala a release mais recente de forma versionada e segura.

### Versão / Version
Mostra a release atualmente ativa.

### Rollback
Volta para a release instalada imediatamente anterior sem novo download e atualiza `bin` + `current.txt`.

### Diagnóstico / Doctor
Verifica `current.txt`, release ativa, `bin`, `state.json` e quantidade de releases instaladas.

### Aplicação
É o nome local escolhido pelo usuário para identificar e separar o programa gerenciado. Não é senha, caminho ou comando. Exemplos: `cinetwitch`, `maginarium`, `meu-servidor`.

### Forçar desbloqueio / Force unlock
Durante uma atualização, o Desktop cria um **lock** para impedir duas atualizações simultâneas sobre a mesma aplicação.

Normalmente esse lock é removido automaticamente ao terminar. **Forçar desbloqueio** só deve ser usado quando uma execução foi interrompida abruptamente — queda de energia, encerramento forçado ou crash — e o lock ficou preso.

No uso normal você não precisa clicar nesse botão.

### Dark / Light
O controle `◐` no topo alterna imediatamente entre tema escuro e claro. Ambos mantêm o acento dourado da identidade visual.

### PT-BR / EN
O seletor no topo troca a interface entre Português do Brasil e Inglês.

## Opções avançadas

- **Pasta de instalação:** onde ficam `releases`, `bin`, `staging` e `current.txt`.
- **Pasta de estado:** onde ficam `state.json`, lock e downloads temporários.
- **Padrão do asset:** Regex para selecionar o arquivo correto da release.
- **Padrão do checksum:** Regex para selecionar o arquivo de hashes SHA-256.
- **Pular SHA-256:** use somente quando a release não fornece checksum confiável.
- **Permitir prereleases:** inclui versões marcadas como prerelease.
- **Manter releases recentes:** quantidade de releases armazenadas; a atual nunca é removida.
- **GitHub API host:** permite GitHub Enterprise/API compatível.
- **GitHub token:** opcional para público; útil para privados e maior limite de API.
- **Comando pós-update/rollback:** comando opcional executado sem abrir console visível.

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

A extração possui proteção contra path traversal, symlinks indevidos, arquivos excessivos, tamanho excessivo e razão de descompressão abusiva.

## Guia visual da interface

A interface segue este fluxo:

```text
┌──────────────────────────────────────────────────────────────┐
│ Distronomicon Desktop                         PT-BR   ◐      │
│ Gerenciador de releases portátil e nativo para Windows      │
├──────────────────────────────────────────────────────────────┤
│ Aplicação                                                   │
│ [ cinetwitch___________________________________________ ]    │
│ Nome local usado para organizar a aplicação                 │
│                                                             │
│ Repositório GitHub                                          │
│ [ lowdrus/watch-monitor________________________________ ]   │
│                                                             │
│ [Verificar] [Atualizar] [Versão] [Rollback] [Diagnóstico]   │
│                                                             │
│ ▸ Opções avançadas                                          │
├──────────────────────────────────────────────────────────────┤
│ Status                                                      │
│ Atualização disponível: v1.1.0 → v1.2.0                    │
└──────────────────────────────────────────────────────────────┘
```

### Etapa 1 — configurar
Preencha **Aplicação** e **Repositório GitHub**.

### Etapa 2 — verificar
Clique em **Verificar**. Nada é instalado nessa etapa.

### Etapa 3 — atualizar
Clique em **Atualizar** para executar download, checksum, staging e ativação.

### Etapa 4 — confirmar
Use **Versão** e **Diagnóstico** para confirmar a instalação ativa.

### Etapa 5 — recuperar
Use **Rollback** se quiser voltar à release anterior já armazenada.

> A documentação visual deve sempre acompanhar a interface real da versão publicada. Quando houver mudanças importantes de layout, este guia deve ser atualizado no mesmo commit.

## CI/CD e Releases

O workflow Windows executa:

1. checkout;
2. Rust stable;
3. geração/validação do `Cargo.lock`;
4. normalização de `rustfmt` quando necessário no `main`;
5. Clippy `-D warnings`;
6. testes;
7. build release com CRT estático;
8. geração e validação do ícone nativo via `build.rs`;
9. ZIP portátil;
10. validação da assinatura Windows `MZ`;
11. SHA-256;
12. artifact de CI;
13. criação/atualização automática da GitHub Release correspondente à versão do `Cargo.toml` quando `main` fica verde.

Cargo/Rust são apenas ferramentas de desenvolvimento e CI. O usuário final recebe o `.exe` portátil.

## Política obrigatória de atualização

Sempre que o Distronomicon Desktop for alterado:

1. atualizar a versão quando necessário;
2. atualizar o README;
3. revisar a tabela de paridade;
4. atualizar tutorial/funções;
5. atualizar o guia visual quando a GUI mudar;
6. manter **DistronomiconDesktop-Latest**, **Todas Versões - Releases** e **BUILDS DE CI** no topo;
7. validar `main` e `native-windows-port`;
8. publicar/atualizar a Release permanente.

## O que ainda podemos melhorar

- agendamento automático nativo do Windows, equivalente ao papel do `systemd timer`;
- configuração persistente de perfis/aplicações;
- histórico de atualizações e rollbacks;
- dry-run para visualizar alterações antes de instalar;
- version pinning;
- health check pós-update;
- auto-update do próprio Distronomicon Desktop;
- múltiplas fontes além de GitHub Releases;
- assinaturas criptográficas adicionais (Sigstore/GPG/BLAKE3/SHA-512);
- downloads retomáveis e mirrors.

## Projeto original e licença

Distronomicon por jtdowney: https://github.com/jtdowney/distronomicon

O projeto original é licenciado sob MIT. Consulte `THIRD_PARTY_NOTICES.md`.

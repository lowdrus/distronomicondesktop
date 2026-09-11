# Distronomicon Desktop

Interface gráfica portátil e leve para Windows para controlar o [Distronomicon](https://github.com/jtdowney/distronomicon) instalado no Ubuntu/WSL.

## Status

Versão desktop em desenvolvimento ativo, com build automático para Windows via GitHub Actions.

## Como funciona

O Distronomicon original é uma ferramenta Linux. O Distronomicon Desktop roda como aplicativo Windows e utiliza o `wsl.exe` para executar os comandos do Distronomicon dentro da distribuição Ubuntu.

### Funções da interface

- Informar o nome da aplicação
- Informar o repositório GitHub no formato `owner/repository`
- Verificar atualizações
- Consultar a versão instalada
- Executar atualização
- Exibir a saída dos comandos diretamente na interface

## Requisitos para uso

- Windows 10 ou Windows 11
- WSL 2
- Ubuntu instalado no WSL
- Distronomicon instalado e funcionando dentro do Ubuntu

O executável da interface é portátil e não requer instalador.

## Desenvolvimento

Projeto escrito em Rust usando `eframe/egui`.

```powershell
cargo build --release
```

O executável local será criado em:

```text
target\release\DistronomiconDesktop.exe
```

## CI/CD

O workflow `.github/workflows/build-windows.yml` compila automaticamente a versão portátil no GitHub Actions.

Em pushes para `main` e execuções manuais do workflow, o pipeline gera o artefato:

```text
DistronomiconDesktop-Windows-Portable
```

contendo:

```text
DistronomiconDesktop.exe
```

## Executando

1. Certifique-se de que o Ubuntu/WSL e o Distronomicon estão funcionando.
2. Baixe o artefato produzido pelo GitHub Actions.
3. Extraia o ZIP.
4. Execute `DistronomiconDesktop.exe`.

## Projeto original

Distronomicon por jtdowney:
https://github.com/jtdowney/distronomicon

# Deploy2Monster

Aplicativo de console para auxiliar no deploy de aplicacoes Blazor e/ou ASP.NET para hospedagens MonsterASP.

O objetivo do projeto e simplificar tarefas comuns de publicacao, como criar um arquivo de configuracao do projeto e executar o processo de deploy a partir da linha de comando.

## Status

Projeto em fase inicial.

Comandos existentes:

```powershell
deploy2monster -new
deploy2monster -deploy
deploy2monster -help
```

## Requisitos

- Rust instalado
- Cargo disponivel no terminal

## Como executar

Durante o desenvolvimento:

```powershell
cargo run -- -help
```

Para gerar o binario:

```powershell
cargo build --release
```

O executavel sera gerado em:

```text
target/release/deploy2monster.exe
```

## Comandos

- `-new`: cria um novo arquivo de projeto, futuramente usado para armazenar as configuracoes de deploy.
- `-deploy`: executa o deploy da aplicacao conforme as configuracoes do projeto.
- `-help`: exibe a ajuda do aplicativo.

## Objetivo

O Deploy2Monster pretende servir como uma ferramenta simples e direta para publicar aplicacoes .NET no MonsterASP, evitando repeticao manual de passos de build, empacotamento e envio.

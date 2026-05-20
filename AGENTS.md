# AGENTS.md

## Contexto do projeto

Deploy2Monster e um aplicativo de console escrito em Rust. Ele deve auxiliar no deploy de aplicacoes Blazor e/ou ASP.NET para o MonsterASP.

O projeto ainda esta no inicio. Preserve a simplicidade da CLI e evite introduzir dependencias sem necessidade clara.

## Comandos uteis

```powershell
cargo run -- -help
cargo run -- -new
cargo run -- -deploy
cargo test
cargo build --release
```

## Convencoes

- Mantenha mensagens e documentacao em portugues.
- Prefira implementacoes simples e explicitas.
- Evite refatoracoes grandes sem necessidade.
- Antes de alterar comportamento da CLI, atualize tambem o `README.md`.
- Se novos comandos forem adicionados, documente o uso esperado e exemplos.

## Direcionamento tecnico

- A ferramenta deve continuar funcionando bem em Windows, ja que o publico provavel inclui desenvolvedores .NET.
- Ao lidar com arquivos de configuracao, prefira formatos estruturados como JSON, TOML ou YAML.
- Ao implementar deploy real, separe claramente as etapas de build, empacotamento, validacao e envio.
- Nao armazene credenciais sensiveis diretamente em arquivos versionados.

## Escopo esperado

Funcionalidades futuras provaveis:

- Criar arquivo de configuracao do projeto.
- Ler configuracoes de deploy.
- Executar publicacao de projetos .NET.
- Compactar artefatos de publicacao.
- Enviar arquivos para o ambiente MonsterASP.
- Validar configuracoes antes do deploy.

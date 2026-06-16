# Deploy2Monster

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange)](https://www.rust-lang.org/)
![CLI](https://img.shields.io/badge/Interface-Console-blue)
![Windows](https://img.shields.io/badge/Windows-first-0078D4)

Ferramenta de console em Rust para apoiar o deploy de aplicações Blazor e ASP.NET para hospedagens MonsterASP.

O objetivo do projeto é reduzir tarefas repetitivas de publicação, centralizar as configurações por projeto e deixar o fluxo de deploy mais previsível no Windows.

## Destaques

- foco em Windows
- CLI simples e direta
- configuração por projeto em JSON
- senhas protegidas localmente
- suporte a exportação e importação de projetos

## Estado atual

O projeto já possui uma CLI funcional para:

- criar e editar projetos de deploy
- listar, exportar, importar e remover projetos cadastrados
- testar conexões FTP e banco de dados
- executar `dotnet publish`
- enviar os arquivos publicados por FTP
- executar script SQL opcional
- registrar logs de deploy e consultar execuções anteriores

Ainda é um projeto em fase inicial, então a interface continua simples e direta.

## Requisitos

- Windows
- Rust e Cargo instalados
- .NET SDK instalado para o comando de deploy
- acesso FTP e ao banco de dados do ambiente de destino

## Estrutura de arquivos

O aplicativo lê e grava arquivos ao lado do executável:

- `deploy2monster.cfg`: configuração local do aplicativo, com a chave usada para criptografar dados sensíveis dos projetos
- `projects/`: pasta com os arquivos `.d2mproj`
- `logs/`: pasta com os logs de deploy

Os arquivos de projeto usam JSON e as senhas são armazenadas criptografadas localmente.

## Compilação e execução

Durante o desenvolvimento:

```powershell
cargo run -- -help
```

Gerar binário de release:

```powershell
cargo build --release
```

Executar o binário compilado:

```powershell
target\release\deploy2monster.exe -help
```

## Comandos

### Ajuda e versão

- `-help`: mostra a lista de comandos disponíveis
- `-version`: exibe a versão atual do aplicativo

### Projetos

- `-new <nome_do_projeto>`: cria um novo arquivo `.d2mproj`
- `-edit <nome_do_projeto>`: edita um projeto existente
- `-list`: lista os projetos cadastrados
- `-delete <nome_do_projeto>`: remove um projeto cadastrado
- `-export <nome_do_projeto> <caminho>`: exporta um projeto para o caminho informado
- `-import <caminho_arquivo.d2mproj>`: importa um projeto exportado

### Deploy e validação

- `-deploy <nome_do_projeto> [--skip-sql]`: executa o deploy completo
- `-dbUpdate <nome_do_projeto>`: executa apenas a etapa de banco de dados
- `-test <nome_do_projeto>`: testa FTP e banco de dados
- `-logs <nome_do_projeto>`: lista logs do projeto e permite abrir um log no terminal

## Fluxo de deploy

Quando o comando `-deploy` é executado, o fluxo atual é:

1. valida as configurações do projeto
2. executa `dotnet publish` em `Release`
3. envia os arquivos publicados via FTP
4. executa o script SQL, se configurado
5. remove logs antigos do projeto com mais de 30 dias

Se a opção `--skip-sql` for usada, a etapa de banco é ignorada.

## Criando um projeto

Exemplo:

```powershell
cargo run -- -new MeuProjeto
```

O assistente interativo solicita:

- caminho do arquivo `.csproj`
- pasta de publicação
- host, porta, usuário e senha FTP
- host, porta, usuário, senha e nome do banco de dados
- caminho opcional de um script `.sql` ou `.txt`

Durante a criação, o script SQL é validado de forma simples antes de salvar o projeto.

## Exemplo de uso

```powershell
cargo run -- -new SiteCliente
cargo run -- -list
cargo run -- -test SiteCliente
cargo run -- -deploy SiteCliente
cargo run -- -logs SiteCliente
```

## Formato dos projetos

Cada projeto é salvo como JSON em um arquivo `.d2mproj`.

Campos principais:

- `name`
- `publish_folder`
- `project_file`
- `ftp_settings`
- `database_settings`
- `sql_script`

As senhas são regravadas criptografadas ao salvar localmente. Ao exportar um projeto, as senhas são ofuscadas no arquivo exportado para permitir transferência entre ambientes.

### Exemplo de `.d2mproj`

```json
{
  "name": "SiteCliente",
  "publish_folder": "C:\\Publicacao\\SiteCliente",
  "project_file": "C:\\Repos\\SiteCliente\\SiteCliente.csproj",
  "ftp_settings": {
    "ftp_host": "ftp.exemplo.com",
    "ftp_port": 21,
    "ftp_user": "sitecliente",
    "ftp_password": "<criptografado>"
  },
  "database_settings": {
    "host": "db.exemplo.com",
    "port": 3306,
    "user": "sitecliente",
    "password": "<criptografado>",
    "database": "sitecliente"
  },
  "sql_script": "C:\\Repos\\SiteCliente\\Scripts\\atualizacao.sql"
}
```

## Observações

- A CLI usa comandos com hífen, como `-new` e `-deploy`.
- O deploy foi pensado para cenários Windows, especialmente para desenvolvedores .NET.
- O aplicativo ainda não faz deploy completo sem interação: criação, edição, confirmação e visualização de logs seguem um fluxo guiado.

## Roadmap

- suporte mais explícito a validação pré-deploy
- separação mais clara entre build, empacotamento, validação e envio
- compactação de artefatos publicados
- melhorias no fluxo de logs e diagnóstico
- criação de configuração mais rica para o projeto
- envio mais detalhado para o ambiente MonsterASP

# Design: CLI `ferro` instalável e comandos explícitos

## Objetivo

Distribuir o binário `ferro` junto aos instaladores do Ferro Studio e permitir
que seja chamado pelo terminal sem Cargo. A CLI deve ter comandos explícitos
para validar, executar, depurar, criar, formatar, exportar, mostrar ajuda e
consultar a sintaxe.

## Decisões

- `ferro programa.fer` é atalho para executar o arquivo na VM; `ferro executar`
  permanece disponível e aceita as opções atuais.
- `ferro valida [arquivo-ou-pasta...]` valida fontes `.fer` e projetos `.design`
  / `.ferproj`; sem caminho, percorre a pasta atual. `ferro validar` é alias.
- `ferro novo --desktop <pasta> --nome "Meu App"` cria app desktop.
- `ferro novo --jogo <pasta> --nome "Meu App"` só deve ser habilitado quando o
  validador de `.ferproj` reconhecer `Tipo="Jogo"`. Neste checkout, o validador
  ainda só aceita projetos desktop; o comando informa isso e não cria arquivos.
- `ferro ajuda` equivale a `--help`; `ferro sintaxe` imprime a referência
  concisa da gramática/sintaxe Ferro já aceita pelo frontend.
- `ferro` e `ferro valida` sem argumentos validam a pasta atual; `ferro --help`
  continua mostrando a ajuda.
- Linux `.deb`, `.rpm` e Arch instalam `/usr/bin/ferro`; o AppImage inclui o
  binário e sua extração manual é documentada. Windows inclui o binário e
  registra o diretório no PATH do usuário via instaladores NSIS/MSI. macOS
  inclui um comando que instala o CLI em `~/.local/bin` e registra o diretório
  em `~/.zprofile`.

## Fora do escopo

- Remover `cargo run` do fluxo de desenvolvimento.
- Introduzir sintaxe Ferro nova ou prometer suporte a gramática além do frontend.
- Instalação automática do AppImage ao apenas baixar/executar o arquivo.

# Modo de aprendizagem por workspace — design

## Objetivo

Lembrar Iniciante, Ponte para Rust ou Rust separadamente por projeto aberto, mantendo a preferência global para arquivos sem workspace e sem modificar a fonte ao alternar modo.

## Fluxo

1. O backend inclui `preferenceKey` em `WorkspaceListing`, derivada de SHA-256 do caminho canônico.
2. Arquivos e exercícios abertos pelo explorador carregam a chave na aba.
3. Ao ativar uma aba, a interface lê o modo da chave do workspace; sem valor específico, usa o modo global legado.
4. Ao trocar o modo, a interface salva apenas a chave do workspace ativo; abas standalone salvam na chave global.
5. A disponibilidade do modo Rust continua condicionada a `rustc`; uma preferência Rust indisponível mostra Iniciante sem reescrever documento.

## Limites

- A preferência é local ao perfil de usuário; não sincroniza entre máquinas.
- O hash não é caminho, permissão ou token de acesso.
- A preferência não determina linguagem do documento, não converte código e não muda o runtime.

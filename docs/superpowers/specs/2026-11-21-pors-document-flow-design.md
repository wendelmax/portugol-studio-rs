# Design: arquivos e análise Portugol-RS (`.pors`)

## Objetivo

Permitir que a IDE abra, edite, salve e analise arquivos Portugol-RS usando o frontend e a HIR já implementados. O usuário deve ver a linguagem correta no editor, receber diagnósticos reais e poder gerar a prévia Rust. A extensão `.por` e a VM clássica continuam iguais.

## Abordagens

1. **Tratar `.pors` como `.por` e reescrever keywords:** rejeitada; muda significados e gera diagnósticos falsos.
2. **Adicionar identificador explícito de linguagem ao documento, com roteamento em serviços Tauri existentes:** escolhida. Mantém o workspace e o IPC Tauri atuais, usa `portugol-rs-frontend` e não introduz permissões nem processo de shell.
3. **Novo app/projeto ou execução Rust embutida nesta fatia:** rejeitada. Substituiria arquitetura e mistura abertura de arquivos com decisão de sandbox/toolchain. Execução `.pors` continua indisponível e a UI a explica.

## Contrato de arquivo

- `.por` identifica Portugol clássico; `.pors` identifica o subset inicial Portugol-RS, sem diferenciação de maiúsculas/minúsculas.
- Arquivo aberto por caminho deriva a linguagem do sufixo. Documento novo pode escolher explicitamente Portugol ou Portugol-RS.
- Salvar como atualiza extensão/language badge/highlighting somente após a gravação ter sucesso.
- Cada aba armazena linguagem; troca de aba restaura editor e linguagem. Buffer não salvo não é reaberto nem descartado silenciosamente.
- `.pors` mantém bytes UTF-8; `.por` mantém leitura UTF-8/Latin-1 histórica.
- Diálogo, argumentos de abertura, arrastar/soltar e associações de plataforma reconhecem ambos os sufixos.

## Análise e prévia

- `portugol-service` expõe `inspect_portugol_rs` e `export_portugol_rs`, convertendo spans de bytes para offsets UTF-16 usados pelo CodeMirror.
- Linter roteia a partir do identificador persistido na aba e descarta resposta antiga se documento/revisão mudou.
- Prévia gerada mostra status e diagnóstico do frontend Portugol-RS, reutilizando o gerador HIR existente.
- Executar e depurar ficam desabilitados em `.pors` com motivo acessível: runtime/toolchain seguro ainda não existe. Nenhum comando de shell é introduzido.
- Formatação e inserção de bibliotecas clássicas não se aplicam a `.pors`; controles correspondentes ficam ocultos/desabilitados enquanto esse documento estiver ativo.

## Não objetivos desta fatia

- Compilar/executar Rust, rodar `cargo`, ownership real/simulado, funções e sintaxe fora do subset inicial.
- Converter automaticamente `.por` ↔ `.pors`.
- Mudar os contratos de execução clássicos ou habilitar shell arbitrário.

## Aceite

- Teste Tauri aceita `.por`, `.pors` e `.pex` ao receber caminhos do sistema.
- Teste da preferência do documento confirma extensão case-insensitive e mudança só após salvar.
- Teste de serviço confirma spans UTF-16 para Unicode, símbolos HIR e diagnóstico de exportação.
- Teste UI valida detecção de linguagem e seleção do linter/highlighter por aba; build frontend passa.
- CI Tauri em Linux/Windows/macOS passa; `.por` continua na suíte de regressão.

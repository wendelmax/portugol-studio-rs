# Prévia e exportação para Rust

O botão **Rust** no editor mostra uma tradução local do programa ativo. O painel acompanha alterações e troca de abas. **Copiar** copia o arquivo gerado e **Salvar .rs** o exporta para continuar fora da IDE. O código não é enviado a nenhum serviço. Arquivos `.rs` podem ser compilados e executados diretamente com `rustc` instalado; a IDE também executa o código da prévia `.fer`.

## Subconjunto disponível

A versão inicial gera Rust para `funcao inicio()` sem parâmetros, declarações locais escalares, expressões tipadas, atribuições, `escreva`, `se` e `senao`. Tipos `inteiro`, `real`, `logico`, `cadeia` e `caracter` são aceitos quando a operação preserva a semântica.

`inteiro` continua sendo de 32 bits, como no Portugol; o Rust gerado usa `i32` e operações wrapping. A cadeia vira `String`, literais são escapados e a concatenação gera uma expressão que não move o valor original. `escreva` não adiciona quebra de linha.

O exportador prefixa nomes de variáveis com `pt_` para evitar palavras reservadas e colisões com auxiliares Rust. Variáveis sem inicializador recebem os valores padrão do interpretador. A saída de `logico` e `real` conserva a grafia e a formatação do Portugol.

Quando o código usa recursos fora desse subconjunto — por exemplo `leia`, laços, bibliotecas, vetores, matrizes, funções auxiliares ou conversões implícitas — a prévia explica o primeiro recurso sem suporte e sua posição. Nenhum código parcial é apresentado como tradução válida. Editar e executar o programa original continuam usando o compilador e a VM de Portugol.

## Ferro (`.fer`)

Ferro é um dialeto didático implementado separadamente da prévia do Portugol clássico. O subset atual inclui variáveis escalares tipadas, funções auxiliares com parâmetros e retorno tipados, chamadas (incluindo recursão), expressões aritméticas, comparações, lógica `&&`/`||`, condicionais, laços `enquanto` e `para`, `pare`, `continue`, concatenação de cadeias e `leia` tipado. O editor oferece outline, navegação e renomeação semântica para funções e bindings PORS. Recursos fora da gramática atual, como operadores bit a bit, vetores e bibliotecas, recebem diagnósticos em português com posição de origem.

Executar e depurar um `.fer` usam a VM educacional, sem exigir Rust instalado; `Arquivos.*` fica restrito à pasta do programa. A opção **Executar como Rust**, na paleta de comandos, compila o Rust gerado com `rustc` quando a toolchain está disponível. A depuração mostra os valores da VM. A depuração não representa execução, ownership ou borrowing do Rust. O laboratório de ownership continua sendo conteúdo editorial baseado em cenários, não uma análise do buffer do aluno.

## Executar arquivos Rust (`.rs`)

A IDE compila arquivos `.rs` como Rust 2021 usando `rustc` encontrado no `PATH`. A toolchain Rust precisa estar instalada no sistema; os instaladores do Ferro Studio não incluem nem instalam `rustc`. Sem a toolchain, o botão Executar fica indisponível. A execução de cada arquivo exige ação explícita e uma confirmação por sessão.

A compilação tem limite de 20 segundos e a execução de 3 segundos. A fonte e a saída combinada têm limite de 1 MiB. A entrada padrão é fechada; dependências externas, módulos externos e entrada interativa não estão disponíveis no fluxo standalone. O supervisor aplica o limite ao processo associado; código nativo sem sandbox ainda pode criar processos destacados ou escapar do grupo/job antes de ser associado.

## Executar um projeto Cargo

Ao abrir pelo explorador um arquivo `.rs` dentro de um workspace que contenha `Cargo.toml` ancestral, o botão Executar compila o projeto com Cargo. Usa o `default-run` quando definido; sem padrão inequívoco, permite selecionar um binário por pacote. Antes do primeiro build, a IDE informa que Cargo pode acessar a rede e executar scripts de build, macros procedurais e dependências com as permissões da conta local. O fluxo não possui sandbox e exige consentimento explícito separado do consentimento de Rust standalone. Arquivos Rust modificados no workspace precisam ser salvos antes da execução.

O build tem limite de 20 segundos; o runtime Cargo, 3 segundos; a saída, 1 MiB. Parar/Shift+F7 cancela o processo supervisionado. Sem `Cargo.toml` ancestral, um `.rs` continua usando o fluxo standalone com `rustc`. O instalador não inclui Rust/Cargo. A IDE identifica os binários por metadata local/offline: usa o alvo padrão do workspace quando inequívoco e abre um seletor acessível quando há vários bins sem padrão; a seleção fica associada à aba. Alvos são revalidados no backend e passados como `--package`/`--bin` sem shell. O campo Parâmetros de execução também encaminha `argv` ao binário após `--`, sem interpretar shell nem aceitar flags Cargo; mantém os limites por plataforma usados no Rust standalone. Projetos sem binários mostram uma mensagem e não tentam compilar. A paleta **Analisar projeto Cargo** executa `cargo check` sob demanda e apresenta no painel Mensagens os diagnósticos do arquivo aberto; essa análise requer consentimento explícito e não roda continuamente. O seletor ainda cobre somente bins; lint contínuo e debugger Rust ainda não estão disponíveis.

**Aviso:** programas Rust nativos executam com as permissões da conta local e não possuem sandbox. Execute somente código em que você confia.

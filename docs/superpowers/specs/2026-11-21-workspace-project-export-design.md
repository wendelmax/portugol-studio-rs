# Design: exportação local de projeto

## Intenção

Implementar UI-006/UC-14/FT-10 com cópia local dos arquivos selecionados no Explorador. A exportação deve preservar subpastas, evitar sobrescrever projetos e nunca sugerir upload/publicação.

## Abordagens

- Criar um ZIP: formato portátil, mas requer uma nova dependência de arquivamento, que a governança atual pede justificar/aprovar.
- Copiar os arquivos selecionados para uma nova subpasta escolhida pelo usuário: escolhido para reutilizar o seletor nativo e as capacidades Rust existentes, sem dependência ou formato de arquivo próprio.
- Copiar o workspace inteiro sem seleção: rejeitado porque o FT-10 pede escolha explícita dos arquivos.

## Limites e compatibilidade

- Selecionar arquivos já visíveis pelo Explorador: `.por`, `.pors`, `.rs`, `.pex`; preservar caminhos relativos.
- Marcar todos por padrão, permitindo desmarcar qualquer subconjunto.
- O seletor de pasta nativo será aberto no comando Tauri; cancelamento não copia arquivos.
- Criar uma subpasta com o nome do workspace e falhar se ela já existir.
- Validar arquivos selecionados pelo workspace aberto, rejeitar traversal/symlinks e confinar gravações à nova subpasta.
- Nenhuma alteração de IPC permissivo, shell, dependência ou runtime.
- Exibir sucesso apenas após todas as cópias terminarem; falha limpa a pasta parcial criada pela operação.

## Aceite

- Ação **Exportar projeto** fica indisponível sem workspace e abre uma lista de arquivos com caixas de seleção.
- Lista vazia, nenhum item selecionado, cancelamento e erro têm estados claros.
- Arquivos selecionados são copiados com seus caminhos; arquivos não selecionados ficam de fora.
- A cópia é local e não modifica a origem.
- Nenhum item existente é sobrescrito e caminhos não podem escapar do workspace/destino.
- Testes cobrem seleção, preservação de caminhos/conteúdo, traversal, symlinks, cancelamento/colisão e fluxo UI.

## Aprovação e escopo

O usuário autorizou continuar as lacunas funcionais. Cópia em pasta é a decisão local para evitar uma dependência não aprovada; a saída fica explícita como exportação local de pasta, não como ZIP.

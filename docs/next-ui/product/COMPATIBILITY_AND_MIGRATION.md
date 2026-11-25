# Compatibilidade com projeto existente

## Política
- Código atual é o baseline observado; este documento não determina que o projeto atual já possua cada recurso.
- Registrar fixtures e formatos reais antes de introduzir novos dialetos ou extensões.
- Recursos originais a investigar: funções, bibliotecas, gráficos, exemplos, estrutura, debugger, arquivos e importação/exportação.
- Qualquer migração deve ser reversível ou precedida de backup; não alterar silenciosamente código do aluno.
- Mapear projeto atual → nova arquitetura por adapter em vez de reescrita em bloco.

## Matriz a preencher pelo agente

| Recurso | Existe? evidência | Teste baseline | Mudança proposta | Risco | Rollback |
|---|---|---|---|---|---|
| Abrir/Salvar | UNKNOWN | UNKNOWN | NONE | Alta | Reverter adapter |
| Executar Portugol | UNKNOWN | UNKNOWN | NONE | Crítica | Flag desativa UI nova |
| Debug | UNKNOWN | UNKNOWN | PENDING | Alta | Retornar fluxo atual |
| Bibliotecas | UNKNOWN | UNKNOWN | PENDING | Alta | Preservar engine |
| Formato de arquivo | UNKNOWN | UNKNOWN | NONE | Crítica | Backup/restauração |

Testar round-trip de arquivos legados, caracteres acentuados, caminhos com espaço e formatos com encoding diferente se suportados atualmente.

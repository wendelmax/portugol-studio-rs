# Plano: layout de painéis ajustável e persistente

1. Definir funções puras para validar, limitar, carregar e salvar proporções; escrever testes e confirmar falhas esperadas.
2. Adicionar uma divisória redimensionável ao Explorador e marcar as três divisórias com semântica ARIA e suporte por teclado.
3. Refatorar os handlers de arraste para limites consistentes, persistência das proporções e adaptação à janela.
4. Restaurar tamanhos ao abrir/mostrar a perspectiva e esconder a divisória do Explorador nos modos oculto/expandido.
5. Adicionar testes de contratos HTML/CSS e validar arraste, limites, teclado e restauração; executar suite frontend, build e verificações Rust/CI.

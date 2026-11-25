# language: pt
Funcionalidade: Execução protegida e contratos de Tauri
  Como aluno
  Quero executar código sem que a interface aceite comandos arbitrários ou perca o controle da execução

  Cenário: Não executar arquivo fora do workspace autorizado
    Dado um workspace aberto e autorizado
    Quando uma requisição tentar acessar caminho fora desse workspace
    Então o backend Rust deve rejeitar a operação com erro tipado
    E nenhum arquivo externo deve ser alterado

  Cenário: Cancelar programa que excede o limite de execução
    Dado um programa com laço infinito
    Quando a execução exceder o timeout configurado
    Então a operação deve terminar com status de timeout
    E os processos filhos devem ser encerrados conforme capacidade da plataforma

  Cenário: Ignorar eventos de uma execução antiga
    Dado que a execução A foi substituída pela execução B
    Quando chegar um evento atrasado da execução A
    Então o console ativo deve continuar mostrando apenas a execução B

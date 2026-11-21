export const OWNERSHIP_SCENARIO_IDS = ["move", "shared-borrow", "mutable-borrow", "scope"] as const;

export type OwnershipScenarioId = (typeof OWNERSHIP_SCENARIO_IDS)[number];
export type OwnershipProvenance = "verified-simulation";
export type OwnershipBindingState =
  | "owner"
  | "shared-reference"
  | "mutable-reference"
  | "moved"
  | "expired"
  | "dropped";

export interface OwnershipBinding {
  readonly name: string;
  readonly type: string;
  readonly state: OwnershipBindingState;
  readonly description: string;
  readonly target?: string;
  readonly value?: string;
}

export interface OwnershipHeapBlock {
  readonly id: string;
  readonly value: string;
  readonly owner: string | null;
  readonly state: "allocated" | "released";
}

export interface OwnershipSnapshot {
  readonly title: string;
  readonly explanation: string;
  readonly currentLine: number;
  readonly provenance: OwnershipProvenance;
  readonly bindings: readonly OwnershipBinding[];
  readonly heap: readonly OwnershipHeapBlock[];
  readonly diagnostic?: string;
}

export interface OwnershipScenario {
  readonly id: OwnershipScenarioId;
  readonly title: string;
  readonly source: string;
  readonly fixture: string;
  readonly expected: "compiles" | "compile-error";
  readonly expectedOutput?: string;
  readonly snapshots: readonly OwnershipSnapshot[];
}

const simulation = (snapshot: Omit<OwnershipSnapshot, "provenance">): OwnershipSnapshot => ({
  ...snapshot,
  provenance: "verified-simulation",
});

const owner = (name: string, value: string, description = "É o único dono do valor String."): OwnershipBinding => ({
  name,
  type: "String",
  state: "owner",
  description,
  target: "String no heap",
  value,
});

const stringHeap = (value: string, ownerName: string | null, state: OwnershipHeapBlock["state"] = "allocated"): OwnershipHeapBlock => ({
  id: "String no heap",
  value,
  owner: ownerName,
  state,
});

const scenarios: Readonly<Record<OwnershipScenarioId, OwnershipScenario>> = {
  move: {
    id: "move",
    title: "Movimento de um valor (move)",
    fixture: "move-invalid.rs",
    expected: "compile-error",
    source: `fn main() {
    let a = String::from("Olá");
    let b = a;
    println!("{a}");
    println!("{b}");
}`,
    snapshots: [
      simulation({
        title: "a recebe a String",
        explanation: "A variável a é dona da String. O conteúdo UTF-8 fica no heap; a variável guarda a posse desse conteúdo.",
        currentLine: 2,
        bindings: [owner("a", "Olá", "É dona da String e controla sua liberação.")],
        heap: [stringHeap("Olá", "a")],
      }),
      simulation({
        title: "A posse passa de a para b",
        explanation: "A atribuição move a String. b passa a ser o único dono; a continua no escopo, mas não pode mais ser usada como valor.",
        currentLine: 3,
        bindings: [
          { name: "a", type: "String", state: "moved", description: "Valor movido para b; não pode ser lido." },
          { name: "b", type: "String", state: "owner", description: "Agora é o único dono da String.", target: "String no heap", value: "Olá" },
        ],
        heap: [stringHeap("Olá", "b")],
      }),
      simulation({
        title: "O compilador rejeita o uso de a",
        explanation: "println!(\"{a}\") tenta usar um valor que já foi movido. Rust interrompe a compilação; o programa não chega a executar.",
        currentLine: 4,
        bindings: [
          { name: "a", type: "String", state: "moved", description: "Uso inválido: a não é mais dona do valor." },
          { name: "b", type: "String", state: "owner", description: "Permanece dona do conteúdo no heap.", target: "String no heap", value: "Olá" },
        ],
        heap: [stringHeap("Olá", "b")],
        diagnostic: "Erro de compilação esperado: a foi usada depois que seu valor foi movido para b.",
      }),
    ],
  },
  "shared-borrow": {
    id: "shared-borrow",
    title: "Empréstimo compartilhado (&T)",
    fixture: "shared-borrow.rs",
    expected: "compiles",
    expectedOutput: "Olá Olá\nOlá\n",
    source: `fn main() {
    let texto = String::from("Olá");
    let ref1 = &texto;
    let ref2 = &texto;
    println!("{ref1} {ref2}");
    println!("{texto}");
}`,
    snapshots: [
      simulation({
        title: "texto possui o conteúdo",
        explanation: "A variável texto é a dona da String alocada no heap.",
        currentLine: 2,
        bindings: [owner("texto", "Olá")],
        heap: [stringHeap("Olá", "texto")],
      }),
      simulation({
        title: "Primeira referência compartilhada",
        explanation: "ref1 empresta acesso de leitura a texto. Criar a referência não transfere a posse.",
        currentLine: 3,
        bindings: [
          owner("texto", "Olá"),
          { name: "ref1", type: "&String", state: "shared-reference", description: "Empréstimo compartilhado de texto.", target: "texto" },
        ],
        heap: [stringHeap("Olá", "texto")],
      }),
      simulation({
        title: "Dois empréstimos de leitura coexistem",
        explanation: "Rust permite várias referências compartilhadas ao mesmo tempo. texto continua sendo o dono e também pode ser lido.",
        currentLine: 4,
        bindings: [
          owner("texto", "Olá"),
          { name: "ref1", type: "&String", state: "shared-reference", description: "Empréstimo compartilhado de texto.", target: "texto" },
          { name: "ref2", type: "&String", state: "shared-reference", description: "Segundo empréstimo compartilhado de texto.", target: "texto" },
        ],
        heap: [stringHeap("Olá", "texto")],
      }),
      simulation({
        title: "As duas referências são lidas",
        explanation: "ref1 e ref2 podem ser usadas juntas para leitura. O conteúdo não muda e a posse continua com texto.",
        currentLine: 5,
        bindings: [
          owner("texto", "Olá"),
          { name: "ref1", type: "&String", state: "shared-reference", description: "Referência compartilhada usada para leitura.", target: "texto" },
          { name: "ref2", type: "&String", state: "shared-reference", description: "Referência compartilhada usada para leitura.", target: "texto" },
        ],
        heap: [stringHeap("Olá", "texto")],
      }),
      simulation({
        title: "texto é lido depois dos últimos usos das referências",
        explanation: "As referências continuam em escopo, mas não serão usadas novamente. Como o empréstimo terminou após a última leitura, o owner pode ser acessado.",
        currentLine: 6,
        bindings: [
          owner("texto", "Olá", "Acesso liberado após os últimos usos de ref1 e ref2."),
          { name: "ref1", type: "&String", state: "expired", description: "Empréstimo encerrado; referência não será usada novamente." },
          { name: "ref2", type: "&String", state: "expired", description: "Empréstimo encerrado; referência não será usada novamente." },
        ],
        heap: [stringHeap("Olá", "texto")],
      }),
    ],
  },
  "mutable-borrow": {
    id: "mutable-borrow",
    title: "Empréstimo mutável (&mut T)",
    fixture: "mutable-borrow.rs",
    expected: "compiles",
    expectedOutput: "Olá!\nOlá!\n",
    source: `fn main() {
    let mut texto = String::from("Olá");
    let referencia = &mut texto;
    referencia.push_str("!");
    println!("{referencia}");
    println!("{texto}");
}`,
    snapshots: [
      simulation({
        title: "texto é mutável e possui a String",
        explanation: "A declaração mut permite alterar o valor. texto continua sendo o único dono.",
        currentLine: 2,
        bindings: [owner("texto", "Olá", "É dona da String e pode ter seu conteúdo alterado.")],
        heap: [stringHeap("Olá", "texto")],
      }),
      simulation({
        title: "referencia toma um empréstimo exclusivo",
        explanation: "Enquanto o empréstimo mutável estiver ativo, só referencia pode acessar texto. Não pode haver outro empréstimo simultâneo nem acesso direto ao owner.",
        currentLine: 3,
        bindings: [
          owner("texto", "Olá", "Continua dona, mas o acesso fica bloqueado durante o empréstimo mutável."),
          { name: "referencia", type: "&mut String", state: "mutable-reference", description: "Único empréstimo mutável ativo de texto.", target: "texto" },
        ],
        heap: [stringHeap("Olá", "texto")],
      }),
      simulation({
        title: "A referência altera o conteúdo",
        explanation: "A alteração é feita por referencia. O owner segue bloqueado até o último uso da referência.",
        currentLine: 4,
        bindings: [
          owner("texto", "Olá!", "Continua dona; o acesso direto ainda está bloqueado."),
          { name: "referencia", type: "&mut String", state: "mutable-reference", description: "Empréstimo exclusivo usado para acrescentar !.", target: "texto" },
        ],
        heap: [stringHeap("Olá!", "texto")],
      }),
      simulation({
        title: "O empréstimo termina após o último uso",
        explanation: "referencia não será mais usada. O empréstimo encerra e texto pode voltar a ser lido, embora ainda esteja no mesmo bloco.",
        currentLine: 6,
        bindings: [
          owner("texto", "Olá!", "Acesso liberado novamente depois do último uso de referencia."),
          { name: "referencia", type: "&mut String", state: "expired", description: "Empréstimo encerrado pelo último uso; não pode ser usado daqui em diante." },
        ],
        heap: [stringHeap("Olá!", "texto")],
      }),
    ],
  },
  scope: {
    id: "scope",
    title: "Escopo e liberação do valor",
    fixture: "scope.rs",
    expected: "compiles",
    expectedOutput: "Olá\nOlá\n",
    source: `fn main() {
    let texto = String::from("Olá");
    {
        let referencia = &texto;
        println!("{referencia}");
    }
    println!("{texto}");
}`,
    snapshots: [
      simulation({
        title: "texto possui a String",
        explanation: "texto vive no escopo de main e controla a posse do conteúdo no heap.",
        currentLine: 2,
        bindings: [owner("texto", "Olá")],
        heap: [stringHeap("Olá", "texto")],
      }),
      simulation({
        title: "A referência nasce no bloco interno",
        explanation: "referencia empresta texto apenas para leitura. Sua vida não pode ultrapassar o escopo do owner.",
        currentLine: 4,
        bindings: [
          owner("texto", "Olá"),
          { name: "referencia", type: "&String", state: "shared-reference", description: "Empréstimo compartilhado no bloco interno.", target: "texto" },
        ],
        heap: [stringHeap("Olá", "texto")],
      }),
      simulation({
        title: "A referência sai do escopo",
        explanation: "Ao fechar o bloco, referencia deixa de existir. O empréstimo termina; texto ainda é válido e pode ser usado.",
        currentLine: 7,
        bindings: [
          owner("texto", "Olá"),
          { name: "referencia", type: "&String", state: "expired", description: "Fora do escopo; o empréstimo já terminou." },
        ],
        heap: [stringHeap("Olá", "texto")],
      }),
      simulation({
        title: "main termina e a String é liberada",
        explanation: "Ao sair do escopo de texto, Rust libera a alocação do heap controlada por esse owner.",
        currentLine: 8,
        bindings: [{ name: "texto", type: "String", state: "dropped", description: "Saiu do escopo; não pode mais ser acessada." }],
        heap: [stringHeap("Olá", null, "released")],
      }),
    ],
  },
};

export function ownershipScenarioIds(): readonly OwnershipScenarioId[] {
  return OWNERSHIP_SCENARIO_IDS;
}

export function getOwnershipScenario(id: string): OwnershipScenario | undefined {
  return scenarios[id as OwnershipScenarioId];
}

export function getOwnershipSnapshot(id: string, stepIndex: number): OwnershipSnapshot | undefined {
  return getOwnershipScenario(id)?.snapshots[stepIndex];
}

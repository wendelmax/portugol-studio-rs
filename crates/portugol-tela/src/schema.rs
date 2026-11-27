//! Esquema da versão 1 do formato `.design`.
//!
//! O esquema é a fonte única do que cada elemento aceita: o validador, a caixa
//! de ferramentas e o painel de propriedades da IDE leem as mesmas tabelas.

/// Tipo esperado para o valor de um atributo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Integer {
        min: i64,
        max: i64,
    },
    Text,
    /// `verdadeiro` ou `falso`.
    Bool,
    /// Nome de uma função PORS que trata um evento.
    Event,
    /// Nome de um `registro` PORS.
    TypeName,
    /// Nome de controle (o atributo `Nome`).
    ControlName,
    /// Um valor entre opções fixas.
    Choice(&'static [&'static str]),
    /// Lista sem repetição das bordas `Cima`, `Baixo`, `Esquerda` e `Direita`.
    Anchors,
    /// Espessura: `"8"`, `"8,4"` ou `"1,2,3,4"`.
    Thickness,
    /// Trilhas da `Grade`: `"Auto,*,2*,120"`.
    Tracks,
    /// Caminho relativo dentro da pasta do projeto, sem `..`.
    ResourcePath,
}

/// Se e como o atributo aceita vínculos `{dados.campo}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    None,
    /// Estado → tela. Em atributos de texto, pode misturar texto e vínculos;
    /// nos outros, o valor deve ser um único vínculo.
    OneWay,
    /// Estado ↔ tela, em controles de entrada. O valor deve ser um único
    /// vínculo.
    TwoWay,
}

/// Contêiner que o atributo exige no pai.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Placement {
    /// Vale em qualquer lugar.
    Any,
    /// Só em `Janela` e `Painel`, que posicionam por `X`/`Y`.
    Free,
    /// Só dentro de `Grade`.
    Grid,
    /// Só dentro de `Encaixe`.
    Dock,
    /// Só dentro de `Pilha`, `Grade` ou `Encaixe`.
    Layout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttributeSchema {
    pub name: &'static str,
    pub value: ValueType,
    pub required: bool,
    pub binding: Binding,
    pub placement: Placement,
    pub description: &'static str,
}

/// Como um contêiner posiciona os filhos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Não aceita filhos.
    None,
    /// Posição livre por `X`/`Y` (`Janela`, `Painel`).
    Free,
    /// Empilha (`Pilha`).
    Stack,
    /// Linhas e colunas (`Grade`).
    Grid,
    /// Bordas e centro (`Encaixe`).
    Dock,
    /// Só aceita `<Item>` (`Lista`, `Combo`).
    Items,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Control,
    Container,
    /// Componente sem aparência, que fica direto na `<Janela>`.
    Component,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementSchema {
    /// Nome da tag.
    pub name: &'static str,
    /// Nome mostrado na caixa de ferramentas.
    pub label: &'static str,
    pub description: &'static str,
    pub category: Category,
    pub layout: Layout,
    /// Evento criado pelo duplo clique no designer.
    pub default_event: Option<&'static str>,
    /// Tamanho inicial ao soltar o controle numa área livre.
    pub default_size: (i64, i64),
    /// Atributos próprios, além dos comuns.
    pub attributes: &'static [AttributeSchema],
}

/// Limite de coordenadas e dimensões em pixels lógicos.
pub const MAX_COORDINATE: i64 = 10_000;
pub const ANCHORS: &[&str] = &["Cima", "Baixo", "Esquerda", "Direita"];
pub const ALIGNMENTS: &[&str] = &["Inicio", "Centro", "Fim", "Esticar"];
pub const DOCK_SIDES: &[&str] = &["Cima", "Baixo", "Esquerda", "Direita"];
pub const ORIENTATIONS: &[&str] = &["Vertical", "Horizontal"];

const POSITION: ValueType = ValueType::Integer {
    min: -MAX_COORDINATE,
    max: MAX_COORDINATE,
};
const SIZE: ValueType = ValueType::Integer {
    min: 1,
    max: MAX_COORDINATE,
};
const INDEX: ValueType = ValueType::Integer { min: 0, max: 99 };

const fn attr(name: &'static str, value: ValueType, description: &'static str) -> AttributeSchema {
    AttributeSchema {
        name,
        value,
        required: false,
        binding: Binding::None,
        placement: Placement::Any,
        description,
    }
}

impl AttributeSchema {
    const fn required(mut self) -> Self {
        self.required = true;
        self
    }
    const fn one_way(mut self) -> Self {
        self.binding = Binding::OneWay;
        self
    }
    const fn two_way(mut self) -> Self {
        self.binding = Binding::TwoWay;
        self
    }
    const fn only(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
}

const NAME: AttributeSchema = attr(
    "Nome",
    ValueType::ControlName,
    "Nome do controle no code-behind",
);

/// Atributos da `<Janela>`.
pub const WINDOW: ElementSchema = ElementSchema {
    name: "Janela",
    label: "Janela",
    description: "A janela do aplicativo; posiciona os filhos livremente por X e Y",
    category: Category::Container,
    layout: Layout::Free,
    default_event: Some("AoAbrir"),
    default_size: (400, 300),
    attributes: &[
        NAME.required(),
        attr(
            "Formato",
            ValueType::Integer { min: 1, max: 1 },
            "Versão do formato .design",
        ),
        attr("Titulo", ValueType::Text, "Texto da barra de título")
            .required()
            .one_way(),
        attr("Largura", SIZE, "Largura da área da janela").required(),
        attr("Altura", SIZE, "Altura da área da janela").required(),
        attr(
            "Redimensionavel",
            ValueType::Bool,
            "Se o usuário pode mudar o tamanho da janela",
        ),
        attr(
            "Preenchimento",
            ValueType::Thickness,
            "Espaço entre a borda da janela e o conteúdo",
        ),
        attr(
            "Dados",
            ValueType::TypeName,
            "Registro PORS com o estado mostrado pelos vínculos {dados.campo}",
        ),
        attr("AoAbrir", ValueType::Event, "Executado ao abrir a janela"),
        attr(
            "AoFechar",
            ValueType::Event,
            "Executado quando o usuário fecha a janela",
        ),
    ],
};

/// Atributos comuns a controles e contêineres visuais.
pub const VISUAL: &[AttributeSchema] = &[
    NAME,
    attr("X", POSITION, "Distância da borda esquerda do contêiner").only(Placement::Free),
    attr("Y", POSITION, "Distância da borda superior do contêiner").only(Placement::Free),
    attr(
        "Ancora",
        ValueType::Anchors,
        "Bordas do contêiner às quais o controle fica preso ao redimensionar",
    )
    .only(Placement::Free),
    attr("Largura", SIZE, "Largura do controle"),
    attr("Altura", SIZE, "Altura do controle"),
    attr(
        "Margem",
        ValueType::Thickness,
        "Espaço por fora do controle",
    ),
    attr(
        "Alinhamento",
        ValueType::Choice(ALIGNMENTS),
        "Alinhamento horizontal dentro do espaço que o contêiner dá",
    )
    .only(Placement::Layout),
    attr(
        "AlinhamentoVertical",
        ValueType::Choice(ALIGNMENTS),
        "Alinhamento vertical dentro do espaço que o contêiner dá",
    )
    .only(Placement::Layout),
    attr("Linha", INDEX, "Linha da Grade, começando em 0").only(Placement::Grid),
    attr("Coluna", INDEX, "Coluna da Grade, começando em 0").only(Placement::Grid),
    attr(
        "Encaixar",
        ValueType::Choice(DOCK_SIDES),
        "Borda do Encaixe ocupada pelo controle",
    )
    .only(Placement::Dock),
    attr("Visivel", ValueType::Bool, "Se o controle aparece").one_way(),
    attr(
        "Habilitado",
        ValueType::Bool,
        "Se o controle aceita interação",
    )
    .one_way(),
    attr(
        "Dica",
        ValueType::Text,
        "Texto mostrado ao parar o mouse sobre o controle",
    )
    .one_way(),
    attr(
        "Descricao",
        ValueType::Text,
        "Texto lido por leitores de tela quando o controle não tem texto visível",
    ),
];

const TEXT: AttributeSchema = attr(
    "Texto",
    ValueType::Text,
    "Texto exibido; `_` marca a tecla de acesso",
)
.one_way();
const PADDING: AttributeSchema = attr(
    "Preenchimento",
    ValueType::Thickness,
    "Espaço entre a borda e o conteúdo",
);
const CHECKED: AttributeSchema = attr("Marcado", ValueType::Bool, "Se está marcado").two_way();
const SELECTED: AttributeSchema = attr(
    "Selecionado",
    ValueType::Integer {
        min: -1,
        max: 100_000,
    },
    "Posição do item selecionado, começando em 0; -1 para nenhum",
)
.two_way();
const ON_CHANGE: AttributeSchema = attr(
    "AoAlterar",
    ValueType::Event,
    "Executado quando o valor muda",
);
const ON_SELECT: AttributeSchema = attr(
    "AoSelecionar",
    ValueType::Event,
    "Executado quando a seleção muda",
);

pub const ELEMENTS: &[ElementSchema] = &[
    ElementSchema {
        name: "Rotulo",
        label: "Rótulo",
        description: "Texto fixo ou vinculado ao estado",
        category: Category::Control,
        layout: Layout::None,
        default_event: None,
        default_size: (100, 24),
        attributes: &[TEXT],
    },
    ElementSchema {
        name: "Botao",
        label: "Botão",
        description: "Executa uma ação ao ser clicado",
        category: Category::Control,
        layout: Layout::None,
        default_event: Some("AoClicar"),
        default_size: (100, 32),
        attributes: &[
            TEXT,
            PADDING,
            attr("AoClicar", ValueType::Event, "Executado ao clicar"),
        ],
    },
    ElementSchema {
        name: "CaixaTexto",
        label: "Caixa de texto",
        description: "Campo para o usuário digitar",
        category: Category::Control,
        layout: Layout::None,
        default_event: Some("AoAlterar"),
        default_size: (160, 28),
        attributes: &[
            attr("Texto", ValueType::Text, "Texto digitado").two_way(),
            attr(
                "TextoVazio",
                ValueType::Text,
                "Texto de exemplo mostrado enquanto o campo está vazio",
            ),
            attr(
                "SomenteLeitura",
                ValueType::Bool,
                "Impede a edição pelo usuário",
            )
            .one_way(),
            attr("Senha", ValueType::Bool, "Esconde os caracteres digitados"),
            attr(
                "Multilinha",
                ValueType::Bool,
                "Aceita várias linhas de texto",
            ),
            ON_CHANGE,
        ],
    },
    ElementSchema {
        name: "CaixaSelecao",
        label: "Caixa de seleção",
        description: "Opção que pode ser marcada ou desmarcada",
        category: Category::Control,
        layout: Layout::None,
        default_event: Some("AoAlterar"),
        default_size: (140, 24),
        attributes: &[TEXT, CHECKED, ON_CHANGE],
    },
    ElementSchema {
        name: "BotaoOpcao",
        label: "Botão de opção",
        description: "Uma opção entre várias do mesmo grupo",
        category: Category::Control,
        layout: Layout::None,
        default_event: Some("AoAlterar"),
        default_size: (140, 24),
        attributes: &[
            TEXT,
            CHECKED,
            attr(
                "Grupo",
                ValueType::Text,
                "Opções do mesmo grupo são mutuamente exclusivas",
            ),
            ON_CHANGE,
        ],
    },
    ElementSchema {
        name: "Lista",
        label: "Lista",
        description: "Lista de itens com seleção; os itens são elementos <Item>",
        category: Category::Control,
        layout: Layout::Items,
        default_event: Some("AoSelecionar"),
        default_size: (160, 120),
        attributes: &[SELECTED, ON_SELECT],
    },
    ElementSchema {
        name: "Combo",
        label: "Caixa de combinação",
        description: "Lista suspensa de itens; os itens são elementos <Item>",
        category: Category::Control,
        layout: Layout::Items,
        default_event: Some("AoSelecionar"),
        default_size: (160, 28),
        attributes: &[SELECTED, ON_SELECT],
    },
    ElementSchema {
        name: "Imagem",
        label: "Imagem",
        description: "Imagem da pasta do projeto",
        category: Category::Control,
        layout: Layout::None,
        default_event: None,
        default_size: (120, 120),
        attributes: &[attr(
            "Arquivo",
            ValueType::ResourcePath,
            "Caminho da imagem relativo à pasta do projeto, como recursos/logo.png",
        )],
    },
    ElementSchema {
        name: "Painel",
        label: "Painel",
        description: "Agrupa controles com posição livre; X e Y dos filhos são relativos ao painel",
        category: Category::Container,
        layout: Layout::Free,
        default_event: None,
        default_size: (200, 150),
        attributes: &[
            attr("Titulo", ValueType::Text, "Título exibido na borda").one_way(),
            attr("Borda", ValueType::Bool, "Desenha uma borda em volta"),
            PADDING,
        ],
    },
    ElementSchema {
        name: "Pilha",
        label: "Pilha",
        description: "Empilha os filhos na vertical ou na horizontal",
        category: Category::Container,
        layout: Layout::Stack,
        default_event: None,
        default_size: (200, 150),
        attributes: &[
            attr(
                "Orientacao",
                ValueType::Choice(ORIENTATIONS),
                "Direção em que os filhos são empilhados",
            ),
            attr(
                "Espaco",
                ValueType::Integer {
                    min: 0,
                    max: MAX_COORDINATE,
                },
                "Espaço entre os filhos",
            ),
            PADDING,
        ],
    },
    ElementSchema {
        name: "Grade",
        label: "Grade",
        description: "Distribui os filhos em linhas e colunas",
        category: Category::Container,
        layout: Layout::Grid,
        default_event: None,
        default_size: (240, 160),
        attributes: &[
            attr(
                "Linhas",
                ValueType::Tracks,
                "Altura de cada linha: Auto, * (proporcional) ou um número",
            ),
            attr(
                "Colunas",
                ValueType::Tracks,
                "Largura de cada coluna: Auto, * (proporcional) ou um número",
            ),
            attr(
                "Espaco",
                ValueType::Integer {
                    min: 0,
                    max: MAX_COORDINATE,
                },
                "Espaço entre linhas e colunas",
            ),
            PADDING,
        ],
    },
    ElementSchema {
        name: "Encaixe",
        label: "Encaixe",
        description: "Prende os filhos às bordas; o último sem Encaixar ocupa o centro",
        category: Category::Container,
        layout: Layout::Dock,
        default_event: None,
        default_size: (320, 240),
        attributes: &[PADDING],
    },
    ElementSchema {
        name: "Temporizador",
        label: "Temporizador",
        description: "Dispara um evento em intervalos regulares",
        category: Category::Component,
        layout: Layout::None,
        default_event: Some("AoDisparar"),
        default_size: (0, 0),
        attributes: &[
            NAME,
            attr(
                "Intervalo",
                ValueType::Integer {
                    min: 10,
                    max: 86_400_000,
                },
                "Intervalo entre disparos, em milissegundos",
            )
            .required(),
            attr("Ativo", ValueType::Bool, "Se o temporizador está ligado").one_way(),
            attr("AoDisparar", ValueType::Event, "Executado a cada intervalo"),
        ],
    },
    ElementSchema {
        name: "DialogoArquivo",
        label: "Diálogo de arquivo",
        description: "Diálogo do sistema para abrir ou salvar arquivos",
        category: Category::Component,
        layout: Layout::None,
        default_event: None,
        default_size: (0, 0),
        attributes: &[
            NAME,
            attr("Titulo", ValueType::Text, "Título do diálogo"),
            attr(
                "Modo",
                ValueType::Choice(&["Abrir", "Salvar"]),
                "Se o diálogo abre ou salva um arquivo",
            ),
            attr(
                "Filtro",
                ValueType::Text,
                "Extensões aceitas, como \"*.txt;*.csv\"",
            ),
        ],
    },
];

/// Elemento filho de `Lista` e `Combo`; o texto entre as tags é o item.
pub const ITEM: &str = "Item";
/// Valores de exemplo para o designer, filho da `<Janela>`.
pub const SAMPLE_DATA: &str = "DadosExemplo";

pub fn element(name: &str) -> Option<&'static ElementSchema> {
    if name == WINDOW.name {
        return Some(&WINDOW);
    }
    ELEMENTS.iter().find(|e| e.name == name)
}

/// Todos os nomes de tag aceitos, para sugestões.
pub fn tag_names() -> impl Iterator<Item = &'static str> {
    std::iter::once(WINDOW.name)
        .chain(ELEMENTS.iter().map(|e| e.name))
        .chain([ITEM, SAMPLE_DATA])
}

impl ElementSchema {
    pub fn is_visual(&self) -> bool {
        self.category != Category::Component && self.name != WINDOW.name
    }

    /// Atributo do elemento, incluindo os comuns aos visuais. Um atributo
    /// próprio substitui o comum de mesmo nome.
    pub fn attribute(&self, name: &str) -> Option<&'static AttributeSchema> {
        let own: &'static [AttributeSchema] = self.attributes;
        own.iter().find(|a| a.name == name).or_else(|| {
            if self.is_visual() {
                VISUAL.iter().find(|a| a.name == name)
            } else {
                None
            }
        })
    }

    /// Todos os atributos aceitos, sem repetição: os comuns primeiro.
    pub fn all_attributes(&self) -> Vec<&'static AttributeSchema> {
        let own: &'static [AttributeSchema] = self.attributes;
        let mut all: Vec<&'static AttributeSchema> = if self.is_visual() {
            VISUAL
                .iter()
                .filter(|common| own.iter().all(|a| a.name != common.name))
                .collect()
        } else {
            Vec::new()
        };
        all.extend(own.iter());
        all
    }
}

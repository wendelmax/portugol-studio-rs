//! Esquema dos controles da versão 1 do formato `.tela`.
//!
//! O esquema é a fonte única do que cada controle aceita: o validador, a caixa
//! de ferramentas e o painel de propriedades da IDE leem as mesmas tabelas.

/// Tipo esperado para o valor de uma propriedade.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Integer {
        min: i64,
        max: i64,
    },
    Text,
    Bool,
    /// Nome de uma função PORS que trata um evento.
    Event,
    /// Nome de um `registro` PORS.
    TypeName,
    /// Lista de cadeias (itens de uma lista ou combo).
    TextList,
    /// Lista sem repetição das bordas `cima`, `baixo`, `esquerda` e `direita`.
    Anchors,
    /// Um nome entre as opções fixas.
    Choice(&'static [&'static str]),
    /// Caminho relativo dentro da pasta do projeto, sem `..`.
    ResourcePath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropertySchema {
    pub name: &'static str,
    pub value: ValueType,
    pub required: bool,
    pub description: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlSchema {
    /// Nome usado no arquivo (`botao`, `caixa_texto`, ...).
    pub name: &'static str,
    /// Nome mostrado na caixa de ferramentas.
    pub label: &'static str,
    pub description: &'static str,
    /// Controles visuais têm posição e tamanho; os outros ficam na faixa de
    /// componentes do designer.
    pub visual: bool,
    /// Pode conter outros controles.
    pub container: bool,
    /// Evento criado pelo duplo clique no designer.
    pub default_event: Option<&'static str>,
    /// Tamanho inicial ao soltar o controle na janela.
    pub default_size: (i64, i64),
    pub properties: &'static [PropertySchema],
}

pub const ANCHORS: &[&str] = &["cima", "baixo", "esquerda", "direita"];

/// Limite de coordenadas e dimensões em pixels lógicos.
pub const MAX_COORDINATE: i64 = 10_000;

const fn prop(
    name: &'static str,
    value: ValueType,
    required: bool,
    description: &'static str,
) -> PropertySchema {
    PropertySchema {
        name,
        value,
        required,
        description,
    }
}

const POSITION: ValueType = ValueType::Integer {
    min: -MAX_COORDINATE,
    max: MAX_COORDINATE,
};
const SIZE: ValueType = ValueType::Integer {
    min: 1,
    max: MAX_COORDINATE,
};

/// Propriedades da janela.
pub const WINDOW_PROPERTIES: &[PropertySchema] = &[
    prop("titulo", ValueType::Text, true, "Texto da barra de título"),
    prop("largura", SIZE, true, "Largura da área da janela"),
    prop("altura", SIZE, true, "Altura da área da janela"),
    prop(
        "redimensionavel",
        ValueType::Bool,
        false,
        "Se o usuário pode mudar o tamanho da janela",
    ),
    prop(
        "dados",
        ValueType::TypeName,
        false,
        "Registro PORS com o estado do programa que não pertence a nenhum controle",
    ),
    prop(
        "ao_abrir",
        ValueType::Event,
        false,
        "Executado ao abrir a janela",
    ),
    prop(
        "ao_fechar",
        ValueType::Event,
        false,
        "Executado quando o usuário fecha a janela",
    ),
];

/// Propriedades comuns a todos os controles visuais.
pub const VISUAL_PROPERTIES: &[PropertySchema] = &[
    prop(
        "x",
        POSITION,
        true,
        "Distância da borda esquerda do contêiner",
    ),
    prop(
        "y",
        POSITION,
        true,
        "Distância da borda superior do contêiner",
    ),
    prop("largura", SIZE, false, "Largura do controle"),
    prop("altura", SIZE, false, "Altura do controle"),
    prop("visivel", ValueType::Bool, false, "Se o controle aparece"),
    prop(
        "habilitado",
        ValueType::Bool,
        false,
        "Se o controle aceita interação",
    ),
    prop(
        "ancora",
        ValueType::Anchors,
        false,
        "Bordas do contêiner às quais o controle fica preso ao redimensionar",
    ),
    prop(
        "dica",
        ValueType::Text,
        false,
        "Texto mostrado ao parar o mouse sobre o controle",
    ),
];

const TEXT: PropertySchema = prop("texto", ValueType::Text, false, "Texto exibido");
const CHECKED: PropertySchema = prop("marcado", ValueType::Bool, false, "Se está marcado");
const ITEMS: PropertySchema = prop("itens", ValueType::TextList, false, "Itens oferecidos");
const SELECTED: PropertySchema = prop(
    "selecionado",
    ValueType::Integer {
        min: -1,
        max: 100_000,
    },
    false,
    "Posição do item selecionado, começando em 0; -1 para nenhum",
);
const ON_CHANGE: PropertySchema = prop(
    "ao_alterar",
    ValueType::Event,
    false,
    "Executado quando o valor muda",
);
const ON_SELECT: PropertySchema = prop(
    "ao_selecionar",
    ValueType::Event,
    false,
    "Executado quando a seleção muda",
);

pub const CONTROLS: &[ControlSchema] = &[
    ControlSchema {
        name: "rotulo",
        label: "Rótulo",
        description: "Texto fixo ou atualizado pelo programa",
        visual: true,
        container: false,
        default_event: None,
        default_size: (100, 24),
        properties: &[TEXT],
    },
    ControlSchema {
        name: "botao",
        label: "Botão",
        description: "Executa uma ação ao ser clicado",
        visual: true,
        container: false,
        default_event: Some("ao_clicar"),
        default_size: (100, 32),
        properties: &[
            TEXT,
            prop("ao_clicar", ValueType::Event, false, "Executado ao clicar"),
        ],
    },
    ControlSchema {
        name: "caixa_texto",
        label: "Caixa de texto",
        description: "Campo para o usuário digitar",
        visual: true,
        container: false,
        default_event: Some("ao_alterar"),
        default_size: (160, 28),
        properties: &[
            TEXT,
            prop(
                "texto_vazio",
                ValueType::Text,
                false,
                "Texto de exemplo mostrado enquanto o campo está vazio",
            ),
            prop(
                "somente_leitura",
                ValueType::Bool,
                false,
                "Impede a edição pelo usuário",
            ),
            prop(
                "senha",
                ValueType::Bool,
                false,
                "Esconde os caracteres digitados",
            ),
            prop(
                "multilinha",
                ValueType::Bool,
                false,
                "Aceita várias linhas de texto",
            ),
            ON_CHANGE,
        ],
    },
    ControlSchema {
        name: "caixa_selecao",
        label: "Caixa de seleção",
        description: "Opção que pode ser marcada ou desmarcada",
        visual: true,
        container: false,
        default_event: Some("ao_alterar"),
        default_size: (140, 24),
        properties: &[TEXT, CHECKED, ON_CHANGE],
    },
    ControlSchema {
        name: "botao_opcao",
        label: "Botão de opção",
        description: "Uma opção entre várias do mesmo grupo",
        visual: true,
        container: false,
        default_event: Some("ao_alterar"),
        default_size: (140, 24),
        properties: &[
            TEXT,
            CHECKED,
            prop(
                "grupo",
                ValueType::Text,
                false,
                "Opções do mesmo grupo são mutuamente exclusivas",
            ),
            ON_CHANGE,
        ],
    },
    ControlSchema {
        name: "lista",
        label: "Lista",
        description: "Lista de itens com seleção",
        visual: true,
        container: false,
        default_event: Some("ao_selecionar"),
        default_size: (160, 120),
        properties: &[ITEMS, SELECTED, ON_SELECT],
    },
    ControlSchema {
        name: "combo",
        label: "Caixa de combinação",
        description: "Lista suspensa de itens",
        visual: true,
        container: false,
        default_event: Some("ao_selecionar"),
        default_size: (160, 28),
        properties: &[ITEMS, SELECTED, ON_SELECT],
    },
    ControlSchema {
        name: "imagem",
        label: "Imagem",
        description: "Imagem da pasta do projeto",
        visual: true,
        container: false,
        default_event: None,
        default_size: (120, 120),
        properties: &[prop(
            "arquivo",
            ValueType::ResourcePath,
            false,
            "Caminho da imagem relativo à pasta do projeto, como recursos/logo.png",
        )],
    },
    ControlSchema {
        name: "painel",
        label: "Painel",
        description: "Agrupa controles; as posições dos filhos são relativas ao painel",
        visual: true,
        container: true,
        default_event: None,
        default_size: (200, 150),
        properties: &[
            prop("titulo", ValueType::Text, false, "Título exibido na borda"),
            prop(
                "borda",
                ValueType::Bool,
                false,
                "Desenha uma borda em volta",
            ),
        ],
    },
    ControlSchema {
        name: "temporizador",
        label: "Temporizador",
        description: "Dispara um evento em intervalos regulares",
        visual: false,
        container: false,
        default_event: Some("ao_disparar"),
        default_size: (0, 0),
        properties: &[
            prop(
                "intervalo",
                ValueType::Integer {
                    min: 10,
                    max: 86_400_000,
                },
                true,
                "Intervalo entre disparos, em milissegundos",
            ),
            prop(
                "ativo",
                ValueType::Bool,
                false,
                "Se o temporizador começa ligado",
            ),
            prop(
                "ao_disparar",
                ValueType::Event,
                false,
                "Executado a cada intervalo",
            ),
        ],
    },
    ControlSchema {
        name: "dialogo_arquivo",
        label: "Diálogo de arquivo",
        description: "Diálogo do sistema para abrir ou salvar arquivos",
        visual: false,
        container: false,
        default_event: None,
        default_size: (0, 0),
        properties: &[
            prop("titulo", ValueType::Text, false, "Título do diálogo"),
            prop(
                "modo",
                ValueType::Choice(&["abrir", "salvar"]),
                false,
                "Se o diálogo abre ou salva um arquivo",
            ),
            prop(
                "filtro",
                ValueType::Text,
                false,
                "Extensões aceitas, como \"*.txt;*.csv\"",
            ),
        ],
    },
];

pub fn control(name: &str) -> Option<&'static ControlSchema> {
    CONTROLS.iter().find(|control| control.name == name)
}

impl ControlSchema {
    /// Propriedade do controle, incluindo as comuns aos controles visuais.
    pub fn property(&self, name: &str) -> Option<&'static PropertySchema> {
        self.properties.iter().find(|p| p.name == name).or_else(|| {
            if self.visual {
                VISUAL_PROPERTIES.iter().find(|p| p.name == name)
            } else {
                None
            }
        })
    }

    /// Todas as propriedades aceitas, na ordem canônica: as comuns primeiro.
    pub fn all_properties(&self) -> impl Iterator<Item = &'static PropertySchema> {
        let common: &'static [PropertySchema] = if self.visual { VISUAL_PROPERTIES } else { &[] };
        common.iter().chain(self.properties.iter())
    }
}

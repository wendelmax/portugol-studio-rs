//! Fontes de Graficos: a padrão (Liberation Sans, com as métricas do Arial,
//! no lugar da fonte "Dialog" do Java) e as carregadas com `carregar_fonte`.
//!
//! As métricas seguem o `FontMetrics` do Java sem métricas fracionárias:
//! ascendente, descendente e entrelinha arredondados, e a largura de um texto
//! como a soma dos avanços arredondados de cada caractere.

use std::sync::Arc;

use ab_glyph::{Font, FontArc};

/// Família padrão do `Graphics2D` (fonte lógica "Dialog").
pub const DEFAULT_FAMILY: &str = "Dialog";
pub const DEFAULT_SIZE: f32 = 12.0;

pub const ITALIC: u8 = 1;
pub const BOLD: u8 = 2;
pub const UNDERLINE: u8 = 4;

/// Nome da família embutida, como a janela gráfica a registra.
pub const BUILTIN_FAMILY: &str = "FerroSans";

static BUILTIN: [(&[u8], u8); 4] = [
    (
        include_bytes!("../../assets/fontes/LiberationSans-Regular.ttf"),
        0,
    ),
    (
        include_bytes!("../../assets/fontes/LiberationSans-Bold.ttf"),
        BOLD,
    ),
    (
        include_bytes!("../../assets/fontes/LiberationSans-Italic.ttf"),
        ITALIC,
    ),
    (
        include_bytes!("../../assets/fontes/LiberationSans-BoldItalic.ttf"),
        BOLD | ITALIC,
    ),
];

/// Bytes das fontes embutidas e o estilo de cada uma, para enviar à janela.
pub fn builtin_files() -> impl Iterator<Item = (&'static [u8], u8)> {
    BUILTIN.iter().copied()
}

pub struct Metrics {
    pub ascent: i32,
    pub descent: i32,
    pub leading: i32,
}

pub struct LoadedFont {
    pub family: String,
    pub bytes: Arc<Vec<u8>>,
    font: FontArc,
}

#[derive(Default)]
pub struct Fonts {
    builtin: Vec<(FontArc, u8)>,
    loaded: Vec<LoadedFont>,
}

impl Fonts {
    fn ensure_builtin(&mut self) {
        if self.builtin.is_empty() {
            self.builtin = BUILTIN
                .iter()
                .map(|(bytes, style)| {
                    (
                        FontArc::try_from_slice(bytes).expect("fonte embutida válida"),
                        *style,
                    )
                })
                .collect();
        }
    }

    fn font(&mut self, family: &str, style: u8) -> FontArc {
        if let Some(loaded) = self
            .loaded
            .iter()
            .find(|f| f.family.eq_ignore_ascii_case(family))
        {
            return loaded.font.clone();
        }
        self.ensure_builtin();
        let wanted = style & (BOLD | ITALIC);
        self.builtin
            .iter()
            .find(|(_, s)| *s == wanted)
            .map(|(f, _)| f.clone())
            .unwrap_or_else(|| self.builtin[0].0.clone())
    }

    pub fn metrics(&mut self, family: &str, style: u8, size: f32) -> Metrics {
        let font = self.font(family, style);
        let unit = size / font.units_per_em().unwrap_or(2048.0);
        Metrics {
            ascent: (font.ascent_unscaled() * unit).round() as i32,
            descent: (-font.descent_unscaled() * unit).round() as i32,
            leading: (font.line_gap_unscaled() * unit).round() as i32,
        }
    }

    pub fn width(&mut self, family: &str, style: u8, size: f32, text: &str) -> i32 {
        let font = self.font(family, style);
        let unit = size / font.units_per_em().unwrap_or(2048.0);
        text.chars()
            .map(|c| (font.h_advance_unscaled(font.glyph_id(c)) * unit).round() as i32)
            .sum()
    }

    /// Registra uma fonte TrueType e devolve o nome da família.
    pub fn load(&mut self, bytes: Vec<u8>) -> Result<&LoadedFont, ()> {
        let family = family_name(&bytes).ok_or(())?;
        let font = FontArc::try_from_vec(bytes.clone()).map_err(|_| ())?;
        self.loaded.retain(|f| f.family != family);
        self.loaded.push(LoadedFont {
            family,
            bytes: Arc::new(bytes),
            font,
        });
        Ok(self.loaded.last().expect("acabou de ser inserida"))
    }
}

fn family_name(bytes: &[u8]) -> Option<String> {
    let face = ttf_parser::Face::parse(bytes, 0).ok()?;
    let names = face.names();
    let pick = |id: u16| {
        names
            .into_iter()
            .filter(|n| n.name_id == id)
            .find_map(|n| n.to_string())
    };
    pick(ttf_parser::name_id::TYPOGRAPHIC_FAMILY).or_else(|| pick(ttf_parser::name_id::FAMILY))
}

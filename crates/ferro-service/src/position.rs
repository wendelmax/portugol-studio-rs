//! Conversão de offsets em bytes (spans do núcleo) para posições do editor.

use ferro_hir::Span;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Position {
    /// Offset em unidades UTF-16 desde o início do texto (o que o CodeMirror
    /// e o JavaScript usam).
    pub offset: u32,
    /// Linha e coluna baseadas em 1; a coluna conta caracteres.
    pub line: u32,
    pub column: u32,
}

/// Índice das quebras de linha para converter vários offsets sem varrer o
/// texto inteiro a cada vez.
pub struct LineIndex<'a> {
    source: &'a str,
    /// Offset em bytes do início de cada linha.
    line_starts: Vec<usize>,
    /// Offset UTF-16 do início de cada linha.
    line_starts_utf16: Vec<u32>,
}

impl<'a> LineIndex<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut line_starts = vec![0];
        let mut line_starts_utf16 = vec![0];
        let mut utf16 = 0u32;
        for (at, ch) in source.char_indices() {
            utf16 += ch.len_utf16() as u32;
            if ch == '\n' {
                line_starts.push(at + 1);
                line_starts_utf16.push(utf16);
            }
        }
        LineIndex {
            source,
            line_starts,
            line_starts_utf16,
        }
    }

    /// Trecho em bytes que cobre uma linha inteira (base 1). Usado para
    /// converter pontos de parada do editor em spans do bytecode.
    pub fn line_span(&self, line: u32) -> Span {
        let index = (line as usize)
            .saturating_sub(1)
            .min(self.line_starts.len().saturating_sub(1));
        let start = self.line_starts[index] as u32;
        let end = self
            .line_starts
            .get(index + 1)
            .copied()
            .unwrap_or(self.source.len()) as u32;
        Span::new(start, end)
    }

    pub fn position(&self, byte_offset: u32) -> Position {
        let mut offset = (byte_offset as usize).min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset -= 1;
        }
        let line = self.line_starts.partition_point(|&start| start <= offset) - 1;
        let prefix = &self.source[self.line_starts[line]..offset];
        Position {
            offset: self.line_starts_utf16[line] + prefix.encode_utf16().count() as u32,
            line: line as u32 + 1,
            column: prefix.chars().count() as u32 + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LineIndex;

    #[test]
    fn converte_bytes_para_utf16_linha_e_coluna() {
        let source = "ação\nx = 😀 + y";
        let index = LineIndex::new(source);

        let y = source.find('y').unwrap() as u32;
        let position = index.position(y);
        assert_eq!(position.line, 2);
        assert_eq!(position.column, 9);
        // "ação\n" = 5 unidades; "x = " = 4; 😀 = 2; " + " = 3.
        assert_eq!(position.offset, 14);

        assert_eq!(index.position(0).offset, 0);
        assert_eq!(index.position(source.len() as u32).line, 2);
    }
}

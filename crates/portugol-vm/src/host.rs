//! Recursos que dependem do ambiente onde o programa roda (IDE, terminal,
//! testes): diálogos, janela gráfica, teclado, mouse e som. A VM chama estes
//! métodos; quem embute a VM fornece a implementação com [`crate::Vm::with_host`].
//!
//! A implementação padrão de cada método devolve um erro dizendo que o
//! recurso não está disponível neste ambiente.

/// Filtro de tipos de arquivo de `Arquivos.selecionar_arquivo`, no formato
/// `"Descrição|ext1,ext2"` do original.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    pub description: String,
    pub extensions: Vec<String>,
}

pub trait Host {
    /// Diálogo de seleção de arquivo. `Ok(None)` quando o usuário cancela.
    fn select_file(
        &mut self,
        filters: &[FileFilter],
        accept_all: bool,
    ) -> Result<Option<String>, String> {
        let _ = (filters, accept_all);
        Err(unavailable("a seleção de arquivos"))
    }
}

pub(crate) fn unavailable(what: &str) -> String {
    format!("Neste ambiente não há suporte para {what}.")
}

/// Ambiente sem interface (terminal e testes).
#[derive(Debug, Default)]
pub struct NoHost;

impl Host for NoHost {}

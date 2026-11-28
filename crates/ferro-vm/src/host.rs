//! Recursos que dependem do ambiente onde o programa roda (IDE, terminal,
//! testes): diálogos, janela gráfica, teclado, mouse e som. A VM chama estes
//! métodos; quem embute a VM fornece a implementação com [`crate::Vm::with_host`].
//!
//! A implementação padrão de cada método devolve um erro dizendo que o
//! recurso não está disponível neste ambiente.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::graphics::{InputState, WindowCommand};

/// Filtro de tipos de arquivo de `Arquivos.selecionar_arquivo`, no formato
/// `"Descrição|ext1,ext2"` do original.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    pub description: String,
    pub extensions: Vec<String>,
}

/// Um clipe de áudio carregado para o programa atual. Soltar o clipe interrompe
/// a reprodução e libera os recursos do host.
pub trait Sound {
    fn play(&mut self, looping: bool) -> Result<(), String>;
    fn pause(&mut self);
    fn stop(&mut self);
    fn duration(&self) -> Duration;
    fn position(&self) -> Duration;
    fn seek(&mut self, position: Duration) -> Result<(), String>;
    fn set_gain(&mut self, gain: f32);
}

/// Pedido de `Telas` (ADR 0018, Emenda 5). Só é atendido quando o handler
/// termina, com a janela que ele devolveu.
#[derive(Debug, Clone)]
pub enum ScreenRequest {
    /// `Telas.abrir`: a tela, os dados entregues e a função de retorno, com a
    /// forma do tipo dos dados (`Conta{nome:cadeia}`) para conferir a outra
    /// tela.
    Open {
        screen: String,
        data: Option<(crate::Value, String)>,
        callback: Option<String>,
    },
    /// `Telas.concluir` (`true`) ou `Telas.fechar` (`false`).
    Close { accept: bool },
}

pub trait Host {
    /// `Telas.abrir`, `Telas.concluir` e `Telas.fechar`.
    fn screen_request(&mut self, request: ScreenRequest) -> Result<(), String> {
        let name = match request {
            ScreenRequest::Open { .. } => "Telas.abrir",
            ScreenRequest::Close { accept: true } => "Telas.concluir",
            ScreenRequest::Close { accept: false } => "Telas.fechar",
        };
        Err(screen_only(name))
    }

    /// Decodifica um clipe a partir de um caminho já resolvido pelo Ferro.
    fn load_sound(&mut self, path: &Path) -> Result<Box<dyn Sound>, String> {
        let _ = path;
        Err(unavailable("a reprodução de áudio"))
    }

    /// Diálogo de seleção de arquivo. `Ok(None)` quando o usuário cancela.
    fn select_file(
        &mut self,
        filters: &[FileFilter],
        accept_all: bool,
    ) -> Result<Option<String>, String> {
        let _ = (filters, accept_all);
        Err(unavailable("a seleção de arquivos"))
    }

    /// `Dialogo.mensagem`: aviso com um botão OK. Só as telas desktop
    /// (ADR 0018) têm uma janela para mostrar a caixa.
    fn show_message(&mut self, text: &str) -> Result<(), String> {
        let _ = text;
        Err(screen_only("Dialogo.mensagem"))
    }

    /// `Dialogo.confirmar`: pergunta com Sim e Não.
    fn confirm(&mut self, text: &str) -> Result<bool, String> {
        let _ = text;
        Err(screen_only("Dialogo.confirmar"))
    }

    /// `Dialogo.abrir_arquivo` e `Dialogo.salvar_arquivo`. `Ok(None)` quando o
    /// usuário cancela.
    fn file_dialog(
        &mut self,
        save: bool,
        title: &str,
        filters: &[FileFilter],
    ) -> Result<Option<String>, String> {
        let _ = (title, filters);
        Err(screen_only(if save {
            "Dialogo.salvar_arquivo"
        } else {
            "Dialogo.abrir_arquivo"
        }))
    }

    /// `AreaTransferencia.copiar`: põe o texto na área de transferência.
    fn clipboard_write(&mut self, text: &str) -> Result<(), String> {
        let _ = text;
        Err(screen_only("AreaTransferencia.copiar"))
    }

    /// `AreaTransferencia.colar`: texto da área de transferência; `None` se
    /// não houver texto.
    fn clipboard_read(&mut self) -> Result<Option<String>, String> {
        Err(screen_only("AreaTransferencia.colar"))
    }

    /// `Notificacao.mostrar`: aviso do sistema, fora da janela.
    fn notify(&mut self, title: &str, text: &str) -> Result<(), String> {
        let _ = (title, text);
        Err(screen_only("Notificacao.mostrar"))
    }

    /// Abre (ou mostra de novo) a janela gráfica com a área de desenho no
    /// tamanho dado.
    fn graphics_open(&mut self, width: i32, height: i32) -> Result<(), String> {
        let _ = (width, height);
        Err(unavailable("o modo gráfico"))
    }

    /// Esconde a janela (`encerrar_modo_grafico`, `fechar_janela` e fim do
    /// programa).
    fn graphics_close(&mut self) {}

    /// Mostra um quadro (protocolo em [`crate::graphics`]). Deve esperar o
    /// quadro anterior ser exibido, como o `renderizar()` do original.
    fn graphics_present(&mut self, frame: Vec<u8>) -> Result<(), String> {
        let _ = frame;
        Err(unavailable("o modo gráfico"))
    }

    /// Desenha as operações num quadro fora da tela e devolve os pixels RGBA
    /// (`renderizar_imagem`).
    fn graphics_snapshot(
        &mut self,
        frame: Vec<u8>,
        width: i32,
        height: i32,
    ) -> Result<Vec<u8>, String> {
        let _ = (frame, width, height);
        Err(unavailable("o modo gráfico"))
    }

    fn graphics_window(&mut self, command: WindowCommand) -> Result<(), String> {
        let _ = command;
        Err(unavailable("o modo gráfico"))
    }

    /// Resolução da tela principal em pixels.
    fn screen_size(&mut self) -> Result<(i32, i32), String> {
        Err(unavailable("o modo gráfico"))
    }

    /// Teclado e mouse da janela gráfica; `None` se não houver janela.
    fn input(&mut self) -> Option<Arc<InputState>> {
        None
    }
}

fn screen_only(function: &str) -> String {
    format!(
        "`{function}` só funciona numa tela desktop (um projeto com `.design`); num programa de console, use `escreva` e `leia`."
    )
}

pub(crate) fn unavailable(what: &str) -> String {
    format!("Neste ambiente não há suporte para {what}.")
}

/// Ambiente sem interface (terminal e testes).
#[derive(Debug, Default)]
pub struct NoHost;

impl Host for NoHost {}

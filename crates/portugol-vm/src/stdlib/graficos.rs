//! `Graficos.java`: janela de desenho, formas, texto e imagens.
//!
//! As funções de desenho acumulam operações (protocolo em
//! [`crate::graphics`]) que `renderizar()` envia à janela do host. Imagens
//! ficam na VM (crate `image`) e são enviadas à janela na primeira vez em que
//! são desenhadas. As regras de desenho reproduzem o original:
//! * o texto é desenhado com a linha de base em
//!   `y + ascendente - descendente + entrelinha + 1`;
//! * a rotação vale para as formas seguintes e gira cada uma em torno do
//!   próprio centro;
//! * cantos arredondados têm raio de 20% do menor lado;
//! * a opacidade entra na cor; o gradiente vale até a próxima `definir_cor`;
//! * imagens totalmente fora da janela não são desenhadas.
//!
//! Diferença: `salvar_imagem` grava o PNG (no original ela criava uma pasta
//! com o nome do arquivo e falhava).

use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;

use image::{Rgba, RgbaImage};

use super::fontes::{self, Fonts};
use super::{NativeContext, NativeFn, arity, boolean, int, real, text};
use crate::graphics::{FrameWriter, WindowCommand, op};
use crate::value::Value;

const MAX_IMAGES: usize = 128;
const MAX_OPERATIONS: usize = 131_072;
const DEFAULT_WIDTH: i32 = 640;
const DEFAULT_HEIGHT: i32 = 480;

const NOT_INITIALIZED: &str = "O modo gráfico não foi inicializado";
const TOO_MANY_OPERATIONS: &str = "A função Graficos.renderizar() não foi chamada nenhuma vez em \
     seu código ou o número de operações de desenho que você chamou antes do \
     Graficos.renderizar() foi muito grande";

struct Frame {
    uid: u32,
    pixels: Arc<RgbaImage>,
    /// Intervalo do quadro de GIF em centésimos de segundo.
    delay_cs: i32,
}

struct Picture {
    frames: Vec<Frame>,
    gif: bool,
    current: usize,
    last_draw_ms: u64,
}

pub struct Graphics {
    initialized: bool,
    width: i32,
    height: i32,
    windowed_size: Option<(i32, i32)>,
    ops: FrameWriter,
    op_count: usize,
    color: u32,
    opacity: u8,
    rotation: f64,
    family: String,
    style: u8,
    size: f32,
    fonts: Fonts,
    builtin_fonts_sent: bool,
    pending_fonts: Vec<(String, Arc<Vec<u8>>)>,
    images: Vec<Option<Picture>>,
    next_uid: u32,
    uploaded: HashSet<u32>,
    needed: Vec<(u32, Arc<RgbaImage>)>,
    released: Vec<u32>,
}

impl Default for Graphics {
    fn default() -> Self {
        Graphics {
            initialized: false,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            windowed_size: None,
            ops: FrameWriter::default(),
            op_count: 0,
            color: 0x000000,
            opacity: 255,
            rotation: 0.0,
            family: fontes::DEFAULT_FAMILY.to_string(),
            style: 0,
            size: fontes::DEFAULT_SIZE,
            fonts: Fonts::default(),
            builtin_fonts_sent: false,
            pending_fonts: Vec::new(),
            images: (0..MAX_IMAGES).map(|_| None).collect(),
            next_uid: 1,
            uploaded: HashSet::new(),
            needed: Vec::new(),
            released: Vec::new(),
        }
    }
}

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "iniciar_modo_grafico" => iniciar_modo_grafico,
        "encerrar_modo_grafico" => encerrar_modo_grafico,
        "fechar_janela" => encerrar_modo_grafico,
        "minimizar_janela" => |c, a| window(c, a, "minimizar_janela", |_| WindowCommand::Minimize),
        "restaurar_janela" => |c, a| window(c, a, "restaurar_janela", |_| WindowCommand::Restore),
        "ocultar_borda_janela" => |c, a| {
            window(c, a, "ocultar_borda_janela", |_| {
                WindowCommand::Decorations(false)
            })
        },
        "exibir_borda_janela" => |c, a| {
            window(c, a, "exibir_borda_janela", |_| {
                WindowCommand::Decorations(true)
            })
        },
        "definir_dimensoes_janela" => definir_dimensoes_janela,
        "definir_titulo_janela" => definir_titulo_janela,
        "entrar_modo_tela_cheia" => entrar_modo_tela_cheia,
        "sair_modo_tela_cheia" => sair_modo_tela_cheia,
        "definir_icone_janela" => definir_icone_janela,
        "largura_janela" => largura_janela,
        "altura_janela" => altura_janela,
        "largura_tela" => |c, a| screen(c, a, "largura_tela", |(w, _)| w),
        "altura_tela" => |c, a| screen(c, a, "altura_tela", |(_, h)| h),
        "limpar" => limpar,
        "renderizar" => renderizar,
        "renderizar_imagem" => renderizar_imagem,
        "definir_cor" => definir_cor,
        "definir_gradiente" => definir_gradiente,
        "definir_opacidade" => definir_opacidade,
        "definir_rotacao" => definir_rotacao,
        "criar_cor" => criar_cor,
        "obter_RGB" => obter_rgb,
        "desenhar_retangulo" => desenhar_retangulo,
        "desenhar_elipse" => desenhar_elipse,
        "desenhar_linha" => desenhar_linha,
        "desenhar_ponto" => desenhar_ponto,
        "desenhar_poligono" => desenhar_poligono,
        "desenhar_texto" => desenhar_texto,
        "definir_fonte_texto" => definir_fonte_texto,
        "definir_tamanho_texto" => definir_tamanho_texto,
        "definir_estilo_texto" => definir_estilo_texto,
        "carregar_fonte" => carregar_fonte,
        "largura_texto" => largura_texto,
        "altura_texto" => altura_texto,
        "carregar_imagem" => carregar_imagem,
        "liberar_imagem" => liberar_imagem,
        "largura_imagem" => |c, a| image_size(c, a, "largura_imagem", |i| i.width()),
        "altura_imagem" => |c, a| image_size(c, a, "altura_imagem", |i| i.height()),
        "desenhar_imagem" => desenhar_imagem,
        "desenhar_quadro_atual_gif" => desenhar_imagem,
        "desenhar_porcao_imagem" => desenhar_porcao_imagem,
        "transformar_imagem" => transformar_imagem,
        "transformar_porcao_imagem" => transformar_porcao_imagem,
        "redimensionar_imagem" => redimensionar_imagem,
        "obter_cor_pixel" => obter_cor_pixel,
        "salvar_imagem" => salvar_imagem,
        "proximo_frame_gif" => proximo_frame_gif,
        "obter_intervalo_gif" => obter_intervalo_gif,
        "obter_numero_quadros_gif" => obter_numero_quadros_gif,
        "obter_numero_quadro_atual_gif" => obter_numero_quadro_atual_gif,
        "obter_quadro_gif" => obter_quadro_gif,
        "definir_quadro_gif" => definir_quadro_gif,
        _ => return None,
    })
}

pub fn constant(name: &str) -> Option<Value> {
    let value = match name {
        "COR_PRETO" => 0xFF00_0000_u32 as i32,
        "COR_BRANCO" => 0xFFFF_FFFF_u32 as i32,
        "COR_AZUL" => 0xFF00_00FF_u32 as i32,
        "COR_VERMELHO" => 0xFFFF_0000_u32 as i32,
        "COR_VERDE" => 0xFF00_FF00_u32 as i32,
        "COR_AMARELO" => 0xFFFF_FF00_u32 as i32,
        "GRADIENTE_DIREITA" => 0,
        "GRADIENTE_ESQUERDA" => 1,
        "GRADIENTE_ACIMA" => 2,
        "GRADIENTE_ABAIXO" => 3,
        "GRADIENTE_INFERIOR_DIREITO" => 4,
        "GRADIENTE_INFERIOR_ESQUERDO" => 5,
        "GRADIENTE_SUPERIOR_DIREITO" => 6,
        "GRADIENTE_SUPERIOR_ESQUERDO" => 7,
        "CANAL_R" => 0,
        "CANAL_G" => 1,
        "CANAL_B" => 2,
        _ => return None,
    };
    Some(Value::Int(value))
}

// ---------------------------------------------------------------------
// Janela
// ---------------------------------------------------------------------

fn require_window(ctx: &NativeContext) -> Result<(), String> {
    if ctx.graphics.initialized {
        Ok(())
    } else {
        Err(NOT_INITIALIZED.into())
    }
}

fn iniciar_modo_grafico(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "iniciar_modo_grafico")?;
    let (width, height) = (ctx.graphics.width, ctx.graphics.height);
    ctx.host.graphics_open(width, height)?;
    ctx.graphics.initialized = true;
    Ok(Value::Void)
}

fn encerrar_modo_grafico(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "encerrar_modo_grafico")?;
    require_window(ctx)?;
    ctx.host.graphics_close();
    ctx.graphics.initialized = false;
    // Imagens enviadas à janela anterior não estão na próxima.
    ctx.graphics.uploaded.clear();
    ctx.graphics.builtin_fonts_sent = false;
    Ok(Value::Void)
}

fn window(
    ctx: &mut NativeContext,
    args: &[Value],
    name: &str,
    command: fn(&[Value]) -> WindowCommand,
) -> Result<Value, String> {
    arity(args, 0, name)?;
    require_window(ctx)?;
    ctx.host.graphics_window(command(args))?;
    Ok(Value::Void)
}

fn definir_dimensoes_janela(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "definir_dimensoes_janela")?;
    require_window(ctx)?;
    let (width, height) = (int(&args[0])?, int(&args[1])?);
    ctx.host
        .graphics_window(WindowCommand::Resize { width, height })?;
    ctx.graphics.width = width;
    ctx.graphics.height = height;
    Ok(Value::Void)
}

fn definir_titulo_janela(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "definir_titulo_janela")?;
    require_window(ctx)?;
    let title = text(&args[0])?.to_string();
    ctx.host.graphics_window(WindowCommand::Title(title))?;
    Ok(Value::Void)
}

fn entrar_modo_tela_cheia(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "entrar_modo_tela_cheia")?;
    require_window(ctx)?;
    let screen = ctx.host.screen_size()?;
    ctx.host.graphics_window(WindowCommand::FullScreen(true))?;
    let g = &mut ctx.graphics;
    g.windowed_size.get_or_insert((g.width, g.height));
    (g.width, g.height) = screen;
    Ok(Value::Void)
}

fn sair_modo_tela_cheia(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "sair_modo_tela_cheia")?;
    require_window(ctx)?;
    ctx.host.graphics_window(WindowCommand::FullScreen(false))?;
    if let Some((w, h)) = ctx.graphics.windowed_size.take() {
        (ctx.graphics.width, ctx.graphics.height) = (w, h);
    }
    Ok(Value::Void)
}

fn definir_icone_janela(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "definir_icone_janela")?;
    require_window(ctx)?;
    let pixels = current_pixels(ctx, int(&args[0])?)?;
    ctx.host.graphics_window(WindowCommand::Icon {
        width: pixels.width(),
        height: pixels.height(),
        rgba: pixels.as_raw().clone(),
    })?;
    Ok(Value::Void)
}

fn largura_janela(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "largura_janela")?;
    require_window(ctx)?;
    Ok(Value::Int(ctx.graphics.width))
}

fn altura_janela(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "altura_janela")?;
    require_window(ctx)?;
    Ok(Value::Int(ctx.graphics.height))
}

fn screen(
    ctx: &mut NativeContext,
    args: &[Value],
    name: &str,
    pick: fn((i32, i32)) -> i32,
) -> Result<Value, String> {
    arity(args, 0, name)?;
    Ok(Value::Int(pick(ctx.host.screen_size()?)))
}

// ---------------------------------------------------------------------
// Operações e quadros
// ---------------------------------------------------------------------

impl Graphics {
    fn begin(&mut self, code: u8) -> Result<&mut FrameWriter, String> {
        if self.op_count >= MAX_OPERATIONS {
            return Err(TOO_MANY_OPERATIONS.into());
        }
        self.op_count += 1;
        self.ops.op(code);
        Ok(&mut self.ops)
    }

    fn argb(&self, rgb: u32) -> u32 {
        ((self.opacity as u32) << 24) | (rgb & 0x00FF_FFFF)
    }

    fn rotation_tail(&mut self, cx: i32, cy: i32) {
        let rotation = self.rotation as f32;
        self.ops.f32(rotation).i32(cx).i32(cy);
    }

    fn use_frame(&mut self, uid: u32, pixels: &Arc<RgbaImage>) {
        if !self.uploaded.contains(&uid) && !self.needed.iter().any(|(u, _)| *u == uid) {
            self.needed.push((uid, Arc::clone(pixels)));
        }
    }

    /// Mensagem completa: fontes, imagens novas e liberadas, depois as
    /// operações. Zera a lista de operações.
    fn take_frame(&mut self) -> Vec<u8> {
        let mut out = FrameWriter::default();
        if !self.builtin_fonts_sent {
            for (bytes, style) in fontes::builtin_files() {
                out.op(op::LOAD_FONT)
                    .str(fontes::BUILTIN_FAMILY)
                    .u8(style)
                    .u32(bytes.len() as u32)
                    .raw(bytes);
            }
            self.builtin_fonts_sent = true;
        }
        for (family, bytes) in self.pending_fonts.drain(..) {
            out.op(op::LOAD_FONT)
                .str(&family)
                .u8(0)
                .u32(bytes.len() as u32)
                .raw(&bytes);
        }
        for uid in self.released.drain(..) {
            out.op(op::RELEASE_IMAGE).u32(uid);
        }
        for (uid, pixels) in self.needed.drain(..) {
            out.op(op::UPLOAD_IMAGE)
                .u32(uid)
                .u32(pixels.width())
                .u32(pixels.height())
                .raw(pixels.as_raw());
            self.uploaded.insert(uid);
        }
        out.raw(&self.ops.bytes);
        self.ops.bytes.clear();
        self.op_count = 0;
        out.bytes
    }
}

fn limpar(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "limpar")?;
    require_window(ctx)?;
    let g = &mut ctx.graphics;
    let (w, h) = (g.width, g.height);
    g.begin(op::CLEAR)?.i32(w).i32(h);
    Ok(Value::Void)
}

fn renderizar(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "renderizar")?;
    require_window(ctx)?;
    let frame = ctx.graphics.take_frame();
    ctx.host.graphics_present(frame)?;
    Ok(Value::Void)
}

fn renderizar_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "renderizar_imagem")?;
    let (width, height) = (int(&args[0])?, int(&args[1])?);
    if width <= 0 || height <= 0 {
        return Err("Width (".to_string()
            + &width.to_string()
            + ") and height ("
            + &height.to_string()
            + ") cannot be <= 0");
    }
    let frame = ctx.graphics.take_frame();
    let rgba = ctx.host.graphics_snapshot(frame, width, height)?;
    let pixels = RgbaImage::from_raw(width as u32, height as u32, rgba)
        .ok_or_else(|| "A janela gráfica devolveu uma imagem inválida".to_string())?;
    let picture = still(ctx, pixels);
    add_picture(ctx, picture)
}

fn definir_cor(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "definir_cor")?;
    let color = int(&args[0])? as u32;
    let g = &mut ctx.graphics;
    g.color = color & 0x00FF_FFFF;
    let argb = g.argb(color);
    g.begin(op::COLOR)?.u32(argb);
    Ok(Value::Void)
}

fn definir_gradiente(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "definir_gradiente")?;
    let kind = int(&args[0])?;
    let (c1, c2) = (int(&args[1])? as u32, int(&args[2])? as u32);
    let g = &mut ctx.graphics;
    let (c1, c2) = (g.argb(c1), g.argb(c2));
    g.begin(op::GRADIENT)?.u8(kind as u8).u32(c1).u32(c2);
    Ok(Value::Void)
}

fn definir_opacidade(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "definir_opacidade")?;
    let opacity = int(&args[0])?;
    if !(0..=255).contains(&opacity) {
        return Err("O valor da opacidade deve esta entre 0 e 255".into());
    }
    let g = &mut ctx.graphics;
    g.opacity = opacity as u8;
    let argb = g.argb(g.color);
    g.begin(op::COLOR)?.u32(argb);
    Ok(Value::Void)
}

fn definir_rotacao(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "definir_rotacao")?;
    ctx.graphics.rotation = ((int(&args[0])? % 360) as f64).to_radians();
    Ok(Value::Void)
}

fn criar_cor(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "criar_cor")?;
    let (r, g, b) = (int(&args[0])?, int(&args[1])?, int(&args[2])?);
    if [r, g, b].iter().any(|c| !(0..=255).contains(c)) {
        return Err("Erro ao criar a cor, os valor dos tons deve estar entre 0 e 255".into());
    }
    Ok(Value::Int(
        (0xFF00_0000_u32 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32) as i32,
    ))
}

fn obter_rgb(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "obter_RGB")?;
    let color = int(&args[0])?;
    Ok(Value::Int(match int(&args[1])? {
        0 => (color & 0xff0000) >> 16,
        1 => (color & 0x00ff00) >> 8,
        2 => color & 0x0000ff,
        _ => 0,
    }))
}

// ---------------------------------------------------------------------
// Formas e texto
// ---------------------------------------------------------------------

fn desenhar_retangulo(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 6, "desenhar_retangulo")?;
    let (x, y, w, h) = (
        int(&args[0])?,
        int(&args[1])?,
        int(&args[2])?,
        int(&args[3])?,
    );
    let (round, fill) = (boolean(&args[4])?, boolean(&args[5])?);
    let g = &mut ctx.graphics;
    g.begin(op::RECT)?
        .i32(x)
        .i32(y)
        .i32(w)
        .i32(h)
        .u8(round as u8)
        .u8(fill as u8);
    g.rotation_tail(x + (w >> 1), y + (h >> 1));
    Ok(Value::Void)
}

fn desenhar_elipse(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 5, "desenhar_elipse")?;
    let (x, y, w, h) = (
        int(&args[0])?,
        int(&args[1])?,
        int(&args[2])?,
        int(&args[3])?,
    );
    let fill = boolean(&args[4])?;
    let g = &mut ctx.graphics;
    g.begin(op::ELLIPSE)?
        .i32(x)
        .i32(y)
        .i32(w)
        .i32(h)
        .u8(fill as u8);
    g.rotation_tail(x + (w >> 1), y + (h >> 1));
    Ok(Value::Void)
}

fn desenhar_linha(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 4, "desenhar_linha")?;
    let (x1, y1, x2, y2) = (
        int(&args[0])?,
        int(&args[1])?,
        int(&args[2])?,
        int(&args[3])?,
    );
    let g = &mut ctx.graphics;
    g.begin(op::LINE)?.i32(x1).i32(y1).i32(x2).i32(y2);
    g.rotation_tail(x1 + ((x1 - x2).abs() >> 1), y1 + ((y1 - y2).abs() >> 1));
    Ok(Value::Void)
}

fn desenhar_ponto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "desenhar_ponto")?;
    let (x, y) = (int(&args[0])?, int(&args[1])?);
    ctx.graphics.begin(op::POINT)?.i32(x).i32(y);
    Ok(Value::Void)
}

fn desenhar_poligono(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "desenhar_poligono")?;
    let Value::Matrix(matrix) = &args[0] else {
        return Err("\"desenhar_poligono\" espera uma matriz de pontos".into());
    };
    let fill = boolean(&args[1])?;
    let points: Vec<(i32, i32)> = {
        let matrix = matrix.borrow();
        if matrix.cols < 2 {
            return Err("Você tentou acessar um índice de vetor ou matriz inválido.".into());
        }
        (0..matrix.rows)
            .map(|r| {
                let row = &matrix.data[r * matrix.cols..];
                Ok((int(&row[0])?, int(&row[1])?))
            })
            .collect::<Result<_, String>>()?
    };
    let n = points.len().max(1) as i64;
    let cx = (points.iter().map(|p| p.0 as i64).sum::<i64>() / n) as i32;
    let cy = (points.iter().map(|p| p.1 as i64).sum::<i64>() / n) as i32;
    let g = &mut ctx.graphics;
    let w = g.begin(op::POLYGON)?;
    w.u32(points.len() as u32);
    for (x, y) in &points {
        w.i32(*x).i32(*y);
    }
    w.u8(fill as u8);
    g.rotation_tail(cx, cy);
    Ok(Value::Void)
}

fn desenhar_texto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "desenhar_texto")?;
    let (x, y) = (int(&args[0])?, int(&args[1])?);
    let content = text(&args[2])?.to_string();
    let g = &mut ctx.graphics;
    let (family, style, size) = (g.family.clone(), g.style, g.size);
    let metrics = g.fonts.metrics(&family, style, size);
    let width = g.fonts.width(&family, style, size, &content);
    let height = metrics.ascent - metrics.descent + metrics.leading + 1;
    g.begin(op::TEXT)?.i32(x).i32(y + height).str(&content);
    g.rotation_tail(x + (width >> 1), y + (height >> 1));
    Ok(Value::Void)
}

fn emit_font(g: &mut Graphics) -> Result<(), String> {
    let (family, size, style) = (g.family.clone(), g.size, g.style);
    g.begin(op::FONT)?.str(&family).f32(size).u8(style);
    Ok(())
}

fn definir_fonte_texto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "definir_fonte_texto")?;
    require_window(ctx)?;
    ctx.graphics.family = text(&args[0])?.to_string();
    emit_font(&mut ctx.graphics)?;
    Ok(Value::Void)
}

/// `Font.deriveFont(float)`; o original não verifica o modo gráfico aqui.
fn definir_tamanho_texto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "definir_tamanho_texto")?;
    ctx.graphics.size = real(&args[0])? as f32;
    emit_font(&mut ctx.graphics)?;
    Ok(Value::Void)
}

fn definir_estilo_texto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "definir_estilo_texto")?;
    require_window(ctx)?;
    let (italic, bold, underline) = (boolean(&args[0])?, boolean(&args[1])?, boolean(&args[2])?);
    // `new Font(nome, estilo, (int) tamanho)`: o tamanho perde a parte decimal.
    let g = &mut ctx.graphics;
    g.size = g.size.trunc();
    g.style = (italic as u8 * fontes::ITALIC)
        | (bold as u8 * fontes::BOLD)
        | (underline as u8 * fontes::UNDERLINE);
    emit_font(g)?;
    Ok(Value::Void)
}

fn carregar_fonte(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "carregar_fonte")?;
    let raw = text(&args[0])?;
    let path = super::arquivos::resolve(ctx, raw);
    if !path.exists() {
        return Err(format!("A fonte '{raw}' não foi encontrada"));
    }
    let failure = || format!("Não foi possível carregar a fonte '{}'", path.display());
    let bytes = std::fs::read(&path).map_err(|_| failure())?;
    let loaded = ctx.graphics.fonts.load(bytes).map_err(|_| failure())?;
    let entry = (loaded.family.clone(), Arc::clone(&loaded.bytes));
    ctx.graphics.pending_fonts.push(entry);
    Ok(Value::Void)
}

fn largura_texto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "largura_texto")?;
    let content = text(&args[0])?;
    let g = &mut ctx.graphics;
    let (family, style, size) = (g.family.clone(), g.style, g.size);
    Ok(Value::Int(g.fonts.width(&family, style, size, content)))
}

fn altura_texto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "altura_texto")?;
    let g = &mut ctx.graphics;
    let (family, style, size) = (g.family.clone(), g.style, g.size);
    let metrics = g.fonts.metrics(&family, style, size);
    Ok(Value::Int(metrics.ascent + metrics.leading))
}

// ---------------------------------------------------------------------
// Imagens
// ---------------------------------------------------------------------

fn still(ctx: &mut NativeContext, pixels: RgbaImage) -> Picture {
    Picture {
        frames: vec![new_frame(ctx, pixels, 0)],
        gif: false,
        current: 0,
        last_draw_ms: 0,
    }
}

fn new_frame(ctx: &mut NativeContext, pixels: RgbaImage, delay_cs: i32) -> Frame {
    let uid = ctx.graphics.next_uid;
    ctx.graphics.next_uid += 1;
    Frame {
        uid,
        pixels: Arc::new(pixels),
        delay_cs,
    }
}

/// O primeiro endereço livre a partir do fim, como no `CacheImagens`.
fn add_picture(ctx: &mut NativeContext, picture: Picture) -> Result<Value, String> {
    let Some(index) = (0..MAX_IMAGES)
        .rev()
        .find(|&i| ctx.graphics.images[i].is_none())
    else {
        return Err("O número máximo de imagens que podem ser carregadas foi atingido".into());
    };
    ctx.graphics.images[index] = Some(picture);
    Ok(Value::Int(index as i32))
}

fn picture(ctx: &mut NativeContext, address: i32) -> Result<&mut Picture, String> {
    usize::try_from(address)
        .ok()
        .and_then(|i| ctx.graphics.images.get_mut(i))
        .and_then(Option::as_mut)
        .ok_or_else(|| "O endereço de memória especificado não aponta para uma imagem".to_string())
}

fn gif(ctx: &mut NativeContext, address: i32) -> Result<&mut Picture, String> {
    let picture = picture(ctx, address)?;
    if picture.gif {
        Ok(picture)
    } else {
        Err("O endereço de memória especificado aponta para uma imagem, mas esta imagem não é um GIF animado".into())
    }
}

fn current_pixels(ctx: &mut NativeContext, address: i32) -> Result<Arc<RgbaImage>, String> {
    let picture = picture(ctx, address)?;
    Ok(Arc::clone(&picture.frames[picture.current].pixels))
}

fn carregar_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "carregar_imagem")?;
    let raw = text(&args[0])?;
    let path = super::arquivos::resolve(ctx, raw);
    if !path.exists() {
        return Err(format!("A imagem '{raw}' não foi encontrada"));
    }
    let failure = || format!("Ocorreu um erro ao carregar a imagem '{raw}'");
    let bytes = std::fs::read(&path).map_err(|_| failure())?;
    let decoded = decode(&bytes).ok_or_else(failure)?;
    let gif = decoded.len() > 1;
    let frames = decoded
        .into_iter()
        .map(|(pixels, delay)| new_frame(ctx, pixels, delay))
        .collect();
    add_picture(
        ctx,
        Picture {
            frames,
            gif,
            current: 0,
            last_draw_ms: 0,
        },
    )
}

/// Quadros e intervalos (centésimos de segundo). GIFs com um só quadro são
/// tratados como imagens comuns.
fn decode(bytes: &[u8]) -> Option<Vec<(RgbaImage, i32)>> {
    if image::guess_format(bytes).ok()? == image::ImageFormat::Gif {
        use image::AnimationDecoder;
        let decoder = image::codecs::gif::GifDecoder::new(std::io::Cursor::new(bytes)).ok()?;
        let frames = decoder.into_frames().collect_frames().ok()?;
        if !frames.is_empty() {
            return Some(
                frames
                    .into_iter()
                    .map(|frame| {
                        let (num, den) = frame.delay().numer_denom_ms();
                        let ms = num.checked_div(den).unwrap_or(0);
                        (frame.into_buffer(), (ms / 10) as i32)
                    })
                    .collect(),
            );
        }
    }
    Some(vec![(image::load_from_memory(bytes).ok()?.to_rgba8(), 0)])
}

fn liberar_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "liberar_imagem")?;
    let address = int(&args[0])?;
    let g = &mut ctx.graphics;
    if let Some(slot) = usize::try_from(address)
        .ok()
        .and_then(|i| g.images.get_mut(i))
        && let Some(picture) = slot.take()
    {
        for frame in picture.frames {
            g.needed.retain(|(uid, _)| *uid != frame.uid);
            if g.uploaded.remove(&frame.uid) {
                g.released.push(frame.uid);
            }
        }
    }
    Ok(Value::Void)
}

fn image_size(
    ctx: &mut NativeContext,
    args: &[Value],
    name: &str,
    pick: fn(&RgbaImage) -> u32,
) -> Result<Value, String> {
    arity(args, 1, name)?;
    let pixels = current_pixels(ctx, int(&args[0])?)?;
    Ok(Value::Int(pick(&pixels) as i32))
}

/// Quadro a desenhar; GIFs avançam sozinhos conforme o intervalo do quadro.
fn frame_to_draw(ctx: &mut NativeContext, address: i32) -> Result<(u32, Arc<RgbaImage>), String> {
    let now = ctx.elapsed_ms() + 1;
    let picture = picture(ctx, address)?;
    if picture.gif {
        let interval = picture.frames[picture.current].delay_cs as u64 * 10;
        if now.saturating_sub(picture.last_draw_ms) >= interval {
            picture.current = (picture.current + 1) % picture.frames.len();
            picture.last_draw_ms = now;
        }
    }
    let frame = &picture.frames[picture.current];
    Ok((frame.uid, Arc::clone(&frame.pixels)))
}

fn desenhar_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "desenhar_imagem")?;
    let (x, y) = (int(&args[0])?, int(&args[1])?);
    let (uid, pixels) = frame_to_draw(ctx, int(&args[2])?)?;
    let (w, h) = (pixels.width() as i32, pixels.height() as i32);
    let g = &mut ctx.graphics;
    if !(y + h > 0 && y < g.height && x < g.width && x + w > 0) {
        return Ok(Value::Void);
    }
    g.use_frame(uid, &pixels);
    let opacity = g.opacity;
    g.begin(op::IMAGE)?.u32(uid).i32(x).i32(y).u8(opacity);
    g.rotation_tail(x + (w >> 1), y + (h >> 1));
    Ok(Value::Void)
}

fn desenhar_porcao_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 7, "desenhar_porcao_imagem")?;
    let values: Vec<i32> = args[..6].iter().map(int).collect::<Result<_, _>>()?;
    let [x, y, xi, yi, w, h] = values[..] else {
        unreachable!("seis inteiros")
    };
    let (uid, pixels) = frame_to_draw(ctx, int(&args[6])?)?;
    let g = &mut ctx.graphics;
    g.use_frame(uid, &pixels);
    let opacity = g.opacity;
    g.begin(op::IMAGE_PART)?
        .u32(uid)
        .i32(x)
        .i32(y)
        .i32(xi)
        .i32(yi)
        .i32(w)
        .i32(h)
        .u8(opacity);
    g.rotation_tail(x + (w >> 1), y + (h >> 1));
    Ok(Value::Void)
}

fn obter_cor_pixel(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "obter_cor_pixel")?;
    let pixels = current_pixels(ctx, int(&args[0])?)?;
    let (x, y) = (int(&args[1])?, int(&args[2])?);
    if x < 0 || y < 0 || x as u32 >= pixels.width() || y as u32 >= pixels.height() {
        return Err("Você tentou acessar um índice de vetor ou matriz inválido.".into());
    }
    let Rgba([r, g, b, a]) = *pixels.get_pixel(x as u32, y as u32);
    Ok(Value::Int(
        (((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32) as i32,
    ))
}

fn transformar_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 5, "transformar_imagem")?;
    let pixels = current_pixels(ctx, int(&args[0])?)?;
    let transformed = transform(
        (*pixels).clone(),
        boolean(&args[1])?,
        boolean(&args[2])?,
        int(&args[3])?,
        int(&args[4])?,
    );
    let picture = still(ctx, transformed);
    add_picture(ctx, picture)
}

fn transformar_porcao_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 9, "transformar_porcao_imagem")?;
    let pixels = current_pixels(ctx, int(&args[0])?)?;
    let (x, y, w, h) = (
        int(&args[1])?,
        int(&args[2])?,
        int(&args[3])?,
        int(&args[4])?,
    );
    if w <= 0 || h <= 0 {
        return Err(format!("Width ({w}) and height ({h}) cannot be <= 0"));
    }
    let mut part = RgbaImage::new(w as u32, h as u32);
    for (px, py, pixel) in part.enumerate_pixels_mut() {
        let (sx, sy) = (x + px as i32, y + py as i32);
        if sx >= 0 && sy >= 0 && (sx as u32) < pixels.width() && (sy as u32) < pixels.height() {
            *pixel = *pixels.get_pixel(sx as u32, sy as u32);
        }
    }
    let transformed = transform(
        part,
        boolean(&args[5])?,
        boolean(&args[6])?,
        int(&args[7])?,
        int(&args[8])?,
    );
    let picture = still(ctx, transformed);
    add_picture(ctx, picture)
}

/// Chroma key, espelhamento e rotação, nessa ordem, como no original.
fn transform(
    mut pixels: RgbaImage,
    flip_h: bool,
    flip_v: bool,
    degrees: i32,
    ignored: i32,
) -> RgbaImage {
    if ignored != 0 {
        let key = (ignored as u32 & 0x00FF_FFFF).to_be_bytes();
        for pixel in pixels.pixels_mut() {
            if pixel.0[..3] == key[1..] {
                *pixel = Rgba([0, 0, 0, 0]);
            }
        }
    }
    if flip_h {
        image::imageops::flip_horizontal_in_place(&mut pixels);
    }
    if flip_v {
        image::imageops::flip_vertical_in_place(&mut pixels);
    }
    if degrees % 360 != 0 {
        pixels = rotate(&pixels, ((degrees % 360) as f64).to_radians());
    }
    pixels
}

/// Rotação com o mesmo enquadramento do `AffineTransformOp` do original: a
/// imagem resultante é a caixa que contém a imagem girada.
fn rotate(source: &RgbaImage, angle: f64) -> RgbaImage {
    let (w, h) = (source.width() as f64, source.height() as f64);
    let (sin, cos) = angle.sin_cos();
    let corners = [(0.0, 0.0), (w, 0.0), (w, h), (0.0, h)]
        .map(|(x, y)| (x * cos - y * sin, x * sin + y * cos));
    let min_x = corners.iter().map(|c| c.0).fold(0.0, f64::min);
    let min_y = corners.iter().map(|c| c.1).fold(0.0, f64::min);
    let max_x = corners.iter().map(|c| c.0).fold(f64::MIN, f64::max);
    let max_y = corners.iter().map(|c| c.1).fold(f64::MIN, f64::max);
    let (out_w, out_h) = (
        (max_x - min_x).round().max(1.0) as u32,
        (max_y - min_y).round().max(1.0) as u32,
    );
    let mut out = RgbaImage::new(out_w, out_h);
    for (px, py, pixel) in out.enumerate_pixels_mut() {
        // Ponto de destino de volta à origem (rotação inversa).
        let (dx, dy) = (px as f64 + 0.5 + min_x, py as f64 + 0.5 + min_y);
        let (sx, sy) = (dx * cos + dy * sin, -dx * sin + dy * cos);
        *pixel = sample(source, sx - 0.5, sy - 0.5);
    }
    out
}

/// Amostragem bilinear; fora da imagem é transparente.
fn sample(source: &RgbaImage, x: f64, y: f64) -> Rgba<u8> {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let get = |ix: f64, iy: f64| -> [f64; 4] {
        if ix < 0.0 || iy < 0.0 || ix >= source.width() as f64 || iy >= source.height() as f64 {
            [0.0; 4]
        } else {
            source.get_pixel(ix as u32, iy as u32).0.map(f64::from)
        }
    };
    let (a, b, c, d) = (
        get(x0, y0),
        get(x0 + 1.0, y0),
        get(x0, y0 + 1.0),
        get(x0 + 1.0, y0 + 1.0),
    );
    let mut out = [0u8; 4];
    for i in 0..4 {
        let top = a[i] * (1.0 - fx) + b[i] * fx;
        let bottom = c[i] * (1.0 - fx) + d[i] * fx;
        out[i] = (top * (1.0 - fy) + bottom * fy).round().clamp(0.0, 255.0) as u8;
    }
    Rgba(out)
}

fn redimensionar_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 4, "redimensionar_imagem")?;
    let address = int(&args[0])?;
    let (mut w, mut h) = (int(&args[1])?, int(&args[2])?);
    let quality = boolean(&args[3])?;
    if w <= 0 && h <= 0 {
        return Err("Impossível transformar imagem para estas dimensões".into());
    }
    let source = picture(ctx, address)?;
    let (ow, oh) = (
        source.frames[0].pixels.width() as f64,
        source.frames[0].pixels.height() as f64,
    );
    if h == 0 && w != 0 {
        h = (oh * (w as f64 / ow)) as i32;
    } else if h != 0 && w == 0 {
        w = (ow * (h as f64 / oh)) as i32;
    }
    if w <= 0 || h <= 0 {
        return Err("Impossível transformar imagem para estas dimensões".into());
    }
    // O original usa VALUE_INTERPOLATION_BICUBIC do Java. Catmull-Rom é o
    // filtro cúbico do `image` e, ao contrário de Lanczos3, não depende de
    // `sin`: produz exatamente os mesmos pixels no Windows, Linux e macOS.
    let filter = if quality {
        image::imageops::FilterType::CatmullRom
    } else {
        image::imageops::FilterType::Nearest
    };
    let gif = source.gif;
    let resized: Vec<(RgbaImage, i32)> = source
        .frames
        .iter()
        .map(|f| {
            (
                image::imageops::resize(&*f.pixels, w as u32, h as u32, filter),
                f.delay_cs,
            )
        })
        .collect();
    let frames = resized
        .into_iter()
        .map(|(pixels, delay)| new_frame(ctx, pixels, delay))
        .collect();
    add_picture(
        ctx,
        Picture {
            frames,
            gif,
            current: 0,
            last_draw_ms: 0,
        },
    )
}

fn salvar_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "salvar_imagem")?;
    let pixels = current_pixels(ctx, int(&args[0])?)?;
    let path = super::arquivos::resolve(ctx, text(&args[1])?);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    pixels
        .save_with_format(Path::new(&path), image::ImageFormat::Png)
        .map(|_| Value::Void)
        .map_err(|e| format!("Erro ao salvar a imagem: {e}"))
}

fn proximo_frame_gif(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "proximo_frame_gif")?;
    let picture = gif(ctx, int(&args[0])?)?;
    picture.current = (picture.current + 1) % picture.frames.len();
    Ok(Value::Void)
}

fn obter_intervalo_gif(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "obter_intervalo_gif")?;
    let picture = gif(ctx, int(&args[0])?)?;
    Ok(Value::Int(picture.frames[picture.current].delay_cs))
}

fn obter_numero_quadros_gif(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "obter_numero_quadros_gif")?;
    Ok(Value::Int(gif(ctx, int(&args[0])?)?.frames.len() as i32))
}

fn obter_numero_quadro_atual_gif(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "obter_numero_quadro_atual_gif")?;
    Ok(Value::Int(gif(ctx, int(&args[0])?)?.current as i32))
}

fn obter_quadro_gif(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "obter_quadro_gif")?;
    let index = int(&args[1])?;
    let picture = gif(ctx, int(&args[0])?)?;
    let count = picture.frames.len();
    let Some(frame) = usize::try_from(index)
        .ok()
        .and_then(|i| picture.frames.get(i))
    else {
        return Err(format!(
            "O índice do quadro é invalido. Esta imagem GIF possui apenas {count} quadros"
        ));
    };
    let pixels = (*frame.pixels).clone();
    let still = still(ctx, pixels);
    add_picture(ctx, still)
}

fn definir_quadro_gif(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "definir_quadro_gif")?;
    let index = int(&args[1])?;
    let picture = gif(ctx, int(&args[0])?)?;
    let count = picture.frames.len();
    if index < 0 || index as usize >= count {
        return Err(format!(
            "O índice do quadro é invalido. Esta imagem GIF possui apenas {count} quadros"
        ));
    }
    picture.current = index as usize;
    Ok(Value::Void)
}

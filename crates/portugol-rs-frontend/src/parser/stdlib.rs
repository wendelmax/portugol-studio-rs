use portugol_hir as hir;
use portugol_syntax::{Diagnostic, Span};

use super::Parser;
use super::constants::*;
use crate::lexer::Kind;

impl Parser {
    pub(super) fn ui_host_call(&mut self, start: Span) -> Result<hir::Expr, Vec<Diagnostic>> {
        self.advance(); // '.'
        let (component, component_span) = self.identifier()?;
        self.expect(Kind::LParen, "Esperado `(` após componente de UI")?;
        let mut arguments = Vec::new();
        if !matches!(self.current().kind, Kind::RParen) {
            loop {
                arguments.push(self.expression(0)?);
                if !self.eat(&Kind::Comma) {
                    break;
                }
            }
        }
        let close = self.expect(Kind::RParen, "Esperado `)` após argumentos de UI")?;
        let whole_span = start.join(close.span);

        match component.as_str() {
            "texto" => {
                if arguments.is_empty() {
                    return self.fail(
                        "pors.ui-call-arity",
                        "`UI.texto` espera pelo menos 1 argumento",
                        whole_span,
                    );
                }
                for argument in &arguments {
                    if !matches!(
                        argument.ty,
                        hir::Type::String
                            | hir::Type::Integer
                            | hir::Type::Real
                            | hir::Type::Boolean
                            | hir::Type::Character
                    ) {
                        return self.fail(
                            "pors.ui-call-type-mismatch",
                            format!(
                                "`UI.texto` espera valores escalares (cadeia, inteiro, real, logico ou caracter), recebeu `{}`",
                                type_label(argument.ty)
                            ),
                            syntax_span(argument.span),
                        );
                    }
                }
            }
            "botao" => {
                if arguments.len() != 2 {
                    return self.fail(
                        "pors.ui-call-arity",
                        format!(
                            "`UI.botao` espera 2 argumentos (id, rotulo), recebeu {}",
                            arguments.len()
                        ),
                        whole_span,
                    );
                }
                for (index, label) in [(0, "id"), (1, "rotulo")] {
                    if arguments[index].ty != hir::Type::String {
                        return self.fail(
                            "pors.ui-call-type-mismatch",
                            format!("O argumento `{label}` de `UI.botao` deve ter tipo `cadeia`"),
                            syntax_span(arguments[index].span),
                        );
                    }
                }
            }
            "entrada" => {
                if arguments.len() != 3 {
                    return self.fail(
                        "pors.ui-call-arity",
                        format!(
                            "`UI.entrada` espera 3 argumentos (id, valor, dica), recebeu {}",
                            arguments.len()
                        ),
                        whole_span,
                    );
                }
                for (index, label) in [(0, "id"), (1, "valor"), (2, "dica")] {
                    if arguments[index].ty != hir::Type::String {
                        return self.fail(
                            "pors.ui-call-type-mismatch",
                            format!("O argumento `{label}` de `UI.entrada` deve ter tipo `cadeia`"),
                            syntax_span(arguments[index].span),
                        );
                    }
                }
            }
            "coluna" | "linha" | "lista" => {
                if arguments.len() != 1 {
                    return self.fail(
                        "pors.ui-call-arity",
                        format!(
                            "`UI.{component}` espera 1 argumento (vetor<inteiro>), recebeu {}",
                            arguments.len()
                        ),
                        whole_span,
                    );
                }
                if arguments[0].ty != hir::Type::VectorInteger {
                    return self.fail(
                        "pors.ui-call-type-mismatch",
                        format!(
                            "`UI.{component}` espera `vetor<inteiro>`, recebeu `{}`",
                            type_label(arguments[0].ty)
                        ),
                        syntax_span(arguments[0].span),
                    );
                }
            }
            _ => {
                return self.fail(
                    "pors.unknown-ui-component",
                    format!("Componente visual desconhecido: UI.{component}"),
                    component_span,
                );
            }
        }

        Ok(hir::Expr {
            kind: hir::ExprKind::HostCall {
                module: "UI".into(),
                name: component,
                arguments,
            },
            ty: hir::Type::Integer,
            span: to_hir_span(whole_span),
        })
    }

    pub(super) fn stdlib_statement_call(&mut self) -> Result<hir::Stmt, Vec<Diagnostic>> {
        let (module, start) = self.identifier()?;
        let expr = self.stdlib_member_or_call(&module, start)?;
        let end = self.expect(Kind::Semicolon, "Esperado `;` após chamada de biblioteca")?;
        let hir::ExprKind::HostCall {
            module,
            name,
            arguments,
        } = expr.kind
        else {
            return self.fail(
                "pors.expected-statement",
                "Esperada chamada de função",
                syntax_span(expr.span),
            );
        };
        Ok(hir::Stmt {
            kind: hir::StmtKind::HostCall {
                module,
                name,
                arguments,
            },
            span: to_hir_span(start.join(end.span)),
        })
    }

    pub(super) fn stdlib_member_or_call(
        &mut self,
        module: &str,
        start: Span,
    ) -> Result<hir::Expr, Vec<Diagnostic>> {
        enum StdlibReturn {
            Scalar(hir::Type),
            Option(hir::Type),
            ResultString(hir::Type),
        }

        self.advance(); // '.'
        let (member, member_span) = self.identifier()?;
        if module == "Matematica"
            && !matches!(self.current().kind, Kind::LParen)
            && let Some(value) = match member.as_str() {
                "PI" => Some(std::f64::consts::PI),
                "E" => Some(std::f64::consts::E),
                _ => None,
            }
        {
            return Ok(hir::Expr {
                kind: hir::ExprKind::Real(value),
                ty: hir::Type::Real,
                span: to_hir_span(start.join(member_span)),
            });
        }
        if module == "Calendario"
            && !matches!(self.current().kind, Kind::LParen)
            && let Some(value) = calendario_constant(&member)
        {
            return Ok(hir::Expr {
                kind: hir::ExprKind::Integer(value),
                ty: hir::Type::Integer,
                span: to_hir_span(start.join(member_span)),
            });
        }
        if !matches!(self.current().kind, Kind::LParen)
            && let Some(value) = match module {
                "Graficos" => graficos_constant(&member),
                "Teclado" => teclado_constant(&member),
                "Mouse" => mouse_constant(&member),
                _ => None,
            }
        {
            return Ok(hir::Expr {
                kind: hir::ExprKind::Integer(value),
                ty: hir::Type::Integer,
                span: to_hir_span(start.join(member_span)),
            });
        }
        self.expect(
            Kind::LParen,
            &format!("Esperado `(` após `{module}.{member}`"),
        )?;
        let mut arguments = Vec::new();
        if !matches!(self.current().kind, Kind::RParen) {
            loop {
                arguments.push(self.expression(0)?);
                if !self.eat(&Kind::Comma) {
                    break;
                }
            }
        }
        let close = self.expect(
            Kind::RParen,
            &format!("Esperado `)` após argumentos de `{module}.{member}`"),
        )?;
        let whole_span = start.join(close.span);

        use StdlibReturn::{Option as RetOpt, ResultString as RetRes, Scalar as Ret};
        use hir::Type::{Boolean as B, Character as C, Integer as I, Real as R, String as S};

        let signature: Option<(&[hir::Type], StdlibReturn)> = match (module, member.as_str()) {
            ("Matematica", "potencia") => Some((&[R, R], Ret(R))),
            ("Matematica", "raiz") => Some((&[R, R], Ret(R))),
            ("Matematica", "raiz_quadrada") => Some((&[R], Ret(R))),
            ("Matematica", "arredondar") => Some((&[R, I], Ret(R))),
            ("Matematica", "piso") => Some((&[R], Ret(R))),
            ("Matematica", "teto") => Some((&[R], Ret(R))),
            ("Matematica", "logaritmo") => Some((&[R, R], Ret(R))),
            ("Matematica", "seno") => Some((&[R], Ret(R))),
            ("Matematica", "cosseno") => Some((&[R], Ret(R))),
            ("Matematica", "tangente") => Some((&[R], Ret(R))),
            ("Matematica", "valor_absoluto") => Some((&[R], Ret(R))),
            ("Matematica", "valor_absoluto_inteiro") => Some((&[I], Ret(I))),
            ("Matematica", "maior_numero") => Some((&[R, R], Ret(R))),
            ("Matematica", "menor_numero") => Some((&[R, R], Ret(R))),
            ("Matematica", "maior_inteiro") => Some((&[I, I], Ret(I))),
            ("Matematica", "menor_inteiro") => Some((&[I, I], Ret(I))),

            ("Texto", "numero_caracteres") => Some((&[S], Ret(I))),
            ("Texto", "caixa_alta") => Some((&[S], Ret(S))),
            ("Texto", "caixa_baixa") => Some((&[S], Ret(S))),
            ("Texto", "aparar") => Some((&[S], Ret(S))),
            ("Texto", "contem") => Some((&[S, S], Ret(B))),
            ("Texto", "comeca_com") => Some((&[S, S], Ret(B))),
            ("Texto", "termina_com") => Some((&[S, S], Ret(B))),
            ("Texto", "substituir") => Some((&[S, S, S], Ret(S))),
            ("Texto", "preencher_a_esquerda") => Some((&[C, I, S], Ret(S))),
            ("Texto", "obter_caracter") => Some((&[S, I], RetOpt(C))),
            ("Texto", "posicao_texto") => Some((&[S, S, I], RetOpt(I))),
            ("Texto", "extrair_subtexto") => Some((&[S, I, I], RetRes(S))),
            ("Texto", "dividir") => Some((&[S, S], Ret(hir::Type::VectorString))),

            ("Tipos", "para_inteiro") => Some((&[S], RetRes(I))),
            ("Tipos", "para_inteiro_base") => Some((&[S, I], RetRes(I))),
            ("Tipos", "para_real") => Some((&[S], RetRes(R))),
            ("Tipos", "para_logico") => Some((&[S], RetRes(B))),
            ("Tipos", "para_caracter") => Some((&[S], RetRes(C))),
            ("Tipos", "inteiro_para_real") => Some((&[I], Ret(R))),
            ("Tipos", "real_para_inteiro") => Some((&[R], Ret(I))),
            ("Tipos", "inteiro_para_cadeia") => Some((&[I], Ret(S))),
            ("Tipos", "inteiro_para_cadeia_base") => Some((&[I, I], RetRes(S))),
            ("Tipos", "real_para_cadeia") => Some((&[R], Ret(S))),
            ("Tipos", "logico_para_cadeia") => Some((&[B], Ret(S))),
            ("Tipos", "caracter_para_cadeia") => Some((&[C], Ret(S))),
            ("Tipos", "logico_para_inteiro") => Some((&[B], Ret(I))),
            ("Tipos", "caracter_para_inteiro") => Some((&[C], RetRes(I))),
            ("Tipos", "inteiro_para_caracter") => Some((&[I], RetRes(C))),
            ("Tipos", "cadeia_e_inteiro") => Some((&[S], Ret(B))),
            ("Tipos", "cadeia_e_real") => Some((&[S], Ret(B))),
            ("Tipos", "cadeia_e_logico") => Some((&[S], Ret(B))),
            ("Tipos", "cadeia_e_caracter") => Some((&[S], Ret(B))),

            ("Util", "sorteia") => Some((&[I, I], RetRes(I))),
            ("Util", "definir_semente") => Some((&[I], Ret(hir::Type::Unit))),
            ("Util", "tempo_decorrido") => Some((&[], Ret(I))),
            ("Util", "aguarde") => Some((&[I], Ret(hir::Type::Unit))),

            ("Calendario", "ano_atual") => Some((&[], Ret(I))),
            ("Calendario", "mes_atual") => Some((&[], Ret(I))),
            ("Calendario", "dia_mes_atual") => Some((&[], Ret(I))),
            ("Calendario", "dia_semana_atual") => Some((&[], Ret(I))),
            ("Calendario", "hora_atual") => Some((&[B], Ret(I))),
            ("Calendario", "minuto_atual") => Some((&[], Ret(I))),
            ("Calendario", "segundo_atual") => Some((&[], Ret(I))),
            ("Calendario", "milisegundo_atual") => Some((&[], Ret(I))),
            ("Calendario", "dia_semana_completo") => Some((&[I, B, B], RetRes(S))),
            ("Calendario", "dia_semana_curto") => Some((&[I, B, B], RetRes(S))),
            ("Calendario", "dia_semana_abreviado") => Some((&[I, B, B], RetRes(S))),
            ("Calendario", "nome_mes") => Some((&[I], RetRes(S))),
            ("Calendario", "eh_bissexto") => Some((&[I], Ret(B))),
            ("Calendario", "dias_no_mes") => Some((&[I, I], RetRes(I))),
            ("Calendario", "data_iso_atual") => Some((&[], Ret(S))),

            ("Arquivos", "ler_texto") => Some((&[S], RetRes(S))),
            ("Arquivos", "ler_linhas") => Some((&[S], RetRes(hir::Type::VectorString))),
            ("Arquivos", "escrever_texto") => Some((&[S, S], RetRes(B))),
            ("Arquivos", "acrescentar_texto") => Some((&[S, S], RetRes(B))),
            ("Arquivos", "existe") => Some((&[S], RetRes(B))),
            ("Arquivos", "remover") => Some((&[S], RetRes(B))),
            ("Arquivos", "listar") => Some((&[S], RetRes(hir::Type::VectorString))),

            ("Json", "valido") => Some((&[S], Ret(B))),
            ("Json", "formatar") => Some((&[S], RetRes(S))),
            ("Json", "minificar") => Some((&[S], RetRes(S))),
            ("Json", "escapar_cadeia") => Some((&[S], Ret(S))),
            ("Json", "contem_chave") => Some((&[S, S], RetRes(B))),
            ("Json", "obter_chaves") => Some((&[S], RetRes(hir::Type::VectorString))),
            ("Json", "obter_cadeia") => Some((&[S, S], RetRes(S))),
            ("Json", "obter_inteiro") => Some((&[S, S], RetRes(I))),
            ("Json", "obter_real") => Some((&[S, S], RetRes(R))),
            ("Json", "obter_logico") => Some((&[S, S], RetRes(B))),
            ("Json", "obter_json") => Some((&[S, S], RetRes(S))),
            ("Json", "obter_lista_cadeia") => Some((&[S, S], RetRes(hir::Type::VectorString))),
            ("Json", "obter_lista_inteiro") => Some((&[S, S], RetRes(hir::Type::VectorInteger))),
            ("Json", "definir_cadeia") => Some((&[S, S, S], RetRes(S))),
            ("Json", "definir_inteiro") => Some((&[S, S, I], RetRes(S))),
            ("Json", "definir_real") => Some((&[S, S, R], RetRes(S))),
            ("Json", "definir_logico") => Some((&[S, S, B], RetRes(S))),
            ("Json", "definir_json") => Some((&[S, S, S], RetRes(S))),
            ("Json", "remover_chave") => Some((&[S, S], RetRes(S))),

            ("Internet", "definir_tempo_limite") => Some((&[I], Ret(hir::Type::Unit))),
            ("Internet", "obter_texto") => Some((&[S], RetRes(S))),
            ("Internet", "enviar_json") => Some((&[S, S], RetRes(S))),
            ("Internet", "endereco_disponivel") => Some((&[S], RetRes(B))),

            ("Graficos", "iniciar_modo_grafico") => Some((&[B], Ret(hir::Type::Unit))),
            ("Graficos", "encerrar_modo_grafico") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "fechar_janela") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "minimizar_janela") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "restaurar_janela") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "ocultar_borda_janela") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "exibir_borda_janela") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "definir_dimensoes_janela") => Some((&[I, I], Ret(hir::Type::Unit))),
            ("Graficos", "definir_titulo_janela") => Some((&[S], Ret(hir::Type::Unit))),
            ("Graficos", "entrar_modo_tela_cheia") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "sair_modo_tela_cheia") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "definir_icone_janela") => Some((&[I], Ret(hir::Type::Unit))),
            ("Graficos", "largura_janela") => Some((&[], Ret(I))),
            ("Graficos", "altura_janela") => Some((&[], Ret(I))),
            ("Graficos", "largura_tela") => Some((&[], Ret(I))),
            ("Graficos", "altura_tela") => Some((&[], Ret(I))),
            ("Graficos", "limpar") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "renderizar") => Some((&[], Ret(hir::Type::Unit))),
            ("Graficos", "renderizar_imagem") => Some((&[I, I], RetRes(I))),
            ("Graficos", "definir_cor") => Some((&[I], Ret(hir::Type::Unit))),
            ("Graficos", "definir_gradiente") => Some((&[I, I, I], Ret(hir::Type::Unit))),
            ("Graficos", "definir_opacidade") => Some((&[I], Ret(hir::Type::Unit))),
            ("Graficos", "definir_rotacao") => Some((&[I], Ret(hir::Type::Unit))),
            ("Graficos", "criar_cor") => Some((&[I, I, I], RetRes(I))),
            ("Graficos", "obter_rgb") => Some((&[I, I], Ret(I))),
            ("Graficos", "desenhar_retangulo") => Some((&[I, I, I, I, B, B], Ret(hir::Type::Unit))),
            ("Graficos", "desenhar_elipse") => Some((&[I, I, I, I, B], Ret(hir::Type::Unit))),
            ("Graficos", "desenhar_linha") => Some((&[I, I, I, I], Ret(hir::Type::Unit))),
            ("Graficos", "desenhar_ponto") => Some((&[I, I], Ret(hir::Type::Unit))),
            ("Graficos", "desenhar_poligono") => Some((
                &[hir::Type::VectorInteger, hir::Type::VectorInteger, B],
                RetRes(B),
            )),
            ("Graficos", "desenhar_texto") => Some((&[I, I, S], Ret(hir::Type::Unit))),
            ("Graficos", "definir_fonte_texto") => Some((&[S], Ret(hir::Type::Unit))),
            ("Graficos", "definir_tamanho_texto") => Some((&[R], Ret(hir::Type::Unit))),
            ("Graficos", "definir_estilo_texto") => Some((&[B, B, B], Ret(hir::Type::Unit))),
            ("Graficos", "carregar_fonte") => Some((&[S], RetRes(B))),
            ("Graficos", "largura_texto") => Some((&[S], Ret(I))),
            ("Graficos", "altura_texto") => Some((&[S], Ret(I))),
            ("Graficos", "carregar_imagem") => Some((&[S], RetRes(I))),
            ("Graficos", "liberar_imagem") => Some((&[I], Ret(hir::Type::Unit))),
            ("Graficos", "largura_imagem") => Some((&[I], RetRes(I))),
            ("Graficos", "altura_imagem") => Some((&[I], RetRes(I))),
            ("Graficos", "desenhar_imagem") => Some((&[I, I, I], Ret(hir::Type::Unit))),
            ("Graficos", "desenhar_porcao_imagem") => {
                Some((&[I, I, I, I, I, I, I], Ret(hir::Type::Unit)))
            }
            ("Graficos", "transformar_imagem") => Some((&[I, B, B, I, I], RetRes(I))),
            ("Graficos", "transformar_porcao_imagem") => {
                Some((&[I, I, I, I, I, B, B, I, I], RetRes(I)))
            }
            ("Graficos", "redimensionar_imagem") => Some((&[I, I, I, B], RetRes(I))),
            ("Graficos", "obter_cor_pixel") => Some((&[I, I, I], RetRes(I))),
            ("Graficos", "salvar_imagem") => Some((&[I, S], RetRes(B))),
            ("Graficos", "proximo_frame_gif") => Some((&[I], Ret(hir::Type::Unit))),
            ("Graficos", "obter_intervalo_gif") => Some((&[I], RetRes(I))),
            ("Graficos", "obter_numero_quadros_gif") => Some((&[I], RetRes(I))),
            ("Graficos", "obter_numero_quadro_atual_gif") => Some((&[I], RetRes(I))),
            ("Graficos", "obter_quadro_gif") => Some((&[I, I], RetRes(I))),
            ("Graficos", "definir_quadro_gif") => Some((&[I, I], RetRes(B))),

            ("Teclado", "tecla_pressionada") => Some((&[I], Ret(B))),
            ("Teclado", "alguma_tecla_pressionada") => Some((&[], Ret(B))),
            ("Teclado", "ler_tecla") => Some((&[], RetRes(I))),
            ("Teclado", "caracter_tecla") => Some((&[I], Ret(C))),

            ("Mouse", "botao_pressionado") => Some((&[I], Ret(B))),
            ("Mouse", "algum_botao_pressionado") => Some((&[], Ret(B))),
            ("Mouse", "ler_botao") => Some((&[], RetRes(I))),
            ("Mouse", "posicao_x") => Some((&[], Ret(I))),
            ("Mouse", "posicao_y") => Some((&[], Ret(I))),
            ("Mouse", "ocultar_cursor") => Some((&[], Ret(hir::Type::Unit))),
            ("Mouse", "exibir_cursor") => Some((&[], Ret(hir::Type::Unit))),

            ("Sons", "carregar_som") => Some((&[S], RetRes(I))),
            ("Sons", "liberar_som") => Some((&[I], Ret(hir::Type::Unit))),
            ("Sons", "reproduzir_som") => Some((&[I, B], RetRes(I))),
            ("Sons", "pausar_som") => Some((&[I], Ret(hir::Type::Unit))),
            ("Sons", "interromper_som") => Some((&[I], Ret(hir::Type::Unit))),
            ("Sons", "obter_tamanho_musica") => Some((&[I], Ret(I))),
            ("Sons", "obter_posicao_atual_musica") => Some((&[I], Ret(I))),
            ("Sons", "definir_posicao_atual_musica") => Some((&[I, I], RetRes(B))),
            ("Sons", "definir_volume") => Some((&[I], Ret(hir::Type::Unit))),
            ("Sons", "definir_volume_reproducao") => Some((&[I, I], Ret(hir::Type::Unit))),
            ("Sons", "obter_volume") => Some((&[], Ret(I))),
            ("Sons", "obter_volume_reproducao") => Some((&[I], Ret(I))),
            _ => None,
        };

        let Some((param_types, return_spec)) = signature else {
            return self.fail(
                "pors.unknown-stdlib-function",
                format!("Função desconhecida na biblioteca padrão: {module}.{member}"),
                member_span,
            );
        };
        if arguments.len() != param_types.len() {
            return self.fail(
                "pors.stdlib-call-arity",
                format!(
                    "`{module}.{member}` espera {} argumento(s), recebeu {}",
                    param_types.len(),
                    arguments.len()
                ),
                whole_span,
            );
        }
        for (index, (argument, expected_ty)) in arguments.iter().zip(param_types).enumerate() {
            if argument.ty != *expected_ty {
                return self.fail(
                    "pors.stdlib-call-type-mismatch",
                    format!(
                        "O argumento {} de `{module}.{member}` deve ter tipo `{}`, recebeu `{}`",
                        index + 1,
                        type_label(*expected_ty),
                        type_label(argument.ty)
                    ),
                    syntax_span(argument.span),
                );
            }
        }
        let ty = match return_spec {
            Ret(scalar) => scalar,
            RetOpt(inner) => self.intern_builtin_enum(hir::BuiltinEnum::Option(inner), whole_span),
            RetRes(ok) => self.intern_builtin_enum(
                hir::BuiltinEnum::Result {
                    ok,
                    err: hir::Type::String,
                },
                whole_span,
            ),
        };
        Ok(hir::Expr {
            kind: hir::ExprKind::HostCall {
                module: module.into(),
                name: member,
                arguments,
            },
            ty,
            span: to_hir_span(whole_span),
        })
    }
}

//! Utilitário de depuração: imprime os tokens de um arquivo ou string.
//!
//! `cargo run -p portugol-syntax --example tokens -- arquivo.por`

fn main() {
    let path = std::env::args().nth(1).expect("informe um arquivo");
    let bytes = std::fs::read(&path).expect("não foi possível ler o arquivo");
    let source = match String::from_utf8(bytes.clone()) {
        Ok(text) => text,
        Err(_) => bytes.iter().map(|&byte| byte as char).collect(),
    };

    for token in portugol_syntax::lexer::lex(&source).tokens {
        let text = &source[token.span.start as usize..token.span.end as usize];
        println!(
            "{:>5}..{:<5} {:?} {:?}",
            token.span.start, token.span.end, token.kind, text
        );
    }
}

#!/usr/bin/env python3
"""Copia os .por do Portugol Studio original para tests/corpus, em UTF-8.

Uso: python3 scripts/importar_corpus.py <caminho do Portugol-Studio>

Os exemplos originais estão em ISO-8859-1 (alguns em UTF-8). A estrutura de
pastas a partir de ide/src/main/assets é preservada.
"""

import pathlib
import sys


def main() -> None:
    origem = pathlib.Path(sys.argv[1]) / "ide" / "src" / "main" / "assets"
    destino = pathlib.Path(__file__).resolve().parent.parent / "tests" / "corpus"
    total = 0
    for arquivo in sorted(origem.rglob("*.por")):
        dados = arquivo.read_bytes()
        try:
            texto = dados.decode("utf-8")
        except UnicodeDecodeError:
            texto = dados.decode("latin-1")
        texto = texto.lstrip("﻿").replace("\r\n", "\n")
        alvo = destino / arquivo.relative_to(origem)
        alvo.parent.mkdir(parents=True, exist_ok=True)
        alvo.write_text(texto, encoding="utf-8")
        total += 1
    print(f"{total} arquivos copiados para {destino}")


if __name__ == "__main__":
    main()

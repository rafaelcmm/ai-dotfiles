#!/usr/bin/env python3
"""Métricas objetivas de legibilidade e de padrões de IA para texto pt-BR.

Existe para tirar a revisão do "achismo": frase média, variação de ritmo,
Flesch adaptado ao português e ocorrência das expressões da lista vermelha são
verificáveis, então servem de evidência no relatório de revisão.

Uso:
    python3 scripts/analisar.py caminho/do/arquivo.md
    cat texto.txt | python3 scripts/analisar.py -

Aceita .md, .txt e arquivos .ts/.tsx (nesse caso extrai apenas o conteúdo das
strings, que é o que o cliente lê — código não entra na conta).

Limitação assumida: a contagem de sílabas é aproximada por grupos vocálicos, sem
tratar hiato/ditongo caso a caso. O erro é sistemático e pequeno, o que preserva
a comparação entre versões do mesmo texto — que é o uso pretendido. Não use o
número como laudo linguístico.
"""

import re
import sys
from pathlib import Path

# Expressões da lista vermelha de `references/padroes-ia-ptbr.md`. Buscadas em
# minúsculas e sem acento para não depender de como o autor digitou.
LISTA_VERMELHA = [
    "no mundo de hoje", "no mundo atual", "nos dias de hoje", "na era digital",
    "cada vez mais competitivo", "com o avanco da tecnologia",
    "e importante ressaltar", "e importante destacar", "vale ressaltar",
    "vale lembrar", "cabe destacar", "nesse sentido", "por conseguinte",
    "em suma", "em conclusao", "mas nao menos importante",
    "papel crucial", "papel fundamental", "divisor de aguas", "marca um novo",
    "revoluciona a forma", "eleva a outro patamar", "marca indelevel",
    "experiencia unica", "qualidade excepcional", "simplesmente perfeito",
    "imperdivel", "inigualavel", "revolucionario", "impecavel",
    "o futuro e promissor", "esperamos que este", "agora que voce ja sabe",
    "boas compras", "espero ter ajudado", "vamos mergulhar", "vamos explorar",
    "otima pergunta", "segue abaixo", "aqui esta o texto",
    "nao e apenas", "nao e so", "mais do que um", "mais do que uma",
    "no fundo,", "na essencia", "a verdadeira questao",
    "especialistas afirmam", "estudos apontam", "segundo relatorios",
]

SEM_ACENTO = str.maketrans("áàâãéêíóôõúüçÁÀÂÃÉÊÍÓÔÕÚÜÇ", "aaaaeeiooouucAAAAEEIOOOUUC")


def extrair_texto(bruto: str, sufixo: str) -> str:
    """Devolve só a prosa que o leitor final enxerga.

    Em arquivo de código, o que interessa são as strings de conteúdo; medir o
    código junto distorceria toda métrica de frase. O corte em 60 caracteres
    descarta chave, classe CSS, slug, import, rótulo e título — strings curtas
    que não são prosa e que puxariam a média de palavras por frase para baixo.
    """
    if sufixo in {".ts", ".tsx", ".js", ".jsx"}:
        strings = re.findall(r'"((?:[^"\\]|\\.){60,})"|`((?:[^`\\]|\\.){60,})`', bruto)
        return "\n".join(a or b for a, b in strings)
    return bruto


def limpar(texto: str) -> str:
    """Remove marcação que não é lida em voz alta (código, link, HTML, bullets)."""
    texto = re.sub(r"```.*?```", " ", texto, flags=re.S)
    texto = re.sub(r"`[^`]*`", " ", texto)
    texto = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", texto)
    texto = re.sub(r"<[^>]+>", " ", texto)
    texto = re.sub(r"^[#>\-*|]+\s*", "", texto, flags=re.M)
    return texto


def contar_silabas(palavra: str) -> int:
    """Aproxima sílabas por grupos vocálicos contíguos (mínimo de 1)."""
    grupos = re.findall(r"[aeiouáàâãéêíóôõúüy]+", palavra.lower())
    return max(1, len(grupos))


def analisar(texto: str) -> dict:
    # Título, item de lista e rótulo raramente terminam em ponto. Sem tratar a
    # quebra de linha como fim de frase, eles se fundiriam ao parágrafo seguinte
    # e inflariam a média de palavras por frase.
    frases = [
        f.strip()
        for linha in texto.splitlines()
        for f in re.split(r"[.!?…]+(?=\s|$)", linha)
        if f.strip()
    ]
    palavras = re.findall(r"[A-Za-zÀ-ÿ][A-Za-zÀ-ÿ'-]*", texto)
    if not frases or not palavras:
        return {}

    tamanhos = [len(re.findall(r"[A-Za-zÀ-ÿ][A-Za-zÀ-ÿ'-]*", f)) for f in frases]
    tamanhos = [t for t in tamanhos if t] or [0]
    media = sum(tamanhos) / len(tamanhos)
    desvio = (sum((t - media) ** 2 for t in tamanhos) / len(tamanhos)) ** 0.5
    silabas = sum(contar_silabas(p) for p in palavras)

    # Martins et al. (1996), adaptação do Flesch ao português do Brasil.
    flesch = 248.835 - 1.015 * (len(palavras) / len(frases)) - 84.6 * (silabas / len(palavras))

    achatado = texto.lower().translate(SEM_ACENTO)
    vermelhas = [(e, achatado.count(e)) for e in LISTA_VERMELHA if e in achatado]

    return {
        "palavras": len(palavras),
        "frases": len(frases),
        "media": media,
        "desvio": desvio,
        "flesch": flesch,
        "longas": [t for t in tamanhos if t > 30],
        "travessoes": (texto.count("—") + texto.count("–")) / len(palavras) * 1000,
        "negrito": len(re.findall(r"\*\*[^*]+\*\*", texto)),
        "vermelhas": sorted(vermelhas, key=lambda x: -x[1]),
    }


def main() -> int:
    if len(sys.argv) < 2:
        print(__doc__)
        return 2

    alvo = sys.argv[1]
    if alvo == "-":
        bruto, sufixo = sys.stdin.read(), ".txt"
    else:
        caminho = Path(alvo)
        bruto, sufixo = caminho.read_text(encoding="utf-8"), caminho.suffix

    m = analisar(limpar(extrair_texto(bruto, sufixo)))
    if not m:
        print("Nenhum texto analisável encontrado.")
        return 1

    def marca(ok: bool) -> str:
        return "ok  " if ok else "ATEN"

    print(f"\n{alvo}")
    if sufixo in {".ts", ".tsx", ".js", ".jsx"}:
        # Num arquivo de conteúdo, título de seção, célula de tabela e item de
        # lista são strings próprias e entram como frases curtas. Serve para
        # comparar versões do mesmo arquivo, não para julgar a média isolada.
        print("  (arquivo de código: métricas de frase são comparativas, não absolutas)")
    print(f"  {m['palavras']} palavras · {m['frases']} frases")
    print(f"  {marca(14 <= m['media'] <= 20)} frase média .... {m['media']:.1f} palavras (meta 14–20)")
    print(f"  {marca(m['desvio'] >= 5)} variação ........ ±{m['desvio']:.1f} (meta ≥ 5; abaixo = ritmo de máquina)")
    print(f"  {marca(m['flesch'] >= 50)} Flesch-BR ....... {m['flesch']:.0f} (meta ≥ 50)")
    print(f"  {marca(not m['longas'])} frases > 30 pal .. {len(m['longas'])}")
    print(f"  {marca(m['travessoes'] <= 3)} travessões ...... {m['travessoes']:.1f}/1000 palavras (meta ≤ 3)")
    print(f"  {'ok  ' if m['negrito'] <= m['palavras'] / 100 else 'ATEN'} negrito ......... {m['negrito']} trechos")

    if m["vermelhas"]:
        print("\n  Lista vermelha (padroes-ia-ptbr.md §1):")
        for expressao, n in m["vermelhas"]:
            print(f"    {n}x  {expressao}")
    else:
        print("\n  Lista vermelha: nenhuma ocorrência.")
    print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

---
name: revisao-texto-ptbr
description: >-
  Revisor e redator profissional de português brasileiro. SOMENTE para texto em
  pt-BR — NÃO use em inglês, espanhol, português europeu ou qualquer outro
  idioma: a norma, a lista de padrões de IA e o índice de legibilidade são
  calibrados para o português do Brasil e não valem fora dele. Serve a qualquer
  domínio: documentação, página institucional, artigo, guia, página de clínica ou
  escritório, serviço público, e-mail, anúncio, microcopy, meta description,
  descrição de produto. Use quando pedirem para "revisar o texto", "corrigir o
  português", "melhorar a redação", "deixar mais humano", "tirar cara de IA",
  "otimizar para SEO" ou "escrever um texto sobre X". Cobre gramática e
  ortografia, coesão e coerência, adequação ao público, SEO/GEO e remoção de
  padrões de escrita de IA.
---

# Revisão e criação de texto em pt-BR

Dois modos: **revisar** um texto que já existe e **criar** um novo. Os dois
terminam na mesma passada final anti-IA e no mesmo relatório.

O trabalho é de copidesque profissional, não de corretor ortográfico: o texto
sai correto, coerente, no tom do público e legível por gente — sem parecer
gerado por máquina.

## Escopo

Esta skill vale **só para texto em português do Brasil**. A norma culta aplicada,
a lista vermelha de expressões, as marcas de escrita de IA em português e o
índice Flesch-BR são todos calibrados para o pt-BR. Para texto em inglês,
espanhol, português europeu ou qualquer outro idioma, **esta skill não se
aplica**: a lista vermelha não corresponde às marcas daquele idioma e os limites
de Flesch-BR não significam nada fora do português brasileiro.

O domínio, por outro lado, é livre. Nada aqui pressupõe loja, produto ou compra:
funciona igual para documentação de API, página de clínica, texto jurídico,
serviço público, artigo, e-mail, anúncio ou descrição de produto. Onde há
exemplo de um domínio, ele é ilustração — não o público presumido.

## Regra zero — o que nunca fazer

1. **Não inventar fato.** Nenhum número, prazo, preço, data, nome, garantia ou
   fonte entra no texto se não estava na origem ou não foi dado pelo usuário.
   Se a frase depende de um dado que falta, pergunte ou escreva a versão sem
   ele. Encurtar é permitido; completar de cabeça, não.
2. **Não mudar compromisso declarado.** Preço, prazo, frete, estoque, garantia,
   condição de pagamento, política de troca, critério de elegibilidade, dosagem,
   cobertura e aviso regulatório são fatos da organização, muitos deles com
   efeito jurídico. Reescreva a forma, nunca o compromisso. Deixar a frase mais
   clara é o trabalho; fazer o texto prometer mais, menos ou coisa diferente é
   erro grave — e vem antes de qualquer outro no relatório.
3. **Não "melhorar" jargão correto do domínio.** O vocabulário técnico do
   público não é erro de português. `requisição idempotente` em documentação de
   API, `anamnese` e `via subcutânea` numa página de saúde, `trânsito em
   julgado` em texto jurídico, `dotação orçamentária` em serviço público: em
   cada área há termos que só um leigo trocaria. Levante o glossário do projeto
   antes de revisar e trate esses termos como intocáveis. Ver
   `references/publico-e-voz.md`.
4. **Não entregar só o texto final.** Sempre acompanha o que mudou e por quê —
   exceto no modo embutido (ver "Como entregar").

## Níveis de intervenção

Combine o nível antes de começar. Na dúvida, use o padrão e diga que usou.

| Nível                   | O que toca                                                  | Quando                                          |
| ----------------------- | ----------------------------------------------------------- | ----------------------------------------------- |
| **Leve** (revisão)      | Ortografia, pontuação, concordância, regência, digitação    | Texto pronto, aprovado, já publicado            |
| **Padrão** (copidesque) | Tudo do leve + frase, ordem das ideias, coesão, tom, cortes | Padrão de qualquer pedido genérico de "revisar" |
| **Reescrita**           | Estrutura, argumento, título, seções inteiras               | Texto que não cumpre o objetivo dele            |

Se durante o nível leve aparecer um problema estrutural, **não faça a reescrita
por conta própria**: aponte no relatório e pergunte.

## Modo revisar — sete passadas, nesta ordem

A ordem é do macro para o micro por economia: não vale polir a vírgula de uma
frase que vai ser cortada na passada 2.

### 1. Briefing

Antes de ler como revisor, responda: **quem lê, para quê, em que canal, com que
restrição** (tamanho, tom, obrigações legais). Se o usuário não disse, infira do
próprio texto e do material do projeto e **declare a inferência no relatório** —
não pergunte o que dá para deduzir. Só pergunte quando duas leituras plausíveis
levam a textos diferentes.

### 2. Estrutura e coerência

- O texto responde à pergunta que o leitor trouxe, e responde **logo**?
- A ordem das seções é a ordem da dúvida do leitor, ou a ordem em que o autor
  aprendeu o assunto?
- Cada parágrafo tem uma ideia? Alguma seção não presta serviço nenhum?
- Existe contradição interna, ou com outro texto já publicado pela organização?
- Falta o passo que o leitor precisa para agir (link, condição, exceção)?

Critérios em `references/coesao-e-coerencia.md`.

### 3. Coesão e ritmo

- Cada conectivo declara a relação certa (causa, oposição, consequência)?
- Referências (`isso`, `ele`, `o mesmo`) apontam para algo inequívoco?
- Repetição desnecessária de palavra ou de estrutura de frase.
- Variação de tamanho de frase — texto todo com frases do mesmo tamanho lê como
  máquina. Alterne curta e longa de propósito.

### 4. Voz, tom e público

Confira contra `references/publico-e-voz.md`: tratamento (`você`), registro,
nível técnico, o que a organização pode e não pode prometer, e as quatro
dimensões de tom (sério↔divertido, formal↔casual, respeitoso↔irreverente,
entusiasmado↔objetivo). Tom é escolha por canal, não gosto pessoal.

### 5. Gramática, ortografia e pontuação

Norma culta pt-BR, Acordo Ortográfico vigente. Prioridade nos erros que mais
aparecem em texto profissional: crase, concordância verbal e nominal, regência,
vírgula, `onde`/`aonde`, `mas`/`mais`, `há`/`a`, `mal`/`mau`, hífen, colocação
pronominal. Lista operacional com correção e exemplo em
`references/gramatica-ptbr.md`.

Também nesta passada: números, moeda, data, unidade e abreviatura no padrão
brasileiro (`R$ 29,90`, `12/03/2026`, `10 cm`, `3x sem juros`).

### 6. SEO e GEO (só para texto que vai à web)

Intenção de busca, título, primeiro parágrafo, hierarquia de headings, entidade,
link interno, meta description, trecho citável por IA. Regras em
`references/seo-e-geo.md`. Nunca force palavra-chave: se ficar estranho lido em
voz alta, está errado.

### 7. Passada anti-IA (obrigatória, inclusive em texto humano)

Rode a lista de `references/padroes-ia-ptbr.md` contra a **versão final**, não
contra o original. Ela pega inflação de importância, "não é só X, é Y", trios
forçados, `é importante ressaltar`, `no mundo atual`, `divisor de águas`, negrito
decorativo, emoji de enfeite, travessão em excesso, fecho otimista vazio e
sign-off de chatbot.

Um sinal isolado não prova nada — travessão sozinho é estilo. Três ou quatro
juntos no mesmo parágrafo é o padrão a corrigir.

### 8. Verificação objetiva

```bash
python3 scripts/analisar.py <arquivo|->
```

Reporta frase média, variação de tamanho, Flesch-BR, travessões por mil palavras
e ocorrências das expressões vigiadas.

Metas padrão, calibradas para texto web voltado ao público geral: frase média
**14–20 palavras**, Flesch-BR **≥ 50** (fácil/médio), travessão **≤ 3/1000
palavras**, nenhum termo da lista vermelha.

Se o texto é documentação de referência, norma, texto jurídico ou guia técnico
com terminologia obrigatória, recalibre a meta e **diga no relatório qual meta
usou**. Número ruim é sinal para reler, não sentença.

## Modo criar

1. **Briefing** — público, objetivo (o que o leitor faz depois), canal, palavra
   -chave/intenção, tamanho, o que **não** pode ser dito. Máximo de 3 perguntas;
   o resto se infere e se declara.
2. **Esqueleto antes da prosa** — promessa em uma frase, perguntas reais que o
   leitor faz, ordem de resposta, próximo passo do leitor. Confirme o esqueleto
   se o texto for longo.
3. **Rascunho** com exemplo concreto no lugar de adjetivo. Um número real, um
   caso da própria operação, um detalhe verificável valem mais que "excelente
   qualidade".
4. **Rode o modo revisar inteiro contra o próprio rascunho.** Todo texto criado
   aqui passa pelas passadas 2 a 8 antes de ser entregue.

Modelos por tipo de texto (página institucional, artigo ou guia, documentação de
referência, descrição de produto, e-mail, anúncio, microcopy, meta description)
em `references/seo-e-geo.md`.

## Como entregar

**Padrão** — três blocos, nesta ordem:

1. **Diagnóstico** (3 a 6 linhas): para quem é, o que o texto queria fazer, o
   que estava atrapalhando. Sem lista genérica de elogios.
2. **O que mudou** — tabela `Trecho original | Correção | Motivo`, agrupada por
   tipo (fato, estrutura, gramática, tom, SEO, IA). Erro de fato vem primeiro,
   sempre. Se forem mais de ~15 itens repetitivos, agrupe por regra e cite dois
   exemplos.
3. **Versão final**, pronta para colar, sem marcação de revisão.

**Modo arquivo** — quando o texto está num arquivo do repositório: aplique a
edição no arquivo e devolva só diagnóstico + resumo do que mudou. Não mexa em
código, chave de objeto, `href`, `src`, `slug` ou identificador — só na string
que o leitor lê. Slug e URL só mudam com aviso explícito de que é redirect.

**Modo embutido** — quando outra tarefa chama esta skill (commit, PR, doc):
devolva só o texto final.

**Dúvida de fato**: nunca resolva escrevendo mais vago. Marque
`[confirmar: prazo de entrega]` e cite no relatório.

## Adaptação ao projeto

A revisão fica no achismo sem cinco insumos. Levante-os antes da primeira
passada num projeto novo; se algum não existir, diga isso no relatório em vez de
supor.

| Insumo                       | O que procurar                                                                                                 |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------- |
| **Onde vive o texto público**| Arquivos, CMS, templates ou campos que o leitor final enxerga — e o que ali é código e não deve ser tocado      |
| **Definição de público**     | Documento de personas, pesquisa, tickets de suporte, perguntas recorrentes. Sem isso, infira e declare          |
| **Compromissos publicados**  | Preço, prazo, garantia, cobertura, elegibilidade, política de troca, avisos legais — a fonte da verdade da regra zero item 2 |
| **Glossário do domínio**     | Termos técnicos que não são erro de português. Monte a lista no primeiro contato e guarde-a no projeto          |
| **Decisões de SEO/GEO**      | Plano, ADR ou documento de estratégia já aprovado. Siga o que está lá em vez de reabrir a discussão             |

Dois detalhes que aparecem em quase todo projeto web e valem checar: texto `alt`
descreve **o que a imagem prova**, não o objeto genérico; e campo de conteúdo que
alimenta dado estruturado (uma seção de perguntas frequentes que vira `FAQPage`
JSON-LD, por exemplo) precisa de resposta que se sustente sozinha, fora da
página.

## Referências

| Arquivo                           | Use quando                              |
| --------------------------------- | --------------------------------------- |
| `references/padroes-ia-ptbr.md`   | Passada 7; texto "certinho demais"      |
| `references/gramatica-ptbr.md`    | Passada 5; dúvida de norma              |
| `references/coesao-e-coerencia.md`| Passadas 2 e 3; texto que "não flui"    |
| `references/seo-e-geo.md`         | Passada 6; qualquer texto que vai à web |
| `references/publico-e-voz.md`     | Passada 4; definir tom e vocabulário    |

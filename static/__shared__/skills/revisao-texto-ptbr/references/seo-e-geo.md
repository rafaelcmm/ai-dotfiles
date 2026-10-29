# SEO e GEO para texto

Duas audiências de máquina, um texto só: o buscador (SEO) e o motor de resposta
por IA — Google AI Overviews/AI Mode, ChatGPT, Perplexity, Gemini (GEO/AEO).
As duas premiam a mesma coisa: resposta específica, verificável e extraível.

Decisões de SEO/GEO já registradas no projeto (plano, ADR, documento de
estratégia, briefing aprovado) valem mais que estratégia nova. Localize e
consulte antes de propor mudança de rumo.

## Conteúdo

1. Intenção de busca vem antes da palavra-chave
2. Estrutura da página
3. O que faz a IA citar a página
4. E-E-A-T e política de conteúdo do Google
5. Palavra-chave, na prática
6. Modelos por tipo de texto
7. Legibilidade

---

## 1. Intenção de busca vem antes da palavra-chave

Toda consulta carrega uma motivação. Escrever para a intenção errada é o erro de
SEO que nenhuma otimização conserta.

| Intenção      | Consulta típica                            | Página certa                     |
| ------------- | ------------------------------------------ | -------------------------------- |
| Informacional | `como declarar imposto de renda atrasado`  | Artigo ou guia                   |
| Comercial     | `melhor plano de telemedicina`             | Comparativo / hub de categoria   |
| Transacional  | `agendar consulta de dermatologia`         | Página de conversão ou de produto|
| Navegacional  | `<marca> horário de atendimento`           | Página institucional             |

Uma página, uma intenção. Guia que tenta vender no meio perde as duas.

## 2. Estrutura da página

**Título (H1)** — a promessa concreta, com o termo que a pessoa digitaria, na
frente. Sentence case. 50–60 caracteres.
`Imposto de renda atrasado: prazo, multa e como declarar`

**Primeiro parágrafo** — responde a pergunta principal **nas duas primeiras
frases**, sem rampa de aquecimento. É o trecho que o motor de IA lê para decidir
se a página responde. Nada de `Você já se perguntou...`.

**H2/H3** — cada um é uma pergunta ou tarefa real do leitor. Um H2 por dúvida.
Não pule níveis, não use dois H1.

**Parágrafo** — 2 a 4 linhas, autossuficiente. IA extrai bloco, não narrativa.

**Tabela** — comparação de duas dimensões (tipo × característica, plano × preço).
É dos formatos mais citados por motor de IA, e o mais ignorado por quem escreve.

**Meta description** — 140–160 caracteres, com o benefício e o termo principal.
Não é fator de ranking; é anúncio. Escreva para o clique.

**Link interno** — texto âncora descritivo (`guia de erros de preenchimento`),
nunca `clique aqui`. Liga a página ao cluster do assunto.

## 3. O que faz a IA citar a página

Sinais medidos em estudos de GEO e coerentes com o comportamento das engines:

1. **Número e dado concreto** — `a multa é de 1% por mês de atraso, limitada a
   20% do imposto devido` é citável; `quanto antes, melhor` não é.
2. **Definição limpa na primeira frase da seção** — `Carência é o período entre a
contratação do plano e a data em que o procedimento passa a ser coberto.`
3. **Fonte nomeada** quando houver (fabricante, norma, tabela oficial, política
   própria).
4. **Pergunta como heading + resposta imediata abaixo.**
5. **Experiência de primeira mão** — o que a organização viu na própria operação.
   É o primeiro `E` de E-E-A-T e é o que nenhum concorrente copia.
6. **Passagem que se sustenta fora da página**: se o parágrafo for lido isolado
   num resumo de IA, ele ainda faz sentido e ainda identifica quem publicou?

## 4. E-E-A-T e política de conteúdo do Google

Google avalia **quem** escreveu, **como** foi feito (incluindo uso de automação)
e **por que** existe. Conteúdo feito só para ranquear é alvo declarado das
políticas de `scaled content abuse`.

Perguntas de autoavaliação, adaptadas do guia oficial de conteúdo útil:

- A página traz informação, apuração ou análise **originais**?
- Descreve o assunto de forma **completa**, ou repete o que já existe?
- Um especialista no assunto reconheceria erro factual aqui?
- Alguém que buscou isso sai satisfeito, ou volta ao buscador?
- A página existe porque ajuda alguém, ou porque um termo tinha volume de busca?

Se a resposta constrange, o problema é de conteúdo — otimizar não resolve.

## 5. Palavra-chave, na prática

- Termo principal: no H1, no primeiro parágrafo, em pelo menos um H2 e na meta.
- Variações semânticas e termos relacionados espalhados naturalmente (`declaração
  retificadora`, `DARF`, `malha fina`, `Receita Federal`) — entidade, não
  repetição.
- **Nunca force.** Se ficar estranho lido em voz alta, está errado.
- Nada de repetir a palavra-chave para bater densidade. Densidade não é métrica
  de nada desde a década passada.

## 6. Modelos por tipo de texto

### Página institucional / política

Frase direta do compromisso, condição, exceção, como acionar. Prazo sempre em
dias úteis e explícito. Zero adjetivo.

### Artigo ou guia

1. H1 com a dúvida real.
2. Resposta nas duas primeiras frases.
3. Seções por pergunta, na ordem em que a dúvida aparece.
4. Tabela ou lista quando houver comparação ou passo a passo.
5. Bloco de perguntas frequentes: perguntas como a pessoa digitaria; respostas
   **autossuficientes** e em texto puro. Se o campo alimenta `FAQPage` JSON-LD,
   mudar a resposta muda o dado estruturado — então cada resposta precisa se
   sustentar fora da página.
6. Link para a página relacionada no fim, sem virar anúncio.

### Documentação de referência

1. O que a coisa faz, numa frase, antes de qualquer configuração.
2. Contrato explícito: entradas, saídas, erros, efeitos colaterais, limites.
3. Um exemplo mínimo que roda, antes dos casos avançados.
4. Terminologia constante — um nome por conceito em toda a documentação.
5. Frase curta importa mais que Flesch alto: o vocabulário obrigatório derruba o
   índice e isso é esperado.

### Descrição de produto

1. Uma frase que identifica o item com precisão (nome, linha, modelo, estado).
2. O que o comprador ganha — traduzido de especificação para consequência
   (`estrutura em aço` → `aguenta a TV de 55" sem precisar de reforço na parede`).
3. Fato específico daquele item (o que faz esta unidade diferente).
4. Condição objetiva: estado, o que acompanha, como é enviado.
5. Nada de superlativo genérico. Nada de texto de fabricante copiado.

### E-mail

Assunto de até ~45 caracteres com o fato (`Seu pedido saiu para entrega`).
Primeira linha entrega a informação. Uma ação por e-mail.

### Anúncio (rede de busca)

Título com o termo buscado + diferencial verificável. Sem promessa que a página
de destino não cumpra — divergência derruba índice de qualidade e é problema de
conformidade, não de copy.

### Microcopy

Rótulo, botão, estado vazio e mensagem de erro dizem **o que houve e o que fazer
agora**, no vocabulário do leitor, sem humor onde há dinheiro, prazo ou saúde.
Mensagem de erro em três partes: o que houve, por quê (se ajudar), o que fazer
agora.

### Meta description

140–160 caracteres: o que a página entrega + o termo principal + o motivo do
clique. Não repita o H1 palavra por palavra e não prometa nada que a página não
tenha.

## 7. Legibilidade

Índice Flesch adaptado ao português (Martins et al., 1996):

```
Flesch-BR = 248,835 − 1,015 × (palavras/frases) − 84,6 × (sílabas/palavra)
```

| Faixa  | Leitura       | Público               |
| ------ | ------------- | --------------------- |
| 75–100 | Muito fácil   | Fundamental I         |
| 50–75  | Fácil         | Fundamental II        |
| 25–50  | Difícil       | Ensino médio/superior |
| 0–25   | Muito difícil | Acadêmico             |

Meta padrão para texto web voltado ao público geral: **≥ 50**. Documentação
técnica, texto jurídico e guia especializado podem ficar em 40 por causa do
vocabulário obrigatório — desde que a frase seja curta e a estrutura, simples.
Só 12% dos brasileiros de 16 a 64 anos leem textos complexos com proficiência
(INAF), o que faz frase curta e ordem direta serem decisão de alcance, não de
estilo.

Princípios de linguagem simples que se aplicam a qualquer texto publicado: ordem
direta (sujeito–verbo–objeto), voz ativa, uma ideia por frase, palavra comum no
lugar do termo técnico quando existir equivalente, e o que o leitor precisa fazer
dito de forma explícita.

Medição: `python3 scripts/analisar.py`.

# Padrões de escrita de IA em português

Adaptação para pt-BR de [Wikipedia: Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing)
(WikiProject AI Cleanup) e da skill [blader/humanizer](https://github.com/blader/humanizer),
somada às marcas que aparecem especificamente em português.

**Como usar:** um sinal isolado não condena nada. O padrão é o **agrupamento** —
três ou quatro destes no mesmo parágrafo. Corrija reescrevendo a ideia, não
trocando a palavra marcada por um sinônimo.

Os exemplos vêm de domínios diferentes de propósito (documentação, saúde, texto
jurídico, serviço público, produto). O padrão é o mesmo em todos.

## Conteúdo

1. Lista vermelha — expressões que denunciam pt-BR de IA
2. Estruturas de frase
3. Formatação
4. Falso positivo — o que NÃO é sinal de IA
5. O que preservar do texto humano
6. Procedimento de reescrita

---

## 1. Lista vermelha — expressões que denunciam pt-BR de IA

Sempre que aparecer, corte ou reescreva. Quase nenhuma delas carrega informação.

**Aberturas de enchimento**
`No mundo de hoje`, `No mundo atual`, `Nos dias de hoje`, `Em um cenário cada vez mais
competitivo`, `Na era digital`, `Com o avanço da tecnologia`, `Você já se perguntou...?`

**Muletas de transição e ênfase**
`É importante ressaltar que`, `É importante destacar que`, `Vale ressaltar que`, `Vale
lembrar que`, `Cabe destacar`, `Nesse sentido`, `Dessa forma`, `Por conseguinte`,
`Além disso` (quando abre três parágrafos seguidos), `Em suma`, `Em conclusão`,
`Por fim, mas não menos importante`.

**Inflação de importância**
`desempenha um papel crucial`, `é fundamental para`, `verdadeiro divisor de águas`,
`marca um novo capítulo`, `revoluciona a forma como`, `um marco`, `peça-chave`,
`transforma completamente`, `eleva a outro patamar`, `deixa uma marca indelével`.

**Adjetivo de folheto**
`incrível`, `poderoso`, `robusto`, `inovador`, `revolucionário`, `impecável`,
`inigualável`, `surpreendente`, `imperdível`, `simplesmente perfeito`,
`experiência única`, `qualidade excepcional`.

**Fórmulas de fecho**
`Em resumo, podemos concluir que`, `Agora que você já sabe tudo sobre`,
`O futuro é promissor`, `Esperamos que este guia tenha ajudado`,
`Aproveite e boas compras!`.

**Sobra de chatbot**
`Claro!`, `Com certeza!`, `Ótima pergunta!`, `Espero ter ajudado`, `Vamos lá!`,
`Vamos mergulhar`, `Vamos explorar`, `Quer que eu`, `Segue abaixo`,
`Aqui está o texto`, `Até a data do meu treinamento`.

Substituto genérico: **diga o fato**. `É importante ressaltar que o token de
acesso expira em 15 minutos` → `O token de acesso expira em 15 minutos.`

---

## 2. Estruturas de frase

### 2.1 "Não é só X, é Y"

Cria uma objeção que ninguém levantou.

- ❌ `Não é apenas um relatório, é uma ferramenta de decisão.`
- ✅ `O relatório mostra o custo por canal por semana, o que permite cortar o
pior canal antes do fechamento do mês.`

### 2.2 Trio forçado

Três itens porque três soa completo, não porque existem três.

- ❌ `Qualidade, agilidade e confiança em cada atendimento.`
- ✅ `Toda solicitação recebe resposta em até um dia útil, com número de
protocolo no e-mail.`

### 2.3 Falso intervalo "de X a Y"

- ❌ `Do estagiário ao diretor, a plataforma atende todo mundo.`
- ✅ `A plataforma tem dois perfis de acesso: quem lança dados e quem aprova.`

### 2.4 Gerúndio de análise rasa

Cauda `-ando/-endo` que finge profundidade.

- ❌ `O painel permite filtrar por período, reforçando a autonomia do time e
destacando a flexibilidade da ferramenta.`
- ✅ `O painel pode ser filtrado por período.`

### 2.5 Fuga do verbo "ser/ter"

`configura-se como`, `apresenta-se como`, `constitui`, `conta com`, `dispõe de`,
`ostenta`, `possui` no lugar de `é` e `tem`.

- ❌ `O plano conta com um painel que dispõe de exportação em CSV.`
- ✅ `O plano tem painel com exportação em CSV.`

### 2.6 Voz passiva sem agente

- ❌ `A solicitação será analisada e o protocolo será disponibilizado.`
- ✅ `A equipe analisa a solicitação e envia o protocolo por e-mail.`

### 2.7 Falsa profundidade

`No fundo`, `na essência`, `a verdadeira questão é`, `mais do que nunca`,
`o que realmente importa`. Corte a moldura e deixe a afirmação.

### 2.8 Anúncio do próximo tópico

`Vamos entender como funciona`, `A seguir, veja`, `Antes de mais nada`.
Diga a coisa em vez de anunciar que vai dizer.

### 2.9 Frases de efeito em série

Uma frase curta enfatiza. Quatro seguidas parecem trailer.

- ❌ `Chegou. Mudou tudo. Sem volta. O jogo é outro.`

### 2.10 Fonte vaga

`Especialistas afirmam`, `Estudos apontam`, `Segundo relatórios do setor`,
`Muitos usuários relatam`. Ou nomeia a fonte, ou corta a afirmação.

---

## 3. Formatação

| Sinal                                         | Correção                                                                                          |
| --------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| Negrito espalhado em termo qualquer           | Negrito só no que o leitor precisa achar de relance (valor, prazo, nome do erro)                  |
| Lista em que todo item começa com **rótulo:** | Vira prosa, ou lista com frases de verdade                                                        |
| Emoji como marcador de seção                  | Remover; emoji só onde a marca já usa                                                             |
| Travessão (—) demais                          | Vírgula, ponto, dois-pontos ou parênteses. Limite: ~3 por mil palavras                            |
| Título em Caixa Alta de Todas as Palavras     | Em pt-BR, título usa maiúscula só na inicial e em nomes próprios: `Como ativar a verificação em duas etapas` |
| Título seguido de frase que repete o título   | Cortar a frase repetida                                                                           |
| Tudo em bullets                               | Bullet é para item paralelo. Explicação é parágrafo                                               |
| Tabela onde cabia uma frase                   | Tabela só quando há duas dimensões para comparar                                                  |

---

## 4. Falso positivo — o que NÃO é sinal de IA

- Português correto e revisado. Texto limpo não é texto de máquina.
- Um travessão. Um `no entanto`. Uma aspa curva (`"` vem do editor, não do robô).
- Termo técnico obrigatório do domínio (`idempotência`, `anamnese`, `trânsito em
  julgado`, `pré-venda`).
- Uma frase curta isolada para dar ênfase.
- Repetição intencional com efeito de ritmo.
- Aviso legal, escopo, limite de garantia e FAQ — informação obrigatória, não
  enchimento.
- Texto anterior a dezembro de 2022.

---

## 5. O que preservar do texto humano

- Detalhe específico e verificável: `chegou amassado no canto inferior direito`.
- Opinião assumida e dúvida honesta: `acho que compensa, mas o preço me incomoda`.
- Aparte entre parênteses, autocorreção, gíria do domínio na medida certa.
- Frases de tamanhos irregulares.
- Exemplo real da própria operação — o caso que apareceu no suporte, o número que
  saiu do relatório, a dúvida que o cliente ou o paciente trouxe. É o que uma IA
  não inventaria com verdade.

---

## 6. Procedimento de reescrita

1. Marque cada padrão encontrado.
2. Reescreva o **parágrafo**, não a palavra sublinhada. Trocar `crucial` por
   `essencial` não resolve nada.
3. Leia em voz alta. Onde travar, quebre a frase.
4. Pergunte duas coisas antes de entregar:
   - o que ainda soa como máquina aqui?
   - alguma informação, número, nome ou ressalva sumiu ou apareceu do nada?
5. Rode `python3 scripts/analisar.py` e confira travessões e lista vermelha.

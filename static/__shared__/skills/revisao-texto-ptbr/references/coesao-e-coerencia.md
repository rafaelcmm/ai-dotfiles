# Coesão e coerência — critérios de revisão

Base: os sete padrões de textualidade de Beaugrande & Dressler (1981) e os
mecanismos de coesão de Halliday & Hasan, reduzidos ao que se checa na prática.

**Coerência** é o sentido: as ideias formam um todo que faz sentido para quem lê.
**Coesão** é a costura visível: os conectivos, pronomes e repetições que amarram
as frases. Texto pode ser coeso e incoerente (tudo ligado, nada faz sentido) e
coerente com coesão ruim (entende-se, mas trava a leitura).

## Conteúdo

1. Os sete critérios, como checklist
2. Coerência — o que quebra
3. Coesão — os cinco mecanismos e o erro de cada um
4. Parágrafo
5. Ritmo
6. Diagnóstico rápido de "não flui"

---

## 1. Os sete critérios, como checklist

| Critério              | Pergunta de revisão                                                      | Falha típica                                      |
| --------------------- | ------------------------------------------------------------------------ | ------------------------------------------------- |
| **Coesão**            | As frases estão amarradas por conectivo, pronome e repetição controlada? | `Isso` sem referente claro                        |
| **Coerência**         | As ideias formam um todo sem contradição nem salto?                      | Parágrafo que responde outra pergunta             |
| **Intencionalidade**  | O texto ainda persegue o objetivo de quem escreveu?                      | Guia que vira anúncio no meio                     |
| **Aceitabilidade**    | O leitor aceita esse texto como útil e relevante?                        | Enchimento antes da resposta                      |
| **Informatividade**   | Traz informação nova na medida certa?                                    | Óbvio demais (tédio) ou denso demais (abandono)   |
| **Situacionalidade**  | Serve ao canal e ao momento (documentação, guia, e-mail, aviso de erro)? | Tom de artigo acadêmico numa tela de erro         |
| **Intertextualidade** | Dialoga certo com o que o leitor já conhece e com os outros textos?      | Contradizer o prazo que outra página publica      |

---

## 2. Coerência — o que quebra

**Salto lógico.** A conclusão não decorre do que veio antes.
❌ `O endpoint aceita uma chave de idempotência. Por isso, repetir a chamada é
seguro.` (falta o elo: o que a chave faz quando chega duas vezes)
✅ `O endpoint aceita uma chave de idempotência. Quando a mesma chave chega duas
vezes, o servidor devolve a resposta do primeiro pedido em vez de criar um
segundo registro — então repetir a chamada é seguro.`

**Contradição interna.** `A consulta pode ser remarcada até 24h antes` num
parágrafo e `remarcações exigem 2 dias úteis de antecedência` no outro. Numa
revisão, isso é erro de fato: aponte antes de tudo.

**Progressão parada.** Três parágrafos dizendo a mesma coisa com palavras
diferentes. Corte dois.

**Pergunta não respondida.** O título promete "como identificar", o texto explica
"o que é". Ou o texto muda, ou o título muda.

**Ordem errada.** A informação que decide a ação do leitor aparece depois da
história da organização. Ordem correta: resposta → condição/exceção → detalhe →
ação.

**Pressuposto não declarado.** O texto assume que o leitor sabe o que é
`carência`, `webhook` ou `trânsito em julgado`. Ou define na primeira ocorrência,
ou linka.

---

## 3. Coesão — os cinco mecanismos e o erro de cada um

**1. Referência (pronome, artigo, demonstrativo)**
Erro: referente ambíguo.
❌ `O relatório é gerado pelo worker e salvo no bucket. Ele expira em 7 dias.`
✅ `...e salvo no bucket. O link de download expira em 7 dias; o arquivo no
bucket, em 90.`
Regra prática: se `isso`/`ele`/`o mesmo` puder apontar para dois substantivos
anteriores, repita o substantivo. Repetição clara vence elegância ambígua.
`O mesmo` como pronome é burocratês: troque pelo nome da coisa.

**2. Substituição e elipse**
Omitir o que já está claro evita repetição chata — desde que a recuperação seja
imediata. `Solicitações enviadas até 14h são analisadas no mesmo dia; depois
disso, no seguinte.`

**3. Conectivo (o que mais erra)**
O conectivo declara uma relação lógica. Se declarar a errada, o leitor tropeça.

| Relação      | Use                               | Não use como sinônimo                        |
| ------------ | --------------------------------- | -------------------------------------------- |
| Adição       | e, também, além de                | `além disso` em todo parágrafo               |
| Oposição     | mas, porém, no entanto, embora    | `entretanto` decorativo                      |
| Causa        | porque, já que, uma vez que, como | `visto que` em texto para o público geral    |
| Consequência | por isso, então, de modo que      | `por conseguinte`, `logo` (formal demais)    |
| Condição     | se, caso, desde que               | —                                            |
| Conclusão    | portanto, ou seja                 | `em suma` repetido                           |
| Tempo        | quando, depois que, assim que     | —                                            |

Teste: apague o conectivo. Se a relação continuar óbvia, ele era enfeite —
deixe apagado.

**4. Repetição lexical e sinônimo**
Erro dos dois lados: repetir a mesma palavra cinco vezes cansa; trocar de nome a
cada frase (`a solicitação` → `o pedido` → `a ficha` → `o requerimento`) faz o
leitor achar que são coisas diferentes. **Um nome por conceito no mesmo texto.**
Repetir o nome certo é melhor que caçar sinônimo.

**5. Paralelismo**
Itens de lista com a mesma estrutura sintática.
❌ `Preencher o cadastro, se os dados estão certos, verificação do anexo.`
✅ `Preencher o cadastro, conferir os dados, anexar o documento.`

---

## 4. Parágrafo

- Uma ideia por parágrafo. Duas ideias, dois parágrafos.
- Web: 2 a 4 linhas. Bloco de 8 linhas afasta leitor no celular e é ignorado por
  mecanismo de extração de resposta.
- A primeira frase carrega a ideia (`topic sentence`). O resto sustenta.
- Cada parágrafo deve poder ser lido sozinho e ainda fazer sentido — é assim que
  IA e trecho destacado de busca leem a página.

## 5. Ritmo

- Alterne frases curtas e longas de propósito. Texto com todas as frases entre
  18 e 22 palavras lê como máquina.
- Frase média de 14 a 20 palavras para texto web voltado ao público geral. Acima
  de 30, quebre.
- Não comece três frases seguidas com a mesma palavra ou a mesma estrutura.
- Leia em voz alta. Onde faltar ar, ponha ponto.

## 6. Diagnóstico rápido de "não flui"

1. Leia só a primeira frase de cada parágrafo. Elas contam a história inteira? Se
   não, o problema é de estrutura (coerência), não de conectivo.
2. Marque todo `isso`, `ele`, `o mesmo`, `tal`. Cada um tem referente único?
3. Apague todos os conectivos e releia. Recoloque só os que fizerem falta.
4. Conte as palavras de cada frase. Sequência sem variação = ritmo de máquina.
5. Pergunte de cada parágrafo: **o que o leitor sabe depois dele que não sabia
   antes?** Sem resposta, corte.

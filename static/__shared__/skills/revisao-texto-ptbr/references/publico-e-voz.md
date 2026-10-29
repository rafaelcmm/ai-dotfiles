# Público, voz e tom

Voz é constante (quem a organização é). Tom varia por canal e por momento do
leitor. Quem escreve escolhe o tom; quem revisa verifica se o tom escolhido é o
certo para aquele canal.

Este arquivo é um **modelo**: as seções 1 e 2 são para você preencher com o
projeto em mãos. Os quadros marcados como exemplo são ilustração de método, não
a definição de nenhum projeto real.

## Conteúdo

1. Definir o público (modelo)
2. Definir a voz (modelo)
3. Tom por canal
4. Limites — o que o texto nunca faz
5. Microcopy
6. Acessibilidade do texto
7. Antes de entregar, confira

---

## 1. Definir o público (modelo)

Sem público definido, "melhorar o texto" não tem critério. Levante isto antes da
primeira revisão e guarde no projeto. Se o material não existir, infira dos
textos já publicados e dos tickets de suporte, e **declare a inferência no
relatório**.

Perguntas a responder, por perfil de leitor:

1. **Quem é?** Em uma frase, sem persona de marketing.
2. **O que quer desta página?** A tarefa concreta, não o sentimento.
3. **O que trava essa tarefa?** A dúvida que faz a pessoa desistir ou abrir
   ticket.
4. **Que vocabulário usa?** O que ela digita na busca, não o nome interno.
5. **Que termo precisa ser explicado na primeira ocorrência?**
6. **Quanto ela sabe do assunto?** Leigo, usuário recorrente, especialista.

Formato de saída sugerido — **exemplo fictício**, para mostrar o grão de detalhe
esperado (aqui, um serviço de agendamento de exames laboratoriais):

| Perfil                   | O que quer                                 | O que trava                                  | Vocabulário                        |
| ------------------------ | ------------------------------------------ | -------------------------------------------- | ---------------------------------- |
| **Paciente de rotina**   | Marcar o exame pedido e saber o preparo    | Não saber se precisa jejum nem quanto tempo  | Comum; `jejum`, `coleta`           |
| **Paciente com convênio**| Confirmar se o exame é coberto             | Não achar a regra de cobertura nem a carência| `cobertura`, `carência`, `guia`    |
| **Acompanhante/cuidador**| Resolver pela outra pessoa, rápido         | Precisar de dado que só o paciente tem       | Zero jargão                        |
| **Médico solicitante**   | Conferir metodologia e prazo do laudo      | Não achar a metodologia usada                | Nome técnico do exame, sem tradução|

O mesmo formato serve a outros domínios. Em documentação de produto, por
exemplo, os perfis costumam ser "quem está integrando pela primeira vez", "quem
mantém a integração em produção" e "quem decide a compra" — e as três colunas
mudam inteiras.

Regra de ouro, válida em qualquer domínio: **o texto precisa servir ao leigo sem
irritar o especialista.** Jargão do domínio fica, mas explicado na primeira
ocorrência quando a página também recebe iniciante.

Feche a seção identificando **as duas dúvidas que mais travam a ação** em quase
todo perfil. Boa redação responde as duas antes de serem perguntadas, e elas são
diferentes em cada projeto: prazo e preço em muitos, compatibilidade e limite de
uso em outros, cobertura e documentação necessária em outros.

## 2. Definir a voz (modelo)

Voz é o que não muda de canal para canal. Extraia-a do que já existe — página
institucional, documentação publicada, e-mails que a equipe manda — em vez de
inventar. Três a cinco traços, cada um com **a consequência prática para a
frase** e um par `assim ✅ / não assim ❌`. Traço sem consequência é adjetivo
decorativo e não orienta revisão nenhuma.

Decida também, explicitamente:

- **Tratamento.** `você`, `vocês`, `o senhor`, ou nome. Escolha um e mantenha.
- **Como a organização se chama.** `nós`/`a gente` em texto de marca, o nome
  próprio ou o cargo em texto de política, onde a formalidade protege. Evite
  `o cliente deve` e `o usuário deve` — vira manual de instruções.
- **O que a voz proíbe.** Todo traço tem um avesso; nomeie-o.

**Exemplo fictício** de dois traços no formato esperado:

- **Direta e conferida.** Diz o número real, o prazo real e de onde o dado vem.
  Se o diferencial declarado é cuidado, texto que exagera contradiz a própria
  voz. ✅ `O laudo fica pronto em até 3 dias úteis.` ❌ `Resultados com
  agilidade incomparável.`
- **Acolhedora, não bajuladora.** Recebe a dúvida sem tratar o leitor como
  ignorante e sem puxar saco. ✅ `Se o pedido não listar o preparo, pode vir
  sem jejum.` ❌ `Que ótima pergunta! Fique tranquilo, estamos aqui por você!`

Afeto, quando a marca tem afeto, está **na escolha do que se diz, não na
sintaxe**: não vira diminutivo em tudo, não vira interjeição, não vira emoji no
meio da frase.

## 3. Tom por canal

Quatro dimensões (Nielsen Norman Group): sério↔divertido, formal↔casual,
respeitoso↔irreverente, entusiasmado↔objetivo.

| Canal                             | Posição                                               | Na prática                                      |
| --------------------------------- | ----------------------------------------------------- | ----------------------------------------------- |
| **Artigo / guia**                 | Sério, casual, respeitoso, objetivo                   | Professor paciente. Sem venda no meio           |
| **Documentação de referência**    | Sério, neutro, respeitoso, objetivo                   | Contrato antes de prosa. Terminologia constante |
| **Página institucional**          | Sério, casual, respeitoso, objetivo                   | Fato antes de emoção                            |
| **Descrição de produto**          | Sério, casual, respeitoso, objetivo                   | Fato antes de emoção. O produto é o herói       |
| **Home / banner / campanha**      | Divertido, casual, um tanto irreverente, entusiasmado | Frase curta, energia, zero promessa vaga        |
| **Formulário / fluxo de decisão** | Sério, casual, respeitoso, objetivo                   | Nada de humor onde há dinheiro, prazo ou saúde  |
| **Erro / indisponibilidade**      | Sério, casual, respeitoso, objetivo                   | Diga o que houve e o que fazer agora            |
| **Política / jurídico**           | Sério, formal, respeitoso, objetivo                   | Frase curta, sem adjetivo, sem simpatia forçada |
| **E-mail transacional**           | Sério, casual, respeitoso, objetivo                   | Informação na primeira linha                    |
| **Redes sociais / comunidade**    | Divertido, casual, irreverente, entusiasmado          | Gíria do público cabe aqui                      |

Erro comum: usar o tom de banner na página de política, ou o tom de política na
home. O canal manda.

**Onde o tom de marca entra, e onde para.** Personalidade é de marca, então vale
nas linhas de marca: home, banners, categorias, newsletter, página institucional,
estados vazios. **Não** vale em formulário, pagamento, prazo, erro, política,
documentação e e-mail transacional — ali a pessoa está decidindo dinheiro,
cuidando de saúde ou resolvendo um problema, e simpatia atrapalha a informação.
Artigo e guia são caso à parte: a abertura pode ser calorosa, o miolo continua
professor paciente.

Dito de outro jeito: a marca pode ser simpática; o prazo não é.

## 4. Limites — o que o texto nunca faz

- **Não promete prazo, preço, cobertura, garantia ou condição** que não venha da
  fonte publicada (política, contrato, tabela oficial, dado cadastral do item).
  Regra com faixas, regiões ou exceções se reproduz exatamente; nunca se
  simplifica para o caso melhor.
- **Não afirma certificação, licenciamento, autoria ou autenticidade** sem
  documento que sustente. Quando a relação é de inspiração ou compatibilidade,
  escreva `inspirado em` e `compatível com`, nunca `oficial` ou `original`.
- **Não extrapola em domínio regulado.** Saúde, finanças, direito, educação e
  serviço público têm limites do que pode ser afirmado e do que exige ressalva.
  Na dúvida, mantenha a formulação da fonte e marque `[confirmar]`.
- **Não usa escassez falsa** (`últimas unidades`, `só hoje`) sem que seja verdade.
  É problema de direito do consumidor, não de copy.
- **Não fala mal de concorrente**, nem por comparação implícita.
- **Não infantiliza nem estereotipa** o público que descreve.
- **Não esconde a natureza do que oferece.** Limitação conhecida, variação entre
  lotes, dependência externa, requisito de infraestrutura: isso se descreve como
  é. Virar "característica exclusiva" é adjetivar defeito.

## 5. Microcopy

A tabela abaixo é a fronteira do que a seção 3 descreve: toda linha aqui é
funcional, e por isso **nenhuma delas recebe tom de marca**.

| Situação             | Escreva                                                        | Não escreva                      |
| -------------------- | -------------------------------------------------------------- | -------------------------------- |
| Indisponível         | `Fora de estoque` · `Agenda sem horário até 12/04`             | `Esgotado! 😢`                   |
| Prazo de produção    | `Produzido sob encomenda · até 3 dias úteis`                   | `Feito especialmente para você!` |
| Erro de formulário   | `Digite um CEP com 8 números`                                  | `Ops! Algo deu errado`           |
| Prazo total          | `Até 3 dias úteis de análise, mais o prazo do banco`           | `Resolvemos rapidinho`           |
| Ação principal       | `Adicionar ao carrinho` · `Enviar solicitação`                 | `Quero o meu!`                   |
| Medida ou limite     | `18 cm de largura × 13 cm de altura` · `até 500 chamadas/min`  | `Tamanho ideal para você`        |
| Requisito de preparo | `Jejum de 8 horas` · `Traga documento com foto`                | `Venha preparado!`               |

Mensagem de erro segue três partes: **o que houve → por quê (se ajudar) → o que
fazer agora.** Sem culpar o leitor e sem `Ops`.

## 6. Acessibilidade do texto

- `alt` descreve **o que a imagem prova**, não o objeto genérico: `gráfico de
  barras com a taxa de erro caindo de 4% para 0,3% após o deploy`, não `gráfico`.
  Onde a imagem carrega a informação que decide a ação do leitor, o `alt` é onde
  escala, estado e contexto aparecem para quem não vê.
- Link com texto que faz sentido fora de contexto — nunca `clique aqui`.
- Não use só cor ou só emoji para carregar informação.
- Número por extenso quando o valor for crítico e curto (`dois dias úteis`),
  algarismo em ficha técnica, tabela e preço.
- Frase curta e ordem direta são decisão de alcance: a maior parte do público
  brasileiro lê no celular, com pressa, e boa parte não tem fluência para texto
  complexo.

## 7. Antes de entregar, confira

1. Um leigo entende? Um especialista respeita?
2. O tom bate com o canal, ou é o tom que eu gosto de escrever?
3. Toda promessa tem fonte na política publicada ou no dado do projeto?
4. O texto trata o leitor como adulto?
5. Sobrou adjetivo que não pode ser conferido? Corte.

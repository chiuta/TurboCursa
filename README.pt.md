[English](README.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · [Español](README.es.md) · **Português** · [Deutsch](README.de.md) · [Magyar](README.hu.md)

# **TURBOCURSA**

Versão: 1.5 · Última atualização: 12 de novembro de 2026

## O que é

TURBOCURSA é um jogo de corridas arcade pseudo-3D escrito em Rust em modo clean-room: motor, gráficos, música e nome originais, sem código, recursos ou marcas de qualquer jogo existente. Tudo o que vê e ouve é gerado a partir de uma semente: árvores, rochas, nuvens, nebulosas, auroras e faixas de Mandelbrot nos pontos de controlo, além de banda sonora e som de motor procedurais. A biblioteca base não tem dependências em tempo de execução.

Dezanove paisagens geradas: floresta, deserto, entardecer, colossais árvores-mundo com as copas nas nuvens, mesas vermelhas estratificadas, ilhas flutuantes com cascatas e um planeta com anéis, um lago espelho sob duas luas, uma tundra com auroras e neve, um vulcão com veios de lava, penacho de fumo fractal e raios recursivos, um abismo iluminado com corais e algas, um cosmos com nebulosa de Julia e estrada arco-íris, uma cidadela néon de pirâmides de Sierpinski e blocos de Menger, e uma selva de micélio luminosa. Cada etapa encadeia uma paisagem clássica e três mais recentes, numa ordem dada pela semente. Mais seis mundos, possíveis e impossíveis: um mundo invertido com uma segunda cordilheira pendurada no céu, um deserto-prisma sob faixas de auroras espectrais, um mundo de relojoaria com engrenagens gigantes, um mar de nuvens, um buraco negro com um anel de acreção deformado pela lente gravitacional e um desfiladeiro fractal de estratos listrados.

## Utilização

- Nativo: `cargo run --release --features native --bin turbo` (acrescente `--versus` para 2 jogadores, `--daily` para a etapa do dia, `--seed N` para uma etapa fixa, `--lang xx` para o idioma).
- Web: `./build_web.sh` gera um único ficheiro offline `dist/turbocursa.html` (controlos táteis, seletor de idioma, melhor pontuação). Parâmetros: `?versus`, `?daily`, `?seed=N`, `?lang=xx`.
- Controlos: setas ou WASD, Espaço = turbo, Enter = repetir, M = sem som. Jogador 2: WASD + Shift esquerdo.
- Interface e documentação: English, Română, Français, Italiano, Español, Português, Deutsch, Magyar. O texto do jogo usa uma fonte de píxeis; os acentos são desenhados como uma marca sobre a letra.
- Modos: `--mode campaign|timetrial|endless|rush|collector|slalom|mirror|blackout|zen`, `--cpu` (corrida contra um rival do computador), `--versus` (2 jogadores); web: `?mode=NOME`. · `--career` (divisões, campeonatos e taças, com gravação)

## Modos de jogo

- **Campanha** — Vença o relógio por quatro paisagens geradas; os pontos de controlo somam tempo.
- **Contra O Relógio** — Sem trânsito, o relógio conta para cima: termine o mais depressa possível.
- **Infinito** — Voltas sem fim: cada volta é uma etapa gerada nova, +25 s.
- **Hora De Ponta** — Trânsito triplo; cada ultrapassagem dá 0,8 segundos.
- **Caça Às Esferas** — Siga rastros fractais de esferas; cada uma soma tempo e combo.
- **Slalom De Anéis** — Atravesse os anéis: cada passagem sobe o multiplicador, uma falha custa um segundo.
- **Inversão** — A direção inverte-se seis segundos em cada catorze.
- **Escuridão** — Escuridão: só se vê o cone dos faróis, a paisagem é um boato.
- **Zen** — Sem relógio, trânsito ou choques: apenas conduz por paisagens infinitas.
- **Contra CPU** — Corra contra um rival controlado pelo computador na mesma etapa, ecrã dividido.
- **2 Jogadores** — Dois humanos, uma etapa, ecrã dividido: ganha quem chegar primeiro.

## Carreira

Inicie com `--career` (web: `?mode=career`). O progresso é guardado automaticamente (ficheiro `~/.turbocursa_career` em nativo, `localStorage` no navegador).
- **Divisões** — Bronze, Prata, Ouro, Platina, Diamante. Cada época é um campeonato de seis jornadas com 12 pilotos: os três primeiros sobem, os três últimos descem; o campeão do Diamante recebe um convite para a Taça do Mundo.
- **Taças com rondas eliminatórias** — O seu tempo é comparado com rivais simulados, mais fortes quanto mais alta é a taça e a divisão; os mais lentos são eliminados em cada ronda (16 → 8 → 4 → final na taça nacional; até 32 → 24 → 16 → 8 → 4 → final na Taça do Mundo).
- **Qualificações múltiplas** — Duas hipóteses de *repescagem* por taça: se for eliminado, os melhores eliminados correm de novo e os dois primeiros regressam.
- **A escada** — Chegue à final da **taça nacional** para se qualificar para a **taça continental**; a sua final abre a **taça intercontinental**; a sua final, a **Taça do Mundo**.
- **Todos os países** — 196 países em seis continentes; também pode disputar a taça nacional de qualquer outro país (taças no estrangeiro) e ganhar um troféu por cada um na vitrina.
- **Classificações** — um ranking mundial de 300 pilotos mais você, o ranking de nações, a tabela do campeonato e as suas estatísticas.

## Carros por país

Cada país tem o seu carro de corrida: uma pintura com as cores nacionais (aproximadas, arte original) com matrícula, e um perfil de condução (velocidade máxima, aceleração, aderência, 2-4 cargas turbo). O perfil é um orçamento de soma zero: nenhum país tem um carro melhor em tudo. Escolhe-se com `--car RO` (ou `--car random`), `?car=RO` na web, ou folheiam-se na Garagem da carreira; na carreira usa-se automaticamente o carro do seu país.

## Dificuldade, veículos, obstáculos e bónus

- **5 níveis de dificuldade** — Fácil, Normal, Difícil, Especialista, Louco: alteram o tempo inicial e as bonificações dos pontos de controlo (×1,25 … ×0,68), a quantidade e velocidade do trânsito, quantos obstáculos e bónus há na estrada e o quanto custa uma colisão. Na carreira, o nível também torna os rivais simulados mais fortes ou mais fracos.
- **19 classes de veículo** para o teu carro: esportivo, sedã, microcarro, caminhão, carreta (TIR), ciclomotor, motocicleta, triciclo, tuk-tuk, quadriciclo, buggy, monolugar, carro de rali, picape, furgão, ônibus, trator, limusine e monster truck. Cada uma mantém a pintura e o perfil do país e acrescenta o seu compromisso: velocidade máxima, aceleração, aderência, largura, resistência a colisões e relógio. As duas rodas são estreitas, ágeis e frágeis; ônibus, trator e monster truck são largos e resistentes; o monolugar é o mais rápido e o mais frágil.
- **Trânsito variado**: carros, microcarros, caminhões, carretas, ônibus, tratores, motos e triciclos; quanto maior o nível, mais veículos pesados.
- **Na estrada**: obstáculos (cones, manchas de óleo, barreiras, buracos, poças, lama) e bónus (+4 s, turbo extra, impulso de velocidade, escudo, moedas, trânsito parado, pontuação ×2). Os efeitos ativos aparecem como barras coloridas sob o relógio.
- **Ecrã de seleção** no início de cada jogo: ← → veículo, ↑ ↓ dificuldade, Enter para começar, Backspace para voltar a ele. `--class` / `--level` / `--nopick` (nativo) ou `?class=…&level=…` / os dois seletores (web) saltam-no; na carreira usa as linhas *Dificuldade* / *Veículo* do centro.

## Mundo, sistemas e ferramentas

- **Clima e pisos** — 11 climas (limpo, chuva, tempestade, nevoeiro, neve, tempestade de neve, tempestade de areia, granizo, gelo negro, onda de calor, chuva de cinzas) e 6 pisos (asfalto, lago congelado, salar, campo de lava, dunas, pântano) mudam a aderência, a visibilidade e o aspeto; deixe-os em Auto ou escolha-os (`--weather`, `--surface`, `?weather=`, `?surface=`).
- **Dia e noite** — amanhecer, dia, entardecer ou noite (`--time`, `?time=`), com faróis, estrelas e grilos.
- **Vida à beira da estrada** — pontes, túneis, viadutos e passagens de montanha; passagens de nível com comboios; animais; rampas e placas de turbo; polícia que persegue os infratores (ser apanhado significa desqualificação).
- **Combustível e danos** — o combustível gasta-se e as estações reabastecem; as colisões danificam a carroçaria e o motor (menos velocidade máxima, aceleração mais fraca, direção mais mole) e as estações reparam-nos.
- **Caixa, pneus e afinação** — caixa automática ou manual (E / Q mudam, T alterna); pneus de estrada, chuva, neve e todo-o-terreno e quatro afinações (equilibrada, sprint, apoio aerodinâmico, economia): `--tyre`, `--setup`.
- **Nível 0–999 e jogo responsável** — o nível sobe com a experiência e ajusta a dificuldade (`--rank N` para o forçar); contam-se os dias seguidos, um lembrete sugere uma pausa após uma sessão longa e um aviso diz quando é suficiente por hoje.
- **Insígnias e coleções** — 25 insígnias e uma coleção de classes, climas, pisos, momentos do dia e países conduzidos.
- **Desafio diário** — `--daily` / `?daily`: a mesma etapa, o mesmo veículo e as mesmas condições para todos, com um código curto de resultado para comparar.
- **Fantasma** — a sua melhor corrida numa etapa é repetida como um fantasma ciano com a diferença de tempo (guardado em `~/.turbocursa_ghost_*` ou `localStorage`; mais útil com semente fixa ou na etapa do dia).
- **Rali** — `--rally` / `?mode=rally`: cinco etapas do amanhecer à noite com clima e piso variáveis; danos, combustível e desgaste dos pneus transitam; os minutos no parque de assistência somam ao tempo; a classificação final é calculada contra 15 equipas simuladas.
- **Modo foto** — P câmara livre (as setas movem-na), H mostra ou esconde o HUD, C guarda um PNG (`turbocursa_photo_N.png`; no navegador é descarregado).
- **Controlos** — teclado, botões táteis, comando no navegador (sticks ou cruzeta viram, RT/A acelerar, LT/B travar, X turbo, Start enter, Select voltar, LB/RB mudanças, Y manual) e direção por inclinação no telemóvel (botão ⟲).
- **Acessibilidade** — filtros para daltonismo (`--colorblind protan|deutan|tritan|contrast`), ajudas que seguram a curva, recentram o carro e suavizam as paredes (`--assist 1..3`), modo lento (`--slow 70` = 70 % da velocidade máxima) e interface grande (`--large`).
- **Som** — motor procedural com mudanças, vento, chuva, tempestade, granizo, sirenes, buzina de comboio, gelo, apito de combustível baixo, motor danificado e sons de animais.
- **Criador de etapas e códigos** — `--design` (web: `?mode=design`): escolha a semente, o modo, o veículo, o nível, o clima, o piso, a hora, as quatro paisagens, quão sinuosa e acidentada é a estrada e o seu comprimento; um gráfico mostra o perfil. Cada etapa tem um código curto verificado `TCS1-…`: conduza-o com `--code CÓDIGO` ou `?code=CÓDIGO`; na versão web prima L para o mostrar e copiar uma ligação.
- **Cidades e regras de trânsito** — cada etapa atravessa cidades com avenidas e ruas: fachadas, passeios, passadeiras, semáforos (verde, amarelo, vermelho), peões e limite de velocidade. Passar no vermelho, exceder o limite ou não dar passagem a um peão chama a polícia (em Easy uma multa custa tempo); atropelar um peão custa tempo e danos.
- **Conduzir com responsabilidade compensa** — parar no vermelho, dar passagem, respeitar o limite e atravessar uma cidade sem infrações dá pontos, tempo e um multiplicador até x3 («BOM MOTORISTA», «CIDADE LIMPA»); qualquer infração reinicia a série. Deixar uma ambulância ou um carro de bombeiros ultrapassar com a sirene ligada também compensa; bloqueá-lo é uma infração.
- **Engarrafamentos, desvios e RadarMap** — algumas etapas têm um engarrafamento: uma fila parada com um corredor sinuoso por onde se pode avançar devagar (bater nela é um acidente). Uma faixa de saída à direita antes do engarrafamento leva a uma rua livre; o RadarMap no HUD (mapa com o rumo para cima) mostra a estrada à frente, as cidades, o engarrafamento a vermelho, o desvio a verde, os semáforos e o tempo de travessia pela via principal face à saída.
- **Veículos de emergência e helicópteros** — ambulâncias e carros de bombeiros ultrapassam com as luzes a piscar; uma cena de acidente (um helicóptero que aterra na estrada à medida que te aproximas, uma ambulância e um carro de bombeiros) bloqueia a faixa da direita e deixa livre a da esquerda; um helicóptero de monitorização do trânsito paira sobre cidades e engarrafamentos.
- **Missões, campo de provas, estacionamentos** — recolhe uma encomenda ou um passageiro e deixa-o na baia antes de o tempo acabar; o percurso tem um campo de provas (rampa, argolas de slalom, salto) e estacionamentos anunciados por pórticos, cada um com bónus.
- **Veículos de serviço desbloqueáveis** — carro da polícia (10 cidades limpas), ambulância (8 entregas), camião de bombeiros (3 desafios diários e 4 bónus de campo de provas ou estacionamento), helicóptero (os outros três mais 7 desafios). Podem violar as regras, o trânsito afasta-se, e o helicóptero sobrevoa o trânsito e aterra a baixa velocidade.
- **Eventos e extras** — campeonatos de 7, 10 ou 15 etapas e volta de oito etapas por um país (`--champ`, `--tour`, código `TCC1-…`); desafio semanal com um modificador rotativo (`--weekly`); importar e exportar fantasmas (`--ghost`, `--ghost-export`); pelotão visível de 8 a 16 rivais com grelha, grelha invertida e volta de qualificação (`--pack`, `--inverted`, `--qualify`); ecrã dividido para 3–4 jogadores com preenchimento de CPU (`--players`); bifurcações (rápida à esquerda, panorâmica à direita) com o helicóptero a indicar o caminho; perseguição com o carro da polícia; cadeia de ultrapassagens de 200 a 2000; garagem onde as moedas compram melhorias (`--garage`, `--buy`); estilos de música, mph, esquemas de teclas e códigos secretos de pintura (`--music`, `--mph`, `--keys`, `--secret`); sirenes para o carro da polícia, a ambulância e o camião dos bombeiros. Tudo se ajusta na página **EXTRAS** do ecrã inicial (Backspace no ecrã do veículo). Um campo de código (⌨ no navegador) aceita um campeonato, uma etapa, um fantasma, uma palavra de pintura, DAILY ou WEEKLY.

## Limites

- A versão WebAssembly e a saída de áudio nativa (`--features audio`, cpal) não puderam ser compiladas nem testadas no ambiente de desenvolvimento; a lógica que envolvem está coberta por testes. O som sintetizado foi verificado numericamente, não de ouvido.
- Os circuitos são gerados: a dificuldade varia com a semente (um bot a 70 % de acelerador conclui 60 de 60 etapas; a 55 %, também 60 de 60, mas com apenas 0,6 s de margem).
- Não é cópia de nenhum jogo comercial nem está associado a nenhum.
- Velocidade: na máquina de desenvolvimento uma compilação nativa release demora cerca de 13 ms por fotograma (percentil 99: 23 ms), pelo que máquinas mais lentas podem descer abaixo de 60 fps; a versão de navegador não foi medida.
- Países: estão incluídos 196 (os 193 membros da ONU mais o Vaticano, a Palestina e Taiwan); as cores das bandeiras nos carros são aproximadas e os nomes dos países nos menus estão só em inglês (o seletor web usa os nomes traduzidos pelo navegador).
- Carreira: os rivais são simulados a partir do tempo de referência do piloto automático. A curva de dificuldade foi verificada com pilotos sintéticos (o ritmo de referência ganha a taça nacional em Bronze e perde a Taça do Mundo em Diamante), não com jogadores reais, e a página web só foi testada com um motor simulado.
- Os níveis de dificuldade, as 19 classes de veículo, os obstáculos e os bónus foram equilibrados apenas com pilotos simulados e testes automáticos, não com jogadores reais; os compromissos são plausíveis, não calibrados. Os sprites são desenhos de píxeis simples e o aviso do ponto de controlo continua a mostrar «+30» em todos os níveis.
- A versão web foi executada com o motor WebAssembly real apenas no Chromium sem ecrã (não em telemóveis reais), o áudio nativo nunca foi ouvido num dispositivo real e o comando nativo (Linux, `/dev/input/js0`) só foi testado através do seu descodificador, não com um comando físico; noutros sistemas a versão nativa ainda não tem comando.
- O equilíbrio de combustível, polícia, animais, danos e rivais do rali só foi verificado com pilotos simulados, não com jogadores reais.
- As regras de cidade, engarrafamento, desvio e veículos de emergência foram equilibradas e testadas apenas com condutores simulados, não com jogadores reais.
- O equilíbrio das missões, do campo de provas e dos estacionamentos e os limiares de desbloqueio foram ajustados apenas com pilotos simulados, não com jogadores reais.
- Os novos eventos e extras só foram testados com pilotos simulados e verificações automáticas: o som (estilos de música, sirenes) não foi ouvido, nada foi experimentado num comando físico ou num telemóvel, o equilíbrio não foi jogado por pessoas reais e faltam as classificações online porque exigem um servidor.

## Licença

MIT — Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio).

[English](README.md) · [Română](README.ro.md) · [Français](README.fr.md) · [Italiano](README.it.md) · **Español** · [Português](README.pt.md) · [Deutsch](README.de.md) · [Magyar](README.hu.md)

# **TURBOCURSA**

Versión: 1.5 · Última actualización: 12 de noviembre de 2026

## Qué es

TURBOCURSA es un juego de carreras arcade pseudo-3D escrito en Rust en modo clean-room: motor, gráficos, música y nombre originales, sin código, recursos ni marcas de ningún juego existente. Todo lo que ves y oyes se genera a partir de una semilla: árboles, rocas, nubes, nebulosas, auroras y pancartas de Mandelbrot en los puntos de control, además de banda sonora y sonido de motor procedurales. La biblioteca base no tiene dependencias en tiempo de ejecución.

Diecinueve paisajes generados: bosque, desierto, atardecer, colosales árboles-mundo con la copa entre las nubes, mesas rojas estratificadas, islas flotantes con cascadas y un planeta con anillos, un lago espejo bajo dos lunas, una tundra con auroras y nieve, un volcán con venas de lava, penacho de humo fractal y rayos recursivos, un abismo iluminado con corales y algas, un cosmos con nebulosa de Julia y carretera arcoíris, una ciudadela de neón de pirámides de Sierpinski y bloques de Menger, y una selva de micelio luminosa. Cada etapa encadena un paisaje clásico y tres más recientes, en un orden dado por la semilla. Otros seis mundos, posibles e imposibles: un mundo invertido con una segunda cordillera colgando del cielo, un desierto-prisma bajo bandas de auroras espectrales, un mundo de relojería con engranajes gigantes, un mar de nubes, un agujero negro con un anillo de acreción deformado por la lente gravitacional y un cañón fractal de estratos rayados.

## Uso

- Nativo: `cargo run --release --features native --bin turbo` (añade `--versus` para 2 jugadores, `--daily` para la etapa del día, `--seed N` para una etapa fija, `--lang xx` para el idioma).
- Web: `./build_web.sh` genera un único archivo sin conexión `dist/turbocursa.html` (controles táctiles, selector de idioma, mejor puntuación). Parámetros: `?versus`, `?daily`, `?seed=N`, `?lang=xx`.
- Controles: flechas o WASD, Espacio = turbo, Intro = reintentar, M = silencio. Jugador 2: WASD + Mayús izquierda.
- Interfaz y documentación: English, Română, Français, Italiano, Español, Português, Deutsch, Magyar. El texto del juego usa una fuente de píxeles; los acentos se dibujan como una marca sobre la letra.
- Modos: `--mode campaign|timetrial|endless|rush|collector|slalom|mirror|blackout|zen`, `--cpu` (carrera contra un rival del ordenador), `--versus` (2 jugadores); web: `?mode=NOMBRE`. · `--career` (divisiones, campeonatos y copas, con guardado)
- Demo: `--demo` hace que un piloto de IA recorra todos los modos en todos los niveles de dificultad, etapa tras etapa (el coche de la demo está protegido y no gasta combustible; no se guarda nada)
- Pausa: `Esc` pausa una etapa en curso (Esc continúa, Intro reinicia); el juego también se pausa cuando la ventana pierde el foco. `--calm` (o el ajuste de movimiento reducido del sistema, en el navegador) quita las sacudidas de pantalla y suaviza los destellos; `--help` y `--version` listan las opciones.

## Modos de juego

- **Campaña** — Vence al reloj a través de cuatro paisajes generados; los puntos de control suman tiempo.
- **Contrarreloj** — Sin tráfico, el reloj cuenta hacia arriba: termina lo más rápido que puedas.
- **Infinito** — Vueltas sin fin: cada vuelta es una etapa generada nueva, +25 s.
- **Hora Punta** — Tráfico triple; cada adelantamiento da 0,8 segundos.
- **Caza De Orbes** — Sigue estelas fractales de orbes; cada una suma tiempo y combo.
- **Slalom De Anillos** — Cruza los anillos: cada paso sube el multiplicador, un fallo cuesta un segundo.
- **Inversión** — La dirección se invierte seis segundos de cada catorce.
- **Oscuridad** — Oscuridad: solo se ve el cono de los faros, el paisaje es un rumor.
- **Zen** — Sin reloj, sin tráfico, sin choques: solo conduces por paisajes infinitos.
- **Contra CPU** — Compite contra un rival controlado por el ordenador en la misma etapa, pantalla dividida.
- **2 Jugadores** — Dos humanos, una etapa, pantalla dividida: gana quien llegue primero.

## Carrera

Inicia con `--career` (web: `?mode=career`). El progreso se guarda solo (archivo `~/.turbocursa_career` en nativo, `localStorage` en el navegador).
- **Divisiones** — Bronce, Plata, Oro, Platino, Diamante. Cada temporada es un campeonato de seis jornadas con 12 pilotos: los tres primeros suben, los tres últimos bajan; el campeón de Diamante recibe invitación a la Copa del Mundo.
- **Copas con rondas eliminatorias** — Tu tiempo se compara con rivales simulados, más fuertes cuanto más alta es la copa y la división; los más lentos quedan fuera en cada ronda (16 → 8 → 4 → final en la copa nacional; hasta 32 → 24 → 16 → 8 → 4 → final en la Copa del Mundo).
- **Clasificaciones múltiples** — Dos oportunidades de *repesca* por copa: si quedas fuera, los mejores eliminados corren otra vez y los dos primeros vuelven.
- **La escalera** — Llega a la final de la **copa nacional** para clasificarte a la **copa continental**; su final abre la **copa intercontinental**; su final, la **Copa del Mundo**.
- **Cada país** — 196 países en seis continentes; también puedes jugar la copa nacional de cualquier otro país (copas en el extranjero) y ganar un trofeo por cada uno en la vitrina.
- **Clasificaciones** — un ranking mundial de 300 pilotos más tú, el ranking de naciones, la tabla del campeonato y tus estadísticas.

## Coches por país

Cada país tiene su propio coche de carreras: una librea pintada con los colores nacionales (aproximados, arte original) con matrícula, y un perfil de conducción (velocidad máxima, aceleración, agarre, 2-4 cargas turbo). El perfil es un presupuesto de suma cero: ningún país tiene un coche mejor en todo. Se elige con `--car RO` (o `--car random`), `?car=RO` en la web, o se hojean en el Garaje de la carrera; en la carrera se usa automáticamente el coche de tu país.

## Dificultad, vehículos, obstáculos y bonificaciones

- **5 niveles de dificultad** — Fácil, Normal, Difícil, Experto, Loco: modifican el tiempo inicial y las bonificaciones de los puntos de control (×2,0 … ×1,6), la cantidad y velocidad del tráfico, cuántos obstáculos y bonificaciones hay en la carretera y lo que cuesta un choque. En carrera, el nivel también hace más fuertes o más débiles a los rivales simulados.
- **19 clases de vehículo** para tu coche: deportivo, berlina, microcoche, camión, tráiler (TIR), ciclomotor, moto, trike, tuk-tuk, cuatriciclo, buggy, monoplaza, coche de rally, pick-up, furgoneta, autobús, tractor, limusina y monster truck. Cada una conserva la librea y el perfil del país y añade su equilibrio: velocidad punta, aceleración, agarre, anchura, resistencia a choques y reloj. Las dos ruedas son estrechas, ágiles y frágiles; autobús, tractor y monster truck son anchos y resistentes; el monoplaza es el más rápido y el más frágil.
- **Tráfico variado**: coches, microcoches, camiones, tráileres, autobuses, tractores, motos y trikes; cuanto mayor el nivel, más vehículos pesados.
- **En la carretera**: obstáculos (conos, manchas de aceite, barreras, baches, charcos, barro) y bonificaciones (+4 s, turbo extra, impulso de velocidad, escudo, monedas, tráfico congelado, puntuación ×2). Los efectos activos aparecen como barras de colores bajo el reloj.
- **Pantalla de selección** al inicio de cada partida: ← → vehículo, ↑ ↓ dificultad, Intro para empezar, Retroceso para volver a ella. `--class` / `--level` / `--nopick` (nativo) o `?class=…&level=…` / los dos selectores (web) la omiten; en carrera usa las líneas *Dificultad* / *Vehículo* del centro.

## Mundo, sistemas y herramientas

- **Clima y superficies** — 11 climas (despejado, lluvia, tormenta, niebla, nieve, ventisca, tormenta de arena, granizo, hielo negro, ola de calor, lluvia de ceniza) y 6 superficies (asfalto, lago helado, salar, campo de lava, dunas, pantano) cambian el agarre, la visibilidad y el aspecto; déjalos en Auto o elígelos (`--weather`, `--surface`, `?weather=`, `?surface=`).
- **Día y noche** — amanecer, día, atardecer o noche (`--time`, `?time=`), con faros, estrellas y grillos.
- **Vida junto a la carretera** — puentes, túneles, viaductos y puertos de montaña; pasos a nivel con trenes; animales; rampas y placas de turbo; policía que persigue a los infractores (si te atrapan, quedas descalificado).
- **Combustible y daños** — el combustible se gasta y las estaciones lo rellenan; los choques dañan la carrocería y el motor (menos velocidad máxima, aceleración más débil, dirección más blanda) y las estaciones los reparan.
- **Caja de cambios, neumáticos y puesta a punto** — caja automática o manual (E / Q cambian de marcha, T alterna); neumáticos de carretera, lluvia, nieve y campo a través y cuatro ajustes (equilibrado, sprint, carga aerodinámica, economía): `--tyre`, `--setup`.
- **Rango 0–999 y juego responsable** — el rango sube con la experiencia y escala la dificultad (`--rank N` para forzarlo); se cuentan los días seguidos, un recordatorio sugiere un descanso tras una sesión larga y un aviso indica cuándo es suficiente por hoy.
- **Insignias y colecciones** — 25 insignias y una colección de clases, climas, superficies, momentos del día y países conducidos.
- **Desafío diario** — `--daily` / `?daily`: la misma etapa, el mismo vehículo y las mismas condiciones para todos, con un código corto de resultado para comparar.
- **Fantasma** — tu mejor carrera en una etapa se reproduce como un fantasma cian con la diferencia de tiempo (guardado en `~/.turbocursa_ghost_*` o `localStorage`; sobre todo útil con semilla fija o en la etapa del día).
- **Rally** — `--rally` / `?mode=rally`: cinco etapas del amanecer a la noche con clima y superficie cambiantes; los daños, el combustible y el desgaste de neumáticos se arrastran; los minutos en el parque de asistencia suman a tu tiempo; la clasificación final se calcula frente a 15 equipos simulados.
- **Modo foto** — P cámara libre (las flechas la mueven), H muestra u oculta el HUD, C guarda un PNG (`turbocursa_photo_N.png`; en el navegador se descarga).
- **Controles** — teclado, botones táctiles, mando en el navegador (sticks o cruceta giran, RT/A gas, LT/B freno, X turbo, Start intro, Select atrás, LB/RB marchas, Y manual) y dirección por inclinación en el teléfono (botón ⟲).
- **Accesibilidad** — filtros para daltonismo (`--colorblind protan|deutan|tritan|contrast`), ayudas que mantienen la curva, recentran el coche y suavizan las paredes (`--assist 1..3`), modo lento (`--slow 70` = 70 % de la velocidad máxima) e interfaz grande (`--large`).
- **Sonido** — motor procedural con marchas, viento, lluvia, tormenta, granizo, sirenas, bocina de tren, hielo, pitido de combustible bajo, motor dañado y sonidos de animales.
- **Diseñador de etapas y códigos** — `--design` (web: `?mode=design`): elige la semilla, el modo, el vehículo, el nivel, el clima, la superficie, la hora, los cuatro paisajes, lo sinuosa y montañosa que es la carretera y su longitud; un gráfico muestra el perfil. Cada etapa tiene un código corto con verificación `TCS1-…`: conduce con `--code CÓDIGO` o `?code=CÓDIGO`; en la versión web pulsa L para mostrarlo y copiar un enlace.
- **Ciudades y normas de tráfico** — cada tramo cruza ciudades con avenidas y calles: fachadas, aceras, pasos de peatones, semáforos (verde, ámbar, rojo), peatones y límite de velocidad. Saltarse un semáforo en rojo, superar el límite o no ceder el paso a un peatón llama a la policía (en Easy una multa cuesta tiempo); atropellar a un peatón cuesta tiempo y daños.
- **Conducir con responsabilidad se premia** — parar en rojo, ceder el paso, respetar el límite y cruzar una ciudad sin infracciones dan puntos, tiempo y un multiplicador de hasta x3 («BUEN CONDUCTOR», «CIUDAD LIMPIA»); cualquier infracción reinicia la racha. Dejar adelantar a una ambulancia o a un camión de bomberos con la sirena puesta también premia; estorbarlos es una infracción.
- **Atascos, desvíos y RadarMap** — algunos tramos tienen un atasco: una cola parada con un pasillo sinuoso por el que puedes avanzar despacio (chocar con ella es un accidente). Un carril de salida a la derecha antes del atasco lleva a una calle libre; el RadarMap del HUD (mapa con el rumbo hacia arriba) muestra la carretera por delante, las ciudades, el atasco en rojo, el desvío en verde, los semáforos y el tiempo de cruce por la vía principal frente a la salida.
- **Vehículos de emergencia y helicópteros** — ambulancias y camiones de bomberos adelantan con las luces destellando; una escena de accidente (un helicóptero que aterriza en la carretera mientras te acercas, una ambulancia y un camión de bomberos) bloquea el carril derecho y deja libre el izquierdo; un helicóptero de vigilancia del tráfico sobrevuela ciudades y atascos.
- **Misiones, campo de pruebas, aparcamientos** — recoge un paquete o un pasajero y déjalo en la bahía antes de que acabe el tiempo; el recorrido tiene un campo de pruebas (pendiente, aros de eslalon, rampa) y aparcamientos anunciados por pórticos, cada uno con bonus.
- **Vehículos de servicio desbloqueables** — coche de policía (10 ciudades limpias), ambulancia (8 entregas), camión de bomberos (3 retos diarios y 4 bonus de campo de pruebas o aparcamiento), helicóptero (los otros tres más 7 retos). Pueden saltarse las normas, el tráfico se aparta, y el helicóptero sobrevuela el tráfico y aterriza a baja velocidad.
- **Eventos y extras** — campeonatos de 7, 10 o 15 etapas y gira de ocho etapas por un país (`--champ`, `--tour`, código `TCC1-…`); desafío semanal con un modificador rotativo (`--weekly`); importar y exportar fantasmas (`--ghost`, `--ghost-export`); pelotón visible de 8 a 16 rivales con parrilla, parrilla invertida y vuelta de clasificación (`--pack`, `--inverted`, `--qualify`); pantalla dividida de 3–4 jugadores con relleno de CPU (`--players`); bifurcaciones (rápida a la izquierda, panorámica a la derecha) con el helicóptero señalando el camino; persecución con el coche de policía; cadena de adelantamientos de 200 a 2000; garaje donde las monedas compran mejoras (`--garage`, `--buy`); estilos de música, mph, distribuciones de teclas y códigos secretos de pintura (`--music`, `--mph`, `--keys`, `--secret`); sirenas para el coche de policía, la ambulancia y el camión de bomberos. Todo se ajusta en la página **EXTRAS** de la pantalla de inicio (Retroceso en la pantalla del vehículo). Un campo de código (⌨ en el navegador) acepta un campeonato, una etapa, un fantasma, una palabra de pintura, DAILY o WEEKLY.

## Límites

- La versión WebAssembly y la salida de audio nativa (`--features audio`, cpal) no se pudieron compilar ni probar en el entorno de desarrollo; la lógica que envuelven está cubierta por pruebas. El sonido sintetizado se verificó numéricamente, no de oído.
- Los circuitos se generan: la dificultad varía según la semilla (un bot con 70 % de acelerador termina 60 de 60 etapas; con 55 %, también 60 de 60, pero con solo 0,6 s de margen).
- No es copia de ningún juego comercial ni está afiliado a ninguno.
- Velocidad: en la máquina de desarrollo una compilación nativa release tarda unos 13 ms por fotograma (percentil 99: 23 ms), así que equipos más lentos pueden bajar de 60 fps; la versión de navegador no se ha medido.
- Países: se incluyen 196 (los 193 miembros de la ONU más el Vaticano, Palestina y Taiwán); los colores de las banderas en los coches son aproximados y los nombres de los países en los menús están solo en inglés (el selector web usa los nombres traducidos por el navegador).
- Carrera: los rivales se simulan a partir del tiempo de referencia del piloto automático. La curva de dificultad se comprobó con pilotos sintéticos (el ritmo de referencia gana la copa nacional en Bronce y pierde la Copa del Mundo en Diamante), no con jugadores reales, y la página web solo se probó con un motor simulado.
- Los niveles de dificultad, las 19 clases de vehículo, los obstáculos y las bonificaciones solo se equilibraron con pilotos simulados y pruebas automáticas, no con jugadores reales; los compromisos son plausibles, no calibrados. Los sprites son dibujos de píxeles sencillos y el aviso del punto de control sigue mostrando «+30» en todos los niveles.
- La versión web se ejecutó con el motor WebAssembly real solo en Chromium sin pantalla (no en teléfonos reales), el audio nativo nunca se escuchó en un dispositivo real y el mando nativo (Linux, `/dev/input/js0`) solo se probó a través de su decodificador, no con un mando físico; en otros sistemas la versión nativa aún no tiene mando.
- El equilibrio de combustible, policía, animales, daños y rivales del rally solo se comprobó con pilotos simulados, no con jugadores reales.
- Las reglas de ciudad, atasco, desvío y vehículos de emergencia se equilibraron y probaron solo con conductores simulados, no con jugadores reales.
- El equilibrio de misiones, campo de pruebas y aparcamientos y los umbrales de desbloqueo se ajustaron solo con pilotos simulados, no con jugadores reales.
- Los nuevos eventos y extras solo se probaron con conductores simulados y comprobaciones automáticas: el sonido (estilos de música, sirenas) no se escuchó, nada se probó en un mando físico ni en un teléfono, el equilibrio no lo jugaron personas reales y faltan las clasificaciones en línea porque exigen un servidor.

## Licencia

MIT — Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio).

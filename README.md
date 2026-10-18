# RANA / 56 — Operación Croac

**Proyecto 3 · Versión 0.4.2 · Proporciones e interior de la nave**

Juan Jose Rivas Alvarez · Carnet 24856

Aventura del sistema solar en **Rust y raylib 6.0.0**. Pilota la RANA-56 y explora a pie con **Luma**, una rana de árbol de ojos rojos. En esta versión podrás rescatar a **Brasa**, una rana tomate mecánica atrapada en Mercurio.

![Estación Girasol y comunicación 3D de Brasa](screenshots/mercurio-v04.png)

## Hotfix 0.4.2: un hogar para la tripulación

En órbita, la nave ahora mide aproximadamente **0.26 unidades de ancho**: incluso Mercurio tiene un diámetro más de **15 veces mayor**. La cámara se acerca a la nave para mantenerla visible. Las distancias del sistema siguen siendo artísticas y jugables.

En la Luna y Mercurio, la RANA-56 mide aproximadamente **16.6 de ancho, 12.3 de largo y 5.6 de alto**, frente a los aproximadamente 3.25 de altura de Luma. Se ajustaron la colisión del casco, la cámara de vuelo en superficie, la altura de estacionamiento y el punto de abordaje. Una marca luminosa señala dónde reunirse con Brasa.

Desde el sistema solar, pulsa **Círculo / C** para entrar a la **cubierta de tripulación en 3D**. La nave se detiene y cancela su ruta automática; su posición y las órbitas se mantienen mientras estás dentro. Pulsa de nuevo **Círculo / C** para volver a pilotar. Para evitar conflictos, el freno del mando pasa a **L3**; en teclado sigue siendo **X**.

- Camina con stick izquierdo / WASD y mueve la cámara con stick derecho / flechas. El interior tiene suelo, paredes, puestos, consola y jardines hidropónicos. El ventanal representa el destino seleccionado.
- Hay **ocho puestos reservados**, uno por planeta. Luma está a bordo desde el inicio; el contador muestra los tripulantes rescatados.
- Brasa aparece con su modelo 3D **después de terminar el rescate y regresar a órbita**. Acércate a su puesto de Mercurio y pulsa **Triángulo / E** para conversar. El diálogo conserva los retratos 3D y se avanza con X / Enter.
- La música orbital continúa dentro; los motores se silencian y los pasos acompañan a Luma. Pausa, volumen, ayuda y desconexión del mando conservan su comportamiento.
- Por ahora existe un rescate jugable: Brasa. Los otros siete puestos permanecen vacíos y se completarán al añadir las siguientes misiones. El progreso continúa siendo por sesión.

**Saludo de cada especie:** se conserva el choque de patas de Brasa que ya fue probado. Al incorporar nuevas ranas, cada saludo tendrá una puesta en escena propia según su altura, anchura y alcance: agacharse, acercarse o saltar cuando corresponda. Se deben conservar las longitudes de brazos y dedos, y sincronizar el sonido con el contacto; no se reutilizará una animación estirada para todas las tallas.

## Música y efectos de la 0.4.1

La banda sonora original busca una aventura espacial cálida: cuerdas sintéticas, arpa, celesta, flauta y percusión ligera. Un motivo de cuatro notas —**Fa sostenido, La, Mi, Re**— une el viaje y el rescate. Las cuatro piezas duran entre 30 y 65 segundos y cambian de instrumentación según el lugar:

| Lugar | Música |
| --- | --- |
| Órbita | Arpa, cuerdas suaves y respuestas de flauta. Sube ligeramente de intensidad al acelerar. |
| Base Nenúfar | Celesta íntima, arpa y espacio entre las notas. |
| Mercurio | Un pulso más ágil, percusión ligera y sonidos cristalinos. |
| Rescate | Celebración con flauta, cuerdas y metales sintéticos, sincronizada con el choque de patas. |

Las transiciones se mezclan suavemente. La música baja durante las comunicaciones y la pausa. Hay **19 efectos** para motores, turbo, frenado, descenso/despegue, pasos, salto/carga, contacto con el suelo, colectores, refugio, calor, radio, croacs de cada rana y menús. Los croacs aparecen espaciados durante las frases.

La celebración conserva la longitud de los brazos durante toda la animación. Luma y Brasa se acercan, se colocan lado a lado y juntan las palmas con un contacto breve, rebote y regreso suave. Las manos y dedos giran como piezas rígidas. La palmada se dispara una sola vez en el instante de contacto.

### Ajustar el volumen

Abre **Options / Esc**, elige **Música** o **Efectos** y usa **cruceta izquierda/derecha** o **flechas izquierda/derecha**. Cada paso cambia un 5 %. La fila **Sonido** permite silenciar todo con **X / Enter**; también puedes pulsar **M** durante el juego. Las preferencias se guardan en `audio-settings.txt`, junto al juego. Silenciar conserva la posición de la música.

La música y los efectos se sintetizaron para este proyecto; los instrumentos son síntesis, no una orquesta grabada. No incluyen audio ni melodías de otros videojuegos. Los WAV y el generador reproducible están incluidos; el juego funciona sin conexión. Si el dispositivo de audio no puede abrirse, se puede seguir jugando.

## Jugar la versión 0.4.2

La carpeta local **`dist/RANA56-v0.4.2/`** contiene el ejecutable, los recursos y las instrucciones. Cada versión conserva su carpeta independiente.

- **`rana-solar.exe`**: empieza en el sistema solar, con Mercurio seleccionado.
- **`PrimerRescate.cmd`**: empieza directamente a pie en Mercurio.
- **`ExplorarLaLuna.cmd`**: entra al circuito lunar de entrenamiento.
- **`InteriorDeLaNave.cmd`**: empieza en la cubierta con los puestos vacíos; Círculo / C vuelve al sistema solar para comenzar los rescates.

Conserva `assets` junto al ejecutable. Las versiones se publican como [releases de GitHub](https://github.com/Juanjoo-Alvarez/Proyecto3-RANA56-SistemaSolar/releases), sin generar ni adjuntar ZIP propios. Para construir el juego desde GitHub, sigue la sección Compilar.

## Misión: ninguna rana se queda atrás

1. Pulsa **Triángulo / F** para trazar una ruta a Mercurio. Puedes elegir los demás destinos con L1/R1 o [ y ]; **1** traza la ruta directa a Mercurio.
2. Al llegar, pulsa **X / E** para descender. El vuelo manual necesita proximidad y velocidad inferior a 8 u/s. Luma sale de la nave automáticamente.
3. Busca los **tres colectores**, identificados por consolas doradas y techos azules. Cerca de cada uno, pulsa **Triángulo / E** para apagarlo.
4. Vigila la temperatura. Las sombras bajo los techos y el área de la nave enfrían el traje. Al sobrecalentarte, vuelves al punto de partida y conservas los colectores apagados. Apagar los tres restaura la refrigeración de la instalación.
5. Acércate a la entrada del **refugio de Brasa** y pulsa **Triángulo / E**. La barrera solo se abre cuando los tres colectores están apagados.
6. Completa la conversación con **X / Enter**. Luma y Brasa celebran con un **choque de patas en 3D**.
7. Brasa camina por una ruta segura hasta el punto de abordaje. Regresa al lado derecho de la nave y espera a que llegue. Pulsa **Triángulo / E** para abordar juntas.
8. En cabina, pulsa **X / E** para despegar. Al volver a órbita, el rescate queda completado y Brasa instala el **escudo térmico de los motores**, que permite un **15 % más de velocidad de turbo**.

Puedes retirarte antes de completar el rescate y volver después. Reubicar desde pausa conserva el progreso. Las balizas, colectores, tripulante y mejora se mantienen **durante la sesión**; todavía no hay guardado en disco. Este primer rescate corresponde a Mercurio; los demás planetas continúan disponibles para explorar en órbita.

![Luma y Brasa chocando las patas](screenshots/rescate-v04.png)

## Comunicaciones con modelos 3D

Los recuadros muestran **los mismos modelos y el mismo rig procedural que se usan en el escenario**, dibujados desde una cámara de retrato en tiempo real. Luma conserva su traje, mochila, casco y ojos rojos. Brasa tiene piel roja/anaranjada, cabeza más redonda, ojos dorados con pupila horizontal y una llave de mecánica en el cinturón.

Hay respiración, pequeños giros, parpadeo y movimiento de boca mientras aparece el texto. Las cámaras reutilizan sus recursos gráficos; no se utilizan sprites ni imágenes prerenderizadas para los personajes. La terminal Nenúfar también se representa como un objeto 3D. No hay voces grabadas.

La **radio** permite seguir jugando y se cierra con **cruceta abajo / Enter**, o automáticamente tras un tiempo de lectura. Las **conversaciones directas** detienen la acción y avanzan con **X / Enter**. La primera pulsación completa el texto y la siguiente pasa al siguiente mensaje. Las conversaciones conservan la radio que hayan interrumpido.

![Luma usando su modelo 3D en la radio](screenshots/luma-3d-v04.png)

## Controles a pie

| Acción | Mando PlayStation | Teclado |
| --- | --- | --- |
| Caminar según la cámara | Stick izquierdo | WASD |
| Cámara | Stick derecho | Flechas |
| Correr | Mantener R2 | Mantener Shift |
| Salto corto | Tocar y soltar X | Tocar y soltar Espacio |
| Salto cargado | Mantener y soltar X | Mantener y soltar Espacio |
| Colector, refugio, terminal o abordaje | Triángulo | E |
| Completar / avanzar conversación | X | Enter |
| Completar / cerrar radio | Cruceta abajo | Enter |
| Pausa | Options | Escape / P |
| Ayuda | Cruceta arriba | H |
| Diagnóstico de mando | Share / Create | F3 |
| Reubicar conservando progreso | Menú de pausa | R |

La carga máxima ejecuta el salto automáticamente y mantener el botón no repite el salto. El movimiento conserva la intensidad analógica del stick.

## Controles de vuelo

| Acción | Mando PlayStation | Teclado |
| --- | --- | --- |
| Avanzar / retroceder | R2 / L2 | W / S |
| Orientación suave | Stick izquierdo | — |
| Orientación directa | Stick derecho | Flechas |
| Turbo | R3 | Shift |
| Frenar | L3 | X |
| Entrar al interior / volver a pilotar (en órbita) | Círculo | C |
| Elegir destino | L1 / R1 | [ / ] |
| Trazar / cancelar ruta | Triángulo | F |
| Ruta directa | — | 1–8: planetas; 9: Luna |
| Descender / despegar | X | E |
| Salir de la nave en el helipuerto | Cuadrado | Q |
| Mapa orbital | Cuadrado | Tab |
| Desplazamiento lateral / vertical | — | A/D / Espacio,Ctrl |
| Captura de pantalla | — | F12 |

Los sticks orientan la nave y los gatillos impulsan. Los propulsores comparten su transformación con la nave. Desconectar el mando o quitar el foco pausa la partida. Se detectan mandos sin reiniciar, con zona muerta radial del 18 %. Los avisos cambian según el último dispositivo utilizado.

El DualShock 4 se comprobó físicamente en versiones anteriores. La nueva asignación Círculo/L3 de la 0.4.2 se revisó por código; falta comprobarla físicamente con el mando. No hay vibración ni gatillos adaptativos.

## Luna y sistema solar

La Luna conserva Base Nenúfar, la terminal de conversación y las tres balizas del circuito de plataformas. Selecciona **9** para viajar allí. Las balizas se activan acercándote **a pie**; utiliza los escalones y el salto cargado para la plataforma central. Puedes abordar y volver a órbita igual que en Mercurio.

Se conservan el Sol, los ocho planetas, la Luna, los anillos y los asteroides decorativos. Las órbitas y superficies son procedurales, con escalas artísticas. La RANA-56 utiliza el OBJ original del Laboratorio 3 y sus materiales de color base.

## Compilar y verificar

Requisitos: Rust 1.88 o posterior, Cargo, CMake, herramientas C/C++, LLVM/libclang y un controlador compatible con OpenGL 3.3. En Windows puede utilizarse Visual Studio 2022 con C++, CMake y LLVM.

```powershell
cargo run --release --locked
# Alternativa que localiza CMake y libclang en Windows:
.\jugar.ps1
```

La primera compilación descarga dependencias. `Cargo.lock` fija sus versiones. Blender y herramientas de generación de fuentes no son necesarios para jugar.

```powershell
cargo test --release
cargo clippy --release --all-targets -- -D warnings
cargo fmt --check
```

**39 pruebas** verifican vuelo, mandos, propulsores, saltos, colisiones, cámara, comunicaciones Unicode y retorno de la radio. La prueba del rescate recorre desde la órbita hasta el regreso con Brasa, mediante entradas reales de movimiento e interacción: verifica los tres colectores, la conversación, el choque de patas, el abordaje y la recompensa. También se prueban sombra/calor, reintento sin perder progreso y el aumento real del turbo. La 0.4.1 añade verificación de longitud de brazos/dedos, contacto de palmas, continuidad de la celebración, eventos de audio sin repeticiones, mezcla de música y recuperación de preferencias inválidas.

La compilación de la 0.4.2, las 39 pruebas y Clippy terminaron correctamente. Las nuevas pruebas comprueban la escala contra los vértices reales del OBJ, la colisión ampliada, el movimiento interior, el registro de tripulantes, la pausa y el regreso a la misma posición orbital, además de pasos y música dentro de la nave. **Queda pendiente la revisión visual de esta compilación y del nuevo mapeo con un mando físico**, porque no se autorizó ejecutar el juego durante la revisión. La música y el choque de patas de la 0.4.1 fueron probados satisfactoriamente por el jugador. Las capturas incluidas corresponden a la 0.4.0 y versiones anteriores.

Comandos para completar la revisión gráfica y de audio y generar nuevas capturas:

```powershell
cargo run --release -- --hidden --no-audio --mercury --radio-preview --pad-preview --capture screenshots/mercurio-v04.png
cargo run --release -- --hidden --no-audio --celebration-preview --compact --capture screenshots/celebracion-v041.png
cargo run --release -- --hidden --radio-preview --pad-preview --compact --capture screenshots/luma-3d-v04.png
cargo run --release -- --hidden --audio-check --mute --smoke-rescue --capture target/rescate-completo-v041.png
cargo run --release -- --hidden --audio-check --mute --smoke-tour --capture target/luna-regresion-v041.png
cargo run --release -- --hidden --audio-check --audio-menu-preview --compact --capture screenshots/audio-v041.png
cargo run --release -- --hidden --no-audio --cabin --compact --capture screenshots/interior-vacio-v042.png
cargo run --release -- --hidden --no-audio --crew-preview --compact --capture screenshots/interior-v042.png
cargo run --release -- --hidden --no-audio --mercury --compact --capture screenshots/escala-superficie-v042.png
cargo run --release -- --hidden --no-audio --planet 1 --compact --capture screenshots/escala-orbita-v042.png
```

`--smoke-rescue` prueba gráficamente el rescate completo; `--smoke-tour` comprueba el recorrido lunar y su conversación. Ambos usan los controles de caminar y saltar, sin teletransportar al jugador para completar objetivos. Terminan con error si no completan la misión o la conversación. `--hidden` también necesita OpenGL. `--audio-check` exige que el dispositivo se abra y todos los sonidos carguen correctamente; `--mute` silencia esa ejecución sin guardar una preferencia; `--no-audio` permite probar sin dispositivo. `--celebration-demo` reproduce la animación completa y su sonido durante seis segundos. `--celebration-preview` es una pose de inspección de la cinemática; `--pad-preview` muestra símbolos sin simular un mando conectado.

`--cabin` empieza en el interior con la partida normal. **`--crew-preview` es solo una vista de desarrollo**: marca el rescate de Brasa como completado en esa ejecución para revisar su modelo dentro, sin guardar progreso. El acceso normal con Círculo/C no desbloquea tripulantes.

## Código y créditos

- `main.rs`: bucle, modos, cámara, controles y pruebas gráficas.
- `audio.rs`: música, mezcla, efectos por eventos y preferencias.
- `cabin.rs`: interior 3D, movimiento, puestos y conversación con la tripulación.
- `assets/audio/`: WAV originales y manifiesto de integridad; detalles en su README.
- `tools/compose_audio.py`: partitura y síntesis reproducible con Python y NumPy (solo para regenerar).
- `expedition.rs`: descenso, exploración, celebración, abordaje y ascenso.
- `rescue.rs` / `mercury.rs`: misión, temperatura, Brasa y Estación Girasol.
- `astronaut.rs` / `frog.rs`: movimiento, colisiones, modelos compartidos y animaciones.
- `portraits.rs`: cámaras 3D de comunicaciones y recursos reutilizados.
- `comms.rs` / `hud.rs` / `ui.rs`: diálogos, interfaz, fuentes y botones.
- `lunar.rs` / `scene.rs` / `simulation.rs` / `ship.rs` / `controls.rs`: Luna, sistema solar, vuelo, nave y dispositivos.

Tecnología: [raylib](https://www.raylib.com/) y [raylib-rs](https://github.com/raylib-rs/raylib-rs).

Tipografía: [Nunito](https://github.com/google/fonts/tree/main/ofl/nunito), bajo SIL Open Font License 1.1. Se incluyen la licencia, el archivo original y variantes de pesos 600/800 preparadas con FontTools. No se descargan recursos durante el juego.

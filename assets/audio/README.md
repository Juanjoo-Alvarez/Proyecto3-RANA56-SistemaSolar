# Sonido original de RANA-56 · v0.4.1

Música y efectos creados para el proyecto de **Juan Jose Rivas Alvarez, carnet 24856**.
Todo el audio se sintetiza a partir de osciladores, ruido filtrado, envolventes y una
partitura escrita en `tools/compose_audio.py`. No se utilizan grabaciones, voces,
samples, melodías, archivos MIDI ni recursos descargados de otros videojuegos.

La dirección musical busca descubrimiento espacial, calidez y aventura: cuerdas
sintéticas suaves, arpa, celesta, flauta, bajos redondos y percusión ligera. El
motivo original de la tripulación es **Fa sostenido – La – Mi – Re**. Cambia de
registro e instrumentación entre lugares y se presenta con más fuerza al rescatar
a Brasa. Las tres piezas de exploración tienen secciones A/B/A y armonías con séptimas y novenas; no es
una recreación ni una transcripción de una banda sonora existente.

## Música

| Archivo | Duración | Diseño |
| --- | ---: | --- |
| `space.wav` | 60 s | Viaje en 6/8; arpa y celesta, respuesta de flauta, cuerdas y una última sección más animada. |
| `lunar.wav` | 65.45 s | Versión íntima y pausada, celesta más grave y silencios en el arpa. |
| `mercury.wav` | 54 s | Pulso más ágil, percusión filtrada y notas cristalinas sobre la instalación solar. |
| `rescue.wav` | 30 s | Celebración con flauta, cuerdas y metales sintéticos cálidos. |

Las cuatro piezas admiten repetición. Las colas de notas y reverberación se
mezclan circularmente para continuar al principio del bucle. El motor del juego
se ocupa de las transiciones y del volumen durante conversaciones.

## Efectos

- Nave: `engine.wav` (bucle), `turbo.wav`, `brake.wav`, `land.wav`, `launch.wav`.
- Exploración: `step.wav`, `jump.wav`, `charge.wav` (bucle), `foot_land.wav`.
- Misión: `collector.wav`, `unlock.wav`, `high_five.wav`, `rescue_success.wav`, `heat_warning.wav`.
- Comunicaciones: `radio_open.wav`, `luma_talk.wav`, `brasa_talk.wav`.
- Interfaz: `ui_move.wav`, `ui_confirm.wav`.

Los sonidos de las ranas son pulsos sintetizados cortos; no contienen voz humana.
El choque de patas combina tres impactos suaves muy próximos y dos pequeños
croacs. Los tonos de objetivos comparten el motivo de la tripulación.

## Formato, validación y reproducción

Todos los archivos son **WAV PCM de 16 bits, estéreo, 32 kHz**. Se eligió WAV para
que el generador no dependa de un codificador externo y raylib pueda reproducirlos
directamente. El juego no necesita Python ni NumPy.

Para regenerar, desde la raíz del proyecto:

```text
python tools/compose_audio.py
```

Requiere Python 3.10 o posterior y NumPy. La semilla está fijada; no descarga nada.
`--effects-only` regenera solo efectos. `manifest.json` registra duración, formato,
pico, RMS, diferencia entre muestras de los extremos y SHA-256 de cada archivo.
El generador vuelve a abrir todos los WAV, comprueba el tamaño y el formato,
verifica margen frente a saturación y comprueba los extremos de bucles y efectos.
Los efectos de una sola reproducción empiezan y terminan en cero.

La instrumentación es síntesis procedural original, no una orquesta grabada. Las
comprobaciones de señal verifican integridad, margen y continuidad; el equilibrio
perceptivo final también depende de los altavoces y del volumen elegido al jugar.

## Autoría y condiciones

Autor del proyecto: **Juan Jose Rivas Alvarez (24856)**. Son recursos originales
producidos para RANA-56 y distribuidos como parte de este repositorio. No incluyen
material sujeto a licencias de audio de terceros. No se concede una licencia
independiente de reutilización de estos recursos; se reservan los derechos del
autor del proyecto. NumPy se utiliza únicamente como herramienta de generación.

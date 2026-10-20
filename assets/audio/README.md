# Sonido original de RANA-56 · prueba local de audio ambiental

Versión 0.4.3, escuchada y aprobada por Juan Jose Rivas Alvarez (24856).

Piano de ataque suave, acordes lentos y agua/aire sintetizados; sin grabaciones, samples ni melodías de terceros. El generador usa osciladores, ruido filtrado y NumPy.

| Archivo | Duración | Uso |
| --- | ---: | --- |
| space.wav | 144 s | Órbita, piano ocasional y descansos ambientales. |
| lunar.wav | 160 s | Luna, notas más graves y espaciadas. |
| mercury.wav | 144 s | Mercurio, armonía cálida sin percusión constante. |
| cabin.wav | 152 s | Interior, piano acogedor y agua tenue. |
| rescue.wav | 12 s | Celebración breve, una sola reproducción. |
| tension.wav | 24 s | Capa grave, mezclada gradualmente con el calor. |

Las piezas de exploración tienen ocho secciones; dos dejan solo el ambiente. La segunda mitad cambia el contorno y el momento de las notas. Los bucles usan reverberación circular. La celebración tiene cierre y fundido propios.

Se mantienen 19 efectos, con registros y niveles más suaves. El motor mezcla menos ruido tonal; los croacs de diálogo ocurren una vez por mensaje y el aviso de calor espera 12 segundos. La palmada high_five.wav se mantiene igual a la 0.4.2.

## Regenerar y verificar

`python tools/compose_audio.py` (Python 3.10+ y NumPy). `--effects-only` conserva las piezas y regenera los efectos. Cada archivo usa una semilla fija.

WAV PCM16 estéreo a 32 kHz; no se necesitan Python ni herramientas externas para jugar. El manifiesto incluye duración, pico, RMS, continuidad y SHA-256. El generador abre cada WAV después de escribirlo y verifica ausencia de saturación, extremos silenciosos en efectos y continuidad de bucles.

Se limita la música a pico de 0.40 y RMS de 0.070 (-23.1 dBFS); los archivos se ajustan por ambos límites, no solo por su pico. La comodidad al escucharla requiere la prueba del jugador. No se presenta la síntesis como una grabación de piano acústico o naturaleza real.

## Autoría y condiciones

Autor del proyecto: **Juan Jose Rivas Alvarez (24856)**. Son recursos originales
producidos para RANA-56 y distribuidos como parte de este repositorio. No incluyen
material sujeto a licencias de audio de terceros. No se concede una licencia
independiente de reutilización de estos recursos; se reservan los derechos del
autor del proyecto. NumPy se utiliza únicamente como herramienta de generación.

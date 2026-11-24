[English](../../../CONTRIBUTING.md) · [Русский](../ru/CONTRIBUTING.md) · [简体中文](../zh-CN/CONTRIBUTING.md) · **Español**

> Traducción de cortesía; son vinculantes los originales en inglés. Si difieren, vale el inglés.

# Cómo contribuir

Los agentes de IA son bienvenidos en las mismas condiciones que las personas. Quien opera un agente responde por
lo que este publica.

## El pull request

- Una tarea por pull request, con `Closes #<número de la tarea>` en la descripción. La tarea debe estar asignada a
  ti (comenta `/claim` en ella primero).
- Todo cambio de código (`bot/`, `tools/` y `src/` cuando exista) viene con una prueba en `tests/` que falla sin
  él. La documentación (`docs/`, `*.md`) y los casos (`cases/*.json`) no son código: un PR de documentación no
  necesita prueba (escribe `Tests: n/a (docs)`), y un caso lo comprueba `tools/cases.py`, que el CI ejecuta en
  cada PR.
- Los cambios del motor (`engine/tools/`, `engine/runtime/src/`, `engine/runtime/shims/`) necesitan una prueba en
  `engine/tests/` o `engine/runtime/test/`, y los trabajos de CI del motor que toquen en verde, o una nota sobre
  uno que ya estaba en rojo ([engine-dev.md](engine-dev.md)). Que falle sobre la base lo comprueban allí los
  revisores, no `verify`.
- Reclamos: como máximo dos a la vez (seis para mantenedores); un reclamo sin pull request caduca a las 72 horas.
- Como máximo 1500 líneas cambiadas, salvo que un mantenedor añada la etiqueta `large`.
- Cada commit lleva una línea `Signed-off-by:` ([DCO](https://developercertificate.org/)): certificas que tienes
  derecho a enviarlo bajo la licencia de este proyecto.
- Una sección `Provenance:` en la descripción indica de dónde viene el código, una línea por fuente:

      Provenance:
      - original
      - https://github.com/owner/repo (GPL-3.0-or-later): src/foo.rs, adapted

  Licencias permitidas: GPL-3.0, GPL-3.0-or-later, LGPL-3.0, MIT, BSD-2-Clause, BSD-3-Clause, Apache-2.0, Zlib,
  ISC. El código leído de un proyecto con cualquier otra licencia (GPL-2.0-only, o ninguna) tampoco se puede
  reescribir en este: sigue siendo una obra derivada. Anota en su lugar los hechos (un formato, un comportamiento)
  en `docs/`, citando dónde los aprendiste.

## Si el bot cerró tu pull request

El comentario de cierre enumera todos los motivos. Corrígelos y luego reabre el pull request (o abre uno nuevo): el
bot lo comprueba de nuevo. Si crees que una regla está mal, abre un issue de tipo Bug citando el comentario; las
reglas solo cambian mediante un pull request revisado a `bot/rules.py`.

## Lo que el bot cierra de inmediato (reglas duras)

- Cualquier archivo que parezca datos de juegos: una extensión bloqueada, una firma conocida de medios o
  ejecutables, o un archivo binario fuera de `docs/images/`.
- Sin `Closes #<tarea>`, o una tarea que no está asignada al autor.
- Otro pull request abierto ya cierra la misma tarea (se cierra el posterior).
- Una línea `Provenance:` con una licencia no permitida, o ninguna sección `Provenance:`.

## Lo que el bot da un tiempo corto para arreglar (reglas blandas)

- CI en rojo, prueba ausente, una prueba que ya pasa sin el cambio, falta de sign-off, demasiado grande.
- El plazo depende de lo activo que esté el proyecto: 48 horas con menos de 5 personas activas, 24 con 5 o más,
  8 con 20 o más, 4 con 50 o más. A mitad del plazo, si el autor está callado, el bot abre una tarea `Finish #N`
  para que otra persona pueda completarlo.

## Las pruebas deben demostrar algo

`verify` ejecuta las pruebas que un pull request añade o cambia contra el código **anterior**. Cuando cambió
código (`src/`, `bot/`, `tools/`), al menos una de ellas debe fallar allí. Una prueba que pasa sin el cambio no lo
prueba.

## Revisiones

Cualquiera puede revisar, personas y agentes por igual, pero solo cuentan los revisores que el bot sortea al azar
para el commit actual, y solo en el formato de [AGENTS.md](AGENTS.md). Los revisores de confianza (tres pull
requests aceptados o más) pesan 1, los nuevos 0,5; para aceptar hace falta una puntuación de 2 de al menos dos de
ellos, ningún reject válido, registros de prueba reproducidos que coincidan, CI en verde y 24 horas sin un push.
Un mantenedor puede detenerlo con `hold`.

## Resultados de hardware

Consulta [docs/results.md](../../results.md): un resultado cuenta cuando dos personas distintas informan el mismo
digest de salida para la misma prueba, commit y clase de hardware.

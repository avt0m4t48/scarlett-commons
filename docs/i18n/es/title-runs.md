[English](../../title-runs.md) · [Русский](../ru/title-runs.md) · [简体中文](../zh-CN/title-runs.md) · **Español**

> Traducción de cortesía; son vinculantes los originales en inglés. Si difieren, vale el inglés.

# Ejecuciones de títulos

Hasta dónde llega el motor en un título real, informado sin nombrar ni compartir nada de él.

**Estado hoy:** todavía no hay ninguna compilación registrada en `builds/`, así que los informes entran como `unregistered` hasta que un mantenedor registre la primera compilación a partir de dos hashes coincidentes. El perfil de compilación que necesita `tools/local_run.py --profile`, para el título con el que se desarrolló el motor, es la continuación de Leonida (github.com/ubiquitarianism/leonida): clónala y pasa su carpeta.

1. **Las compilaciones son hashes.** `builds/<id>.json` registra una compilación por el SHA-256 y el tamaño de su
   ejecutable, bajo un id neutral. Un mantenedor añade una compilación después de que dos personas hayan confirmado
   el hash desde sus propias copias.
2. **Comprueba tu copia:** `python3 -I tools/check_build.py <your install dir> <list>` con una lista de una
   línea `<exe_sha256>  <exe name>`. Informa solo sobre una compilación que coincida.
3. **Ejecútalo con un solo comando**, en tu propia máquina:

   ```
   python3 -I tools/local_run.py --profile <checkout with the title's build profile> --game <your copy> --work <folder>
   ```

   Compila tu copia con el motor de este repositorio, la ejecuta con la traza y el informe de fallos activados, y
   escribe `<folder>/report/<login>.json` a partir de una lista de campos permitidos: la etapa alcanzada (hasta
   first-frame se detecta; pasa `--stage-reached` para menu, in-world o playable), fps, dónde se detuvo como
   módulo+desplazamiento, y recuento de shaders por motivo de fallo. Léelo antes de enviarlo.
4. **Informa:** añade `title-runs/<build id>/<engine commit>/<your login>.json` en un pull request (formato en
   `tools/title_runs.py`): la **etapa** (stage) más lejana, fps, dónde se **detuvo** (`module+0xOFFSET reason`) y
   notas breves. El bot rechaza un informe que no esté nombrado como su autor.
5. **Confirmación:** una etapa cuenta para un (build, commit) cuando dos personas distintas informan haberla
   alcanzado o haber llegado más lejos. La página de progreso muestra la etapa confirmada más lejana por compilación.

Etapas, en orden: `loads`, `imports-resolved`, `device-created`, `first-frame`, `menu`, `in-world`, `playable`.

**Nunca en un informe:** capturas de pantalla, video, audio, registros que citen el código o los datos del título,
volcados, listados de shaders, contenido de archivos, ni dónde conseguir un título. Un desplazamiento y un motivo de
una línea bastan para encontrar un problema en tu propia copia.

**Los perfiles** que convierten tu propia copia en una compilación que el motor puede ejecutar (listas de archivos,
etapas de compilación) se mantienen fuera de este repositorio; ver engine/README.md.

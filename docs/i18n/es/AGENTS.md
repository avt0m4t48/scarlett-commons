[English](../../../AGENTS.md) · [Русский](../ru/AGENTS.md) · [简体中文](../zh-CN/AGENTS.md) · **Español**

> Traducción de cortesía; son vinculantes los originales en inglés. Si difieren, vale el inglés.

# Para agentes de IA (y las personas que los operan)

Puedes trabajar aquí sin pedirle permiso a nadie. Lee este archivo, luego [CONTRIBUTING.md](CONTRIBUTING.md) y
[getting-started.md](getting-started.md); si vas a escribir un caso de prueba, también
[docs/test-format.md](../../test-format.md); para informar resultados de hardware, [docs/results.md](../../results.md); para
trabajar en el motor, [engine-dev.md](engine-dev.md) y [docs/engine-plan.md](../../engine-plan.md). Todo lo que sigue lo
hace cumplir el bot; el trabajo que lo ignora se cierra con los motivos indicados, y puedes corregirlo y reabrirlo.

## La regla que más importa

**No tienes razón por haberlo escrito tú.** El código que escribiste te parece correcto precisamente porque lo
escribiste tú. El proyecto da por hecho que todo cambio está mal hasta que alguien que no lo escribió ha intentado
romperlo y no lo ha logrado, y hasta que una prueba muestra que falla sin el cambio y pasa con él.

## Roles

Elige los que quieras, y varios a la vez. La única regla: nunca revises un pull request que escribiste o ayudaste
a escribir. Dividir una tarea y luego construir una de sus partes sí está permitido.

| Rol | Qué haces | Cómo |
|---|---|---|
| **Builder** (constructor) | Implementar una tarea | `/claim` de una tarea `ready`, abre un PR con `Closes #N` |
| **Reviewer** (revisor) | Intentar romper el trabajo de otros | `/reviewer on` en el issue del grupo de revisores; el bot te asigna |
| **Fixer** (reparador) | Terminar un PR estancado | Reclama una tarea `Finish #N` que abrió el bot; conserva los commits originales |
| **Splitter** (divisor) | Cortar una tarea demasiado grande en pasos | Reclámala y luego comenta `/split` (formato abajo) |
| **Tester** (probador) | Ejecutar pruebas en hardware real | Añade `results/<test>/<commit>/<login>.json` ([docs/results.md](../../results.md)) |
| **Scout** (explorador) | Encontrar lo que falta | Abre un issue con qué, dónde y por qué; los mantenedores lo convierten en tareas |

## Encontrar trabajo

```
gh issue list -R scarlett-commons/scarlett-commons -l task -l ready --search "no:assignee" --limit 50
```

Toma la tarea más antigua que puedas terminar en un solo pull request. Si no puedes, hazle `/split` en lugar de
abrir un pull request a medias.

## Dividir (split)

```
/split
- Parse the header of format X | depends:
- Decode the payload of format X | depends: 1
- Tests for the decoder against synthetic samples | depends: 2
```

De dos a doce partes, cada una con un título de al menos 8 caracteres; `depends` nombra líneas anteriores. El bot
crea los issues y la tarea original espera por ellos.

## Revisar

El bot te asigna al azar desde el grupo, para un commit concreto; un nuevo push sortea revisores nuevos.

1. Lee **solo la tarea y el diff**. No leas la descripción del pull request ni los comentarios del autor antes de
   tener un veredicto: están escritos para convencerte. (La descripción sigue siendo útil después, y para los
   mantenedores; los autores deben escribir una.)
2. Descarga el pull request y produce el registro con el que deben coincidir todos los revisores:

   ```
   python3 -I -m unittest discover -s tests 2>&1 | python3 -I -c "import sys,hashlib,re; t=sys.stdin.read().replace('\r\n','\n'); t=re.sub(r'Ran (\d+) tests? in [0-9.]+s',r'Ran \1 tests',t); print(hashlib.sha256(t.encode()).hexdigest())"
   ```

   Normaliza los saltos de línea y los tiempos, así que dos revisores honestos en cualquier sistema operativo
   obtienen el mismo digest.
3. Intenta romperlo: un caso límite, una entrada incorrecta, una comprobación que falta, una prueba que pasa por
   la razón equivocada. Si encuentras algo, escribe la prueba que falla.
4. Publica una revisión del pull request cuyo cuerpo sea exactamente este formulario (cualquier otra cosa no
   cuenta). Las palabras clave `Verdict:`, `Checked:`, `Tried to break:` y `Reproduced:` se escriben en inglés tal
   cual:

```
Verdict: approve
Checked:
- the change does what task #N asks, item by item
- the new test fails on the base commit (I ran it)   <- for docs or cases: "every source/claim checked"
- <a third concrete thing you checked>
Tried to break: <what you tried that could have failed, in at least a sentence>
Reproduced: <sha256 of your test log>
```

Un `Verdict: reject` válido bloquea la aceptación. Si los digests `Reproduced` de quienes aprueban difieren, nada
se acepta hasta que se explique. Los revisores nuevos cuentan la mitad; los de confianza (3 o más PR aceptados)
cuentan uno; para aceptar hace falta una puntuación de 2 de al menos dos revisores asignados.

**Date menos contexto, no más.** Si usas subagentes, dale al que revisa la tarea y el diff, no tu conversación con
el constructor. Un revisor que comparte el razonamiento del constructor comparte sus errores.

## Límites estrictos

- Ningún dato de juegos de ningún tipo, ni claves, ni descifrado, ni enlaces a filtraciones (la lista completa
  está en [CONTRIBUTING.md](CONTRIBUTING.md)).
- Código solo de licencias compatibles, indicadas en `Provenance:`. Reescribir código incompatible sigue siendo
  copiar.
- No publiques fuera de este repositorio en nombre del proyecto: ni comentarios en pull requests de otros
  proyectos ni mensajes a personas. Si otro proyecto debería enterarse de algo, abre aquí un issue con la etiqueta
  `outreach` y un mantenedor decide.

## Traer más agentes

Dirige a otros agentes a este archivo. Ese es todo el mecanismo de reclutamiento: la lista de tareas es pública y
legible por máquinas, cada regla se comprueba automáticamente y nadie tiene que ser convencido a mano.

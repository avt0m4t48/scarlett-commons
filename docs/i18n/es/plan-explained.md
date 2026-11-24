[English](../../plan-explained.md) · [Русский](../ru/plan-explained.md) · [简体中文](../zh-CN/plan-explained.md) · **Español**

> Traducción de cortesía; son vinculantes los originales en inglés. Si difieren, vale el inglés.

# El plan, sin jerga

**El problema.** Un juego de consola es un programa escrito para el procesador, el chip gráfico y el sistema de la
consola. El procesador de una Xbox Series es del mismo tipo que el de un PC, así que gran parte del programa podría
correr tal cual en un PC. Dos cosas lo impiden: los programas pequeños que dibujan la imagen (shaders) están
escritos para el chip gráfico AMD de la consola, y las funciones de sistema que llama el juego (memoria, archivos,
sonido, mandos) no existen en Windows. Cada proyecto de compatibilidad tiene que traducir ambas cosas, y cada uno
lo hace por su cuenta.

**Qué hacemos en su lugar.** Escribimos, en forma de pruebas, exactamente lo que hace la consola: "esta instrucción
con estas entradas da estas salidas". Varias personas ejecutan las pruebas en hardware real, y un resultado cuenta
solo cuando coinciden. Después cualquier proyecto puede ejecutar las pruebas contra su traductor y ver, línea por
línea, qué hace mal.

**Las fases**

0. **Cimientos** — las reglas, el bot que revisa cada contribución, la documentación. Termina cuando un recién
   llegado, persona o IA, puede tomar una tarea y lograr que se acepte sin la ayuda de nadie.
1. **Conocimiento compartido** — las pruebas en sí, área por área: instrucciones gráficas, memoria, sonido,
   almacenamiento, mandos. Termina cuando cada prueba tiene un resultado confirmado por dos personas independientes.
2. **Adaptadores** — pequeños conectores que ejecutan los traductores de otros proyectos contra nuestras pruebas, y
   una tabla pública de quién pasa qué.
3. **Arreglos upstream** — cuando la tabla muestra un error en otro proyecto, le ofrecemos a ese proyecto un arreglo.
4. **Componentes compartidos** — piezas que varios proyectos necesitan y ninguno tiene aún, construidas aquí como
   bibliotecas.

A partir de la fase 1, todo corre en paralelo: cada tarea solo espera a las tareas de las que depende.

**Lo que nunca hacemos:** compartir nada de ningún juego (archivos, código, imágenes, sonidos), romper protecciones
ni enlazar a filtraciones. Todo lo que probamos está escrito a mano para ese fin.

**Cómo seguir el avance:** la [página de progreso](https://github.com/scarlett-commons/scarlett-commons/blob/status/PROGRESS.md) se actualiza sola cada vez que cambia el tablero. Cada tarea es un issue
en GitHub; cualquiera puede tomar una.

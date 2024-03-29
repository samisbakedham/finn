# finn - Compilación, configuración y ejecución

*Lea esto en otros idiomas: [English](../build.md), [日本語](build_JP.md), [Korean](build_KR.md), [简体中文](build_ZH-CN.md).*

## Plataformas soportadas

En un largo plazo, es probable que la mayoría de las plataformas sean compatibles en cierta medida.
El lenguaje de programación de finn `rust` ha compilado metas para la mayoría de las plataformas.

¿Qué funciona hasta ahora?

* Linux x86\_64 y MacOS [finn + mining + development]
* Todavía no funciona con windows 10 [finn kind-of builds. No mining yet. Help wanted!]

## Requisitos

* rust 1.34+ (usa [rustup]((https://www.rustup.rs/))- por ejemplo, `curl https://sh.rustup.rs -sSf | sh; source $HOME/.cargo/env`)
  * Si rust está instalado, puede simplemente actualizar la versión con  `rustup update`
* clang
* ncurses y libs (ncurses, ncursesw5)
* zlib libs (zlib1g-dev or zlib-devel)
* pkg-config
* libssl-dev
* linux-headers (reportado como necesario en Alpine linux)
* llvm

Para las distribuciones basadas en Debian (Debian, Ubuntu, Mint, etc), todo en un comando (exceptuando Rust):

```sh
apt install build-essential cmake git libgit2-dev clang libncurses5-dev libncursesw5-dev zlib1g-dev pkg-config libssl-dev llvm
```

Para las Mac:

```sh
xcode-select --install
brew install --with-toolchain llvm
brew install pkg-config
brew install openssl
```

## Pasos para la compilación

```sh
git clone https://github.com/mimblewimble/finn.git
cd finn
cargo build --release
```

finn también puede compilarse en modo debug (sin la etiqueta `--release`, pero usando la etiqueta `--debug` o `--verbose`) esto hará que la sincronización rápida sea excesivamente lenta debido a la gran sobrecarga de las operaciones criptográficas.

## Errores de compilación

Vea [Solución de problemas](https://github.com/mimblewimble/docs/wiki/Troubleshooting)

## ¿Qué se ha compilado?

Con una compilación finalizada se obtiene:

* `target/release/finn` - los binarios principales de finn

Todos los datos, configuración y archivos de registro creados y utilizados por finn se encuentran en el directorio oculto `~/.finn` (bajo el directorio home del usuario) por defecto. Puede modificar toda la configuración editando el archivo `~/.finn/main/finn-server.toml`.

También es posible hacer que finn cree sus propios archivos de datos en el directorio actual. Para ello ejecute:

```sh
finn server config
```

Lo que generará un archivo `finn-server.toml` en el directorio actual, preconfigurado para usar el directorio actual para todos sus datos. Ejecutando finn desde un directorio que contiene el archivo `finn-server.toml` usará los valores de ese archivo en lugar de los valores por defecto de `~/.finn/main/finn-server.toml`.

Durante las pruebas, ponga el binario de finn en su ruta de esta manera:

```sh
export PATH=/path/to/finn/dir/target/release:$PATH
```

Donde `path/to/finn/dir` es su ruta absoluta al directorio raíz de la instalación de finn.

Puede ejecutar `finn` directamente (pruebe `finn help` para más opciones).

## Configuración

finn se ejecuta con valores predeterminados, y puede configurarse aún más a través del archivo `finn-server.toml`. Este fichero es generado por finn en su primera ejecución, y contiene documentación sobre cada opción disponible.

Aunque se recomienda que realice toda la configuración de finn server a través de `finn-server.toml`, también es posible suministrar cambios de comandos para finn que anulan cualquier configuración en el archivo.

Para obtener ayuda sobre los comandos de finn y sus cambios intente:

```sh
finn help
finn server --help
finn client --help
```

## Docker

```sh
docker build -t finn -f etc/Dockerfile .
```

Puede ubicar la caché de finn para que se ejecute dentro del contenedor

```sh
docker run -it -d -v $HOME/.finn:/root/.finn finn
```
## Compilación multiplataforma

Rust (cargo) puede compilar finn para muchas plataformas, así que en teoría ejecutar `finn` como un nodo de validación en un dispositivo de baja potencia podría ser posible. Para hacer una compilación cruzada `finn` en una plataforma x86 Linux y generar binarios de ARM, por ejemplo para Raspberry-pi.

## Usando finn

La página de la wiki [Cómo usar finn](https://github.com/mimblewimble/docs/wiki/How-to-use-finn) y las páginas de enlaces tienen más información sobre las características que disponemos, resolución de problemas, etc.

## Minando en finn

Tenga en cuenta que todas las funciones de minería de finn se han trasladado a un paquete independiente llamado [finn_minner](https://github.com/mimblewimble/finn-miner). Una vez que el nodo de finn esté listo y funcionando, puede empezar a minar compilando y ejecutando finn-miner con su nodo finn en funcionamiento.

# Playbook: translit release with standard game fonts

Короткий практический сценарий для случая, когда у вас уже есть готовый перевод в `_ru`-файлах, но итоговый мод должен использовать стандартные латинские шрифты игры. В этом варианте текст переводится в транслит, а `fonts_en.swf` и `fontconfig_en.txt` в релиз не включаются.

## Когда использовать

- есть готовый пакет `Data/Strings/*_ru.*` и `Data/Interface/translate_ru.txt`
- нужен вариант без кастомных кириллических шрифтов
- нужен воспроизводимый локальный выпуск вне репозитория

## Что получится

- `StarfieldRussian.esm`
- `StarfieldRussian - Main.ba2`
- `StarfieldRussian - Interface.ba2`
- `CREDITS.txt`

`Interface.ba2` в этом сценарии содержит только транслитерированный `translate_en.txt`.

## Быстрый smoke-run на свежем main

Если сначала нужно проверить, что текущий pipeline вообще работает на вашем наборе данных:

```bash
cd tools/ba2-packer
cargo run --release -- smoke-test \
  --input-dir /path/to/Data \
  --interface-dir ../../src/interface \
  --output-dir /tmp/starfield-smoke \
  --credit "Автор перевода"
```

Важно: сейчас `smoke-test` нужно запускать с явным `--interface-dir ../../src/interface`, если вы находитесь в `tools/ba2-packer/`. Дефолтный `src/interface` зависит от `cwd`.

## Канонический standard-font translit release

Ниже команды для сборки в отдельную временную директорию без изменений tracked-файлов в репозитории.

```bash
ROOT="$(mktemp -d /tmp/starfield-translit.XXXXXX)"

cd tools/ba2-packer
cargo build --release

cargo run --release -- rename \
  --input-dir /path/to/Data \
  --output-dir "$ROOT/renamed"

cargo run --release -- transliterate \
  --input-dir "$ROOT/renamed" \
  --output-dir "$ROOT/transliterated" \
  --credit "Автор перевода"

mkdir -p "$ROOT/stage/strings" "$ROOT/stage/interface" "$ROOT/dist"
cp "$ROOT"/transliterated/*.STRINGS "$ROOT/stage/strings/"
cp "$ROOT"/transliterated/*.DLSTRINGS "$ROOT/stage/strings/"
cp "$ROOT"/transliterated/*.ILSTRINGS "$ROOT/stage/strings/"
cp "$ROOT"/transliterated/translate_en.txt "$ROOT/stage/interface/"

cargo run --release -- create-esm \
  --output "$ROOT/dist/StarfieldRussian.esm"

cargo run --release -- pack \
  --input-strings "$ROOT/stage/strings" \
  --input-interface "$ROOT/stage/interface" \
  --output-dir "$ROOT/dist" \
  --credit "Автор перевода"
```

## Проверка результата

Артефакты будут лежать в `$ROOT/dist`.

Проверить содержимое:

```bash
find "$ROOT/dist" -maxdepth 1 -type f | sort
du -h "$ROOT/dist"/*
```

На проверенном прогоне с Nexus-пакетом `Manual install v0.42-233-0-42-1694191060` получился такой результат:

- `StarfieldRussian - Main.ba2`: `13M`
- `StarfieldRussian - Interface.ba2`: `192K`
- `StarfieldRussian.esm`: `76B`
- `CREDITS.txt`: присутствует

## Ограничение текущего validate

Если прогнать:

```bash
cargo run --release -- validate "$ROOT/dist" \
  --source-strings "$ROOT/stage/strings" \
  --source-interface "$ROOT/stage/interface" \
  --require-credits
```

то сейчас ожидаемо провалятся 2 проверки:

- `Interface file present: fontconfig_en.txt`
- `Interface file present: fonts_en.swf`

Это не ошибка сборки standard-font translit-варианта. Это текущее ограничение `validate`, который моделирует полный interface bundle.

## Перед публикацией

- проверьте, что `CREDITS.txt` содержит автора исходного перевода
- продублируйте авторство в описании релиза или страницы мода
- отдельно помните про PS5 caveat: preloading шрифтов через Creations остаётся экспериментальным сценарием

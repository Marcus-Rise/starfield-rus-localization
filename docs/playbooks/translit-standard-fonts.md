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

## Валидация

Используйте профиль `standard-font-translit`, чтобы пропустить проверки `fontconfig_en.txt` и `fonts_en.swf`:

```bash
cargo run --release -- validate "$ROOT/dist" \
  --source-strings "$ROOT/stage/strings" \
  --source-interface "$ROOT/stage/interface" \
  --require-credits \
  --profile standard-font-translit
```

Профиль `standard-font-translit` проверяет всё то же, что и `full`, но пропускает проверки шрифтовых файлов, которые намеренно отсутствуют в этом сценарии.

## Упаковка для релиза

Если нужен один архив для Nexus Mods или ручной раздачи, упакуйте содержимое `$ROOT/dist` в zip:

```bash
cd "$ROOT/dist"
zip -r ../StarfieldRussian-standard-font-translit.zip .
```

После этого архив будет лежать по пути `$ROOT/StarfieldRussian-standard-font-translit.zip`.

## Перед публикацией

- проверьте, что `CREDITS.txt` содержит автора исходного перевода
- продублируйте авторство в описании релиза или страницы мода
- отдельно помните про PS5 caveat: preloading шрифтов через Creations остаётся экспериментальным сценарием

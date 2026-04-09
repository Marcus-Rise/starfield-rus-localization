# Playbook: transliterate `Strings` only

Отдельный workflow для случая, когда не нужно собирать `BA2` и `ESM`, а нужно только быстро получить новую папку `Strings` с транслитом вместо кириллицы.

## Когда использовать

- на входе уже есть бинарные строковые таблицы `.STRINGS` / `.DLSTRINGS` / `.ILSTRINGS`
- нужно получить такой же набор файлов, но с латинской транслитерацией
- не хочется гонять полный pipeline сборки мода ради промежуточной проверки

## Что важно

- команда работает с **бинарными** string table файлами, не с JSONL
- если входные файлы ещё называются `*_ru.*`, сначала прогоните `rename`
- на выходе получится обычная папка `Strings`; просто укажите её как `--output-dir`
- `BA2`, `ESM` и `validate` в этом сценарии не участвуют

## Базовый сценарий: `Strings` -> `Strings`

```bash
ROOT="$(mktemp -d /tmp/starfield-strings-translit.XXXXXX)"
INPUT="/path/to/Data/Strings"
OUTPUT="$ROOT/Strings"

cd tools/ba2-packer
cargo build --release

mkdir -p "$OUTPUT"

cargo run --release -- transliterate \
  --input-dir "$INPUT" \
  --output-dir "$OUTPUT"
```

Результат:
- в `$OUTPUT` появятся `.STRINGS`, `.DLSTRINGS`, `.ILSTRINGS` с теми же именами файлов
- структура простая: входная папка `Strings`, выходная папка `Strings`

## Если на входе `_ru`, а нужен `_en`

```bash
ROOT="$(mktemp -d /tmp/starfield-strings-translit.XXXXXX)"

cd tools/ba2-packer
cargo build --release

cargo run --release -- rename \
  --input-dir /path/to/Data/Strings \
  --output-dir "$ROOT/renamed"

mkdir -p "$ROOT/Strings"

cargo run --release -- transliterate \
  --input-dir "$ROOT/renamed" \
  --output-dir "$ROOT/Strings"
```

Это даёт ровно тот промежуточный результат, который нужен для быстрой проверки текста без сборки мода.

## Быстрая проверка результата

Если нужно убедиться, что внутри файлов действительно транслит, извлеките их в JSONL и посмотрите несколько строк:

```bash
cargo run --release -- extract \
  --input "$ROOT/Strings" \
  --output-dir "$ROOT/jsonl"
```

Дальше можно открыть любой `*.jsonl` и проверить, что кириллица заменена на латиницу.

## Примечания

- `transliterate` принимает как саму папку `Strings`, так и корень `Data`, внутри которого есть `Strings`
- если `translate_en.txt` в этом сценарии не нужен, просто не кладите его во входную директорию
- если нужен и UI-текст в транслите, добавьте `translate_en.txt` во входную директорию и команда обработает его рядом со строковыми таблицами

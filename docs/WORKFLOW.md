# Workflow: сборка мода из файлов перевода

Пошаговая инструкция по сборке готового PS5-мода из файлов русской локализации.

## Предварительные требования

- Rust toolchain (`cargo`)
- Файлы перевода в формате `_ru` (собственный перевод или файлы, на использование которых есть разрешение автора):
  - 12 строковых таблиц в `Data/Strings/`
  - `fontconfig_ru.txt`, `fonts_ru.swf`, `translate_ru.txt` в `Data/Interface/`

## Шаг 1: Сборка CLI

```bash
cd tools/ba2-packer
cargo build --release
```

Бинарь: `target/release/ba2-packer`

## Шаг 2: Переименование `_ru` -> `_en`

PS5 загружает только файлы с суффиксом `_en`. Команда `rename` копирует файлы с переименованием и автоматически исправляет `fontlib "fonts_ru"` -> `fontlib "fonts_en"` внутри fontconfig.

```bash
ba2-packer rename \
  --input-dir /path/to/reference/Data \
  --output-dir ./build
```

## Шаг 3: Извлечение строковых таблиц для редактирования (опционально)

Если нужно отредактировать перевод, можно извлечь строковые таблицы в JSONL:

```bash
ba2-packer extract \
  --input build/starfield_en.STRINGS \
  --output-dir ./extracted
```

Каждая запись — отдельная строка JSON: `{"id":12345,"text":"Hello, world!"}`. Файл сохраняется с двойным расширением (например, `starfield_en.STRINGS.jsonl`), чтобы при обратной сборке определить тип таблицы.

Можно извлечь сразу всю директорию:
```bash
ba2-packer extract --input build/ --output-dir ./extracted
```

## Шаг 3a: Упаковка отредактированных JSONL обратно в бинарный формат

После редактирования JSONL-файлов — собрать обратно:

```bash
ba2-packer repack \
  --input ./extracted/starfield_en.STRINGS.jsonl \
  --output-dir build/
```

Или целую директорию:
```bash
ba2-packer repack --input ./extracted/ --output-dir build/
```

## Шаг 4: Размещение файлов в `src/`

```bash
# Строковые таблицы
cp build/*_en.STRINGS build/*_en.DLSTRINGS build/*_en.ILSTRINGS src/strings/

# Интерфейс (шрифты, fontconfig, UI переводы)
cp build/fonts_en.swf build/fontconfig_en.txt build/translate_en.txt src/interface/
```

## Шаг 5: Создание ESM-плагина

```bash
ba2-packer create-esm --output dist/StarfieldRussian.esm
```

Генерирует минимальный Starfield ESM с флагами ESM + Localized Strings, HEDR 0.96, master Starfield.esm.

## Шаг 6: Упаковка в BA2

```bash
ba2-packer pack \
  --input-strings src/strings \
  --input-interface src/interface \
  --output-dir dist
```

Создаёт:
- `dist/StarfieldRussian - Main.ba2` (строковые таблицы)
- `dist/StarfieldRussian - Interface.ba2` (шрифты, fontconfig, UI переводы)

## Шаг 7: Валидация

```bash
ba2-packer validate dist \
  --source-strings src/strings \
  --source-interface src/interface
```

Проверяет 13 пунктов: ESM флаги, строковые файлы, интерфейс, BA2 заголовки, размер < 2 GB.

## Шаг 8: Установка на PS5

Итоговые файлы для загрузки через Bethesda Creations:
- `StarfieldRussian.esm`
- `StarfieldRussian - Main.ba2`
- `StarfieldRussian - Interface.ba2`

Подробнее: [PUBLISH_CREATIONS.md](PUBLISH_CREATIONS.md)

## Замена шрифтов на свободные (для публичного релиза)

Исходный `fonts_ru.swf` может содержать проприетарные шрифты. Для публичного релиза нужно пересобрать SWF со свободными шрифтами (SIL OFL):

| Оригинальный шрифт | Свободная замена |
|---|---|
| RF_35_M (основной) | PT Sans Regular |
| RF_55_M (полужирный) | PT Sans Bold |
| RF_55_SB (grotesk bold) | PT Sans Bold |
| SnideHand (рукописный) | Caveat / Neucha |
| Starfield_Grotesk_R (консоль) | Noto Sans Mono |

Инструмент: [JPEXS FFDec](https://github.com/jindrapetrik/jpexs-decompiler)

1. Открыть `fonts_ru.swf` в JPEXS
2. Для каждого DefineFont — заменить шрифт на свободный аналог с кириллическими глифами (U+0400-U+04FF)
3. Экспортировать как `fonts_en.swf`
4. Если имена шрифтов изменились — обновить `fontconfig_en.txt` (маппинг `map "$MAIN_Font" = "NewFontName" Normal`)

# Workflow: сборка мода из файлов перевода

## Предварительные требования

- Rust toolchain (`cargo`)
- Файлы перевода (собственный перевод или файлы с разрешением автора)

```bash
cd tools/ba2-packer
cargo build --release
```

Бинарь: `target/release/ba2-packer`

---

## Сценарий 1: Есть файлы `_ru`

Самый частый путь. У вас есть файлы перевода с суффиксом `_ru`:
- 12 строковых таблиц (`starfield_ru.STRINGS`, `starfield_ru.DLSTRINGS`, ... )
- `fontconfig_ru.txt`, `fonts_ru.swf`, `translate_ru.txt`

### Результат

Три файла, готовые для PS5 (Bethesda Creations):
```
dist/StarfieldRussian.esm
dist/StarfieldRussian - Main.ba2
dist/StarfieldRussian - Interface.ba2
```

### Команды

```bash
# 1. Переименовать _ru → _en (PS5 загружает только _en)
ba2-packer rename \
  --input-dir /path/to/Data \
  --output-dir ./build

# 2. Разместить файлы в src/
cp build/*_en.STRINGS build/*_en.DLSTRINGS build/*_en.ILSTRINGS src/strings/
cp build/fonts_en.swf build/fontconfig_en.txt build/translate_en.txt src/interface/

# 3. Создать ESM-плагин
ba2-packer create-esm --output dist/StarfieldRussian.esm

# 4. Упаковать в BA2-архивы
ba2-packer pack \
  --input-strings src/strings \
  --input-interface src/interface \
  --output-dir dist

# 5. Проверить результат
ba2-packer validate dist \
  --source-strings src/strings \
  --source-interface src/interface
```

Готово. Далее: [Публикация в Creations](PUBLISH_CREATIONS.md) или [Публикация на Nexus](PUBLISH_NEXUS.md).

---

## Сценарий 2: Есть оригинальные файлы

У вас есть строковые таблицы из игры (`_en`) и вы хотите отредактировать перевод вручную.

### Команды

```bash
# 1. Извлечь строковые таблицы в JSONL для редактирования
ba2-packer extract --input /path/to/strings/ --output-dir ./extracted

# 2. Отредактировать JSONL-файлы
#    Каждая строка: {"id":12345,"text":"Hello, world!"}
#    Замените текст на русский перевод

# 3. Собрать JSONL обратно в бинарный формат
ba2-packer repack --input ./extracted/ --output-dir ./build

# 4. (Опционально) Транслитерация, если нет кириллических шрифтов
ba2-packer transliterate \
  --input-dir ./build \
  --output-dir ./build \
  --credit "Автор перевода"

# 5. Разместить файлы в src/
cp build/*_en.STRINGS build/*_en.DLSTRINGS build/*_en.ILSTRINGS src/strings/
cp build/fontconfig_en.txt build/translate_en.txt src/interface/
# Шрифт fonts_en.swf нужно подготовить отдельно — см. раздел «Замена шрифтов»

# 6. Создать ESM-плагин
ba2-packer create-esm --output dist/StarfieldRussian.esm

# 7. Упаковать в BA2-архивы
ba2-packer pack \
  --input-strings src/strings \
  --input-interface src/interface \
  --output-dir dist

# 8. Проверить результат
ba2-packer validate dist \
  --source-strings src/strings \
  --source-interface src/interface
```

Готово. Далее: [Публикация в Creations](PUBLISH_CREATIONS.md) или [Публикация на Nexus](PUBLISH_NEXUS.md).

---

## Сценарий 3: Валидация и упаковка

Файлы уже подготовлены и лежат в `src/`. Нужно только упаковать и проверить.

### Команды

```bash
# 1. Создать ESM-плагин
ba2-packer create-esm --output dist/StarfieldRussian.esm

# 2. Упаковать в BA2-архивы
ba2-packer pack \
  --input-strings src/strings \
  --input-interface src/interface \
  --output-dir dist

# 3. Проверить результат
ba2-packer validate dist \
  --source-strings src/strings \
  --source-interface src/interface
```

Готово. Далее: [Публикация в Creations](PUBLISH_CREATIONS.md) или [Публикация на Nexus](PUBLISH_NEXUS.md).

---

## Подробный справочник

Детальное описание каждого шага для всех команд.

### Переименование `_ru` → `_en`

PS5 загружает только файлы с суффиксом `_en`. Команда `rename` копирует файлы с переименованием и автоматически исправляет `fontlib "fonts_ru"` → `fontlib "fonts_en"` внутри fontconfig.

```bash
ba2-packer rename \
  --input-dir /path/to/reference/Data \
  --output-dir ./build
```

### Извлечение строковых таблиц

Извлекает бинарные STRINGS/DLSTRINGS/ILSTRINGS в JSONL для редактирования:

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

### Упаковка JSONL обратно в бинарный формат

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

### Транслитерация кириллицы в латиницу

Если невозможно использовать кириллицу (например, шрифты не поддерживают или нужен упрощённый мод без замены шрифтов), можно транслитерировать текст:

```bash
ba2-packer transliterate \
  --input-dir build/ \
  --output-dir build/ \
  --credit "ZoG Forum Team"
```

Транслитерирует кириллический текст в латиницу (Привет → Privet) в строковых таблицах и `translate_en.txt`. Флаг `--credit` создаёт файл `CREDITS.txt` с указанием автора оригинального перевода.

### Создание ESM-плагина

```bash
ba2-packer create-esm --output dist/StarfieldRussian.esm
```

Генерирует минимальный Starfield ESM с флагами ESM + Localized Strings, HEDR 0.96, master Starfield.esm.

### Упаковка в BA2

```bash
ba2-packer pack \
  --input-strings src/strings \
  --input-interface src/interface \
  --output-dir dist
```

Создаёт:
- `dist/StarfieldRussian - Main.ba2` (строковые таблицы)
- `dist/StarfieldRussian - Interface.ba2` (шрифты, fontconfig, UI переводы)

### Валидация

```bash
ba2-packer validate dist \
  --source-strings src/strings \
  --source-interface src/interface
```

Проверяет 13 пунктов: ESM флаги, строковые файлы, интерфейс, BA2 заголовки, размер < 2 GB.

### Установка на PS5

Итоговые файлы для загрузки через Bethesda Creations:
- `StarfieldRussian.esm`
- `StarfieldRussian - Main.ba2`
- `StarfieldRussian - Interface.ba2`

Подробнее: [PUBLISH_CREATIONS.md](PUBLISH_CREATIONS.md)

---

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

## Авторские права при использовании чужого перевода

При использовании перевода, созданного другими авторами (например, с Nexus Mods), необходимо соблюдать авторские права:

### Обязательные шаги

1. **Получите разрешение** автора перевода на использование и распространение
2. **Укажите авторство** при сборке мода:
   ```bash
   ba2-packer pack \
     --input-strings src/strings \
     --input-interface src/interface \
     --output-dir dist \
     --credit "Автор перевода (Nexus Mods)"
   ```
3. Файл `CREDITS.txt` будет создан автоматически в `dist/` рядом с BA2-архивами
4. При публикации на Nexus Mods или Creations — укажите автора оригинального перевода в описании мода

### Что нельзя

- Распространять чужой перевод без разрешения автора
- Размещать текст чужого перевода в этом репозитории
- Удалять или скрывать информацию об авторстве

### Шрифты

- Проприетарные шрифты (NB Architekt, NB Grotesk, Handwritten_Institute) **нельзя** включать в мод
- Используйте только свободные шрифты (SIL OFL): PT Sans, Noto Sans, Caveat и др.

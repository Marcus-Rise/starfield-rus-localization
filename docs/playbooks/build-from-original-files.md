# Playbook: build from original game files

Используйте этот сценарий, если у вас есть оригинальные `_en` файлы игры и вы хотите подготовить перевод вручную.

## Что нужно на входе

- исходные строковые таблицы игры (`*_en.STRINGS`, `*_en.DLSTRINGS`, `*_en.ILSTRINGS`)
- интерфейсные файлы игры (`fontconfig_en.txt`, `translate_en.txt`)
- отдельно подготовленный `fonts_en.swf`, если нужен кириллический шрифт

## Команды

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
cp /path/to/Data/Interface/fontconfig_en.txt src/interface/
cp /path/to/Data/Interface/translate_en.txt src/interface/
# Шрифт fonts_en.swf нужно подготовить отдельно

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

Далее: [Публикация в Creations](../PUBLISH_CREATIONS.md) или [Публикация на Nexus](../PUBLISH_NEXUS.md).

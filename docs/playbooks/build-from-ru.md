# Playbook: build from `_ru` files

Используйте этот сценарий, если у вас уже есть готовые файлы перевода с суффиксом `_ru`.

## Что нужно на входе

- 12 строковых таблиц (`starfield_ru.STRINGS`, `starfield_ru.DLSTRINGS`, ...)
- `fontconfig_ru.txt`
- `fonts_ru.swf`
- `translate_ru.txt`

## Что получится

- `dist/StarfieldRussian.esm`
- `dist/StarfieldRussian - Main.ba2`
- `dist/StarfieldRussian - Interface.ba2`

## Команды

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

Далее: [Публикация в Creations](../PUBLISH_CREATIONS.md) или [Публикация на Nexus](../PUBLISH_NEXUS.md).

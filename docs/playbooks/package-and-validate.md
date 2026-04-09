# Playbook: package and validate prepared sources

Используйте этот сценарий, если `src/strings` и `src/interface` уже подготовлены, и нужно только собрать артефакты и проверить результат.

## Что получится

- `dist/StarfieldRussian.esm`
- `dist/StarfieldRussian - Main.ba2`
- `dist/StarfieldRussian - Interface.ba2`

## Команды

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

Далее: [Публикация в Creations](../PUBLISH_CREATIONS.md) или [Публикация на Nexus](../PUBLISH_NEXUS.md).

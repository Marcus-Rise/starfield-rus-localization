# Как вносить вклад

## Общие правила

1. **Не добавляйте** проприетарный контент (шрифты, игровые ассеты)
2. **Не добавляйте** чужие переводы без разрешения автора
3. Код и переменные — на английском
4. Документация — на русском
5. Все изменения через Pull Request (не коммитить в main напрямую)

## Процесс

1. Форк репозитория
2. Создайте ветку: `git checkout -b feature/my-feature`
3. Внесите изменения
4. Убедитесь что тесты проходят:
   ```bash
   cd tools/ba2-packer
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
5. Для проверки пайплайна целиком используйте `smoke-test`:
   ```bash
   ba2-packer smoke-test \
     --input-dir /path/to/your/_ru/files \
     --interface-dir ../../src/interface
   ```
6. Создайте Pull Request

## Стандарты кода

- **Rust**: `cargo fmt` + `cargo clippy -D warnings`
- **TDD**: тесты пишутся до реализации
- Минимальные зависимости, без переусложнения

## Перевод

Если вы хотите добавить собственный перевод:

1. Подготовьте строковые таблицы (12 файлов: 4 плагина × 3 типа)
2. Убедитесь что файлы `_en` (не `_ru`) — PS5 загружает только `_en`
3. Используйте `ba2-packer rename` для конвертации `_ru` → `_en`
4. Для редактирования перевода: `ba2-packer extract` → правка JSONL → `ba2-packer repack`
5. Проверьте кодировку `translate_en.txt` — UTF-16LE с BOM
6. Создайте ESM-плагин: `ba2-packer create-esm --output dist/StarfieldRussian.esm`
7. Упакуйте архивы: `ba2-packer pack --input-strings ... --input-interface ... --output-dir dist`
8. Запустите `ba2-packer validate dist/ --source-strings src/strings --source-interface src/interface` для проверки (для translit-варианта добавьте `--profile standard-font-translit`)

### Авторские права

При использовании чужого перевода (например, с Nexus Mods):
- Получите разрешение автора на использование
- Укажите авторство: `ba2-packer pack ... --credit "Автор перевода"`
- Не размещайте текст чужого перевода в репозитории

## Шрифты

- Используйте только свободные шрифты (SIL OFL): PT Sans, Noto Sans и т.д.
- Создавайте SWF через JPEXS FFDec
- Не включайте проприетарные шрифты (NB Architekt, NB Grotesk)

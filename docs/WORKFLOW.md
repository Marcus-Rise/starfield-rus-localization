# Workflow: сборка мода из файлов перевода

`WORKFLOW.md` теперь играет роль карты сценариев. Подробные пошаговые команды вынесены в отдельные playbook-файлы, чтобы не дублировать их между `README`, operational runbook'ами и справочными документами.

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

Самый частый путь. У вас есть готовые файлы перевода с суффиксом `_ru`:
- строковые таблицы (`*_ru.STRINGS`, `*_ru.DLSTRINGS`, `*_ru.ILSTRINGS`)
- `translate_ru.txt`
- при полном кириллическом варианте также `fontconfig_ru.txt` и `fonts_ru.swf`

Что читать дальше:
- [docs/playbooks/build-from-ru.md](playbooks/build-from-ru.md) — канонический пошаговый сценарий
- [docs/playbooks/translit-standard-fonts.md](playbooks/translit-standard-fonts.md) — отдельный practical runbook для translit-варианта без кастомных кириллических шрифтов

---

## Сценарий 2: Есть оригинальные файлы

Используйте этот сценарий, если у вас есть оригинальные `_en` файлы игры и вы хотите подготовить перевод вручную через `extract` / `repack`.

Что читать дальше:
- [docs/playbooks/build-from-original-files.md](playbooks/build-from-original-files.md)

---

## Сценарий 3: Валидация и упаковка

Используйте этот сценарий, если `src/strings` и `src/interface` уже подготовлены, и нужно только собрать артефакты и проверить результат.

Что читать дальше:
- [docs/playbooks/package-and-validate.md](playbooks/package-and-validate.md)

---

## Быстрый smoke-тест

Если нужен быстрый локальный E2E check без ручного прогона всех шагов, используйте `smoke-test`. Команда выполняет: `rename -> transliterate -> create-esm -> pack -> validate`.

См. `ba2-packer smoke-test --help` и используйте этот путь как fast feedback, а не как замену отдельным playbook'ам.

---

## Замена шрифтов на свободные

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

- распространять чужой перевод без разрешения автора
- размещать текст чужого перевода в этом репозитории
- удалять или скрывать информацию об авторстве

### Шрифты

- проприетарные шрифты (NB Architekt, NB Grotesk, Handwritten_Institute) **нельзя** включать в мод
- используйте только свободные шрифты (SIL OFL): PT Sans, Noto Sans, Caveat и др.

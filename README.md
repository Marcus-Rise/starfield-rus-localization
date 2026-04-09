# Starfield — Русская локализация (PS5)

## Что это

Rust-инструментарий для сборки и валидации мода русской локализации Starfield из пользовательских ресурсов. Не содержит текстов перевода и проприетарных шрифтов.

## Что это НЕ

- Готовый перевод — текст пользователь добавляет самостоятельно
- Шрифтовой пакет — проприетарные шрифты запрещены, используются свободные (SIL OFL)
- Скачиваемый мод — здесь только инструменты сборки

## Если вы новичок

Если нужно быстро проверить, что весь pipeline вообще работает, начните со `smoke-test`. Остальные сценарии вынесены в отдельные документы, чтобы `README` оставался короткой точкой входа.

### Рекомендуемый первый запуск

Это самый частый путь: у вас уже есть перевод и интерфейсные файлы (`*_ru.STRINGS`, `translate_ru.txt`, возможно `fonts_ru.swf` и `fontconfig_ru.txt`).

```bash
cd tools/ba2-packer
cargo build --release

./target/release/ba2-packer smoke-test \
  --input-dir /path/to/Data \
  --output-dir /tmp/starfield-smoke \
  --credit "Автор перевода"
```

Что делает `smoke-test`:
- переименовывает `_ru` → `_en`
- транслитерирует кириллицу в латиницу
- создаёт `StarfieldRussian.esm`
- собирает `Main.ba2` и `Interface.ba2`
- запускает `validate`

Дальше выберите нужный сценарий:
- [docs/playbooks/build-from-ru.md](docs/playbooks/build-from-ru.md) — если у вас есть готовые файлы `_ru`
- [docs/playbooks/build-from-original-files.md](docs/playbooks/build-from-original-files.md) — если нужно извлечь и отредактировать перевод
- [docs/playbooks/package-and-validate.md](docs/playbooks/package-and-validate.md) — если `src/strings` и `src/interface` уже подготовлены
- [docs/playbooks/transliterate-strings-only.md](docs/playbooks/transliterate-strings-only.md) — если нужно только быстро получить папку `Strings` с транслитом
- [docs/playbooks/translit-standard-fonts.md](docs/playbooks/translit-standard-fonts.md) — если нужен practical runbook для translit-варианта со стандартными игровыми шрифтами

## Быстрый выбор команды

| Если нужно | Команда |
|--------|---------|
| Быстро прогнать весь pipeline на наборе `_ru` | `smoke-test` |
| Переименовать `_ru` в `_en` | `rename` |
| Транслитерировать кириллицу в латиницу | `transliterate` |
| Получить только новую папку `Strings` с транслитом | `transliterate` + [playbook](docs/playbooks/transliterate-strings-only.md) |
| Извлечь строки для ручного редактирования | `extract` |
| Собрать строки обратно из JSONL | `repack` |
| Создать ESM-плагин | `create-esm` |
| Упаковать BA2-архивы | `pack` |
| Проверить готовность артефактов | `validate` |

## Куда смотреть дальше

- [docs/WORKFLOW.md](docs/WORKFLOW.md) — карта сценариев и общие заметки
- [docs/playbooks/transliterate-strings-only.md](docs/playbooks/transliterate-strings-only.md) — отдельный workflow `Strings -> Strings`
- [docs/playbooks/translit-standard-fonts.md](docs/playbooks/translit-standard-fonts.md) — короткий practical playbook для translit-варианта со стандартными игровыми шрифтами
- [docs/PUBLISH_CREATIONS.md](docs/PUBLISH_CREATIONS.md) — публикация в Bethesda Creations
- [docs/PUBLISH_NEXUS.md](docs/PUBLISH_NEXUS.md) — публикация на Nexus Mods

## Отказ от ответственности

> **Это некоммерческий проект с открытым исходным кодом.**
>
> - Мы **не нарушаем** авторские права Bethesda Softworks, ZeniMax Media или других правообладателей
> - Репозиторий **не содержит** проприетарный контент (шрифты, игровые ассеты, тексты чужих переводов)
> - Здесь только **инструменты сборки**, шаблоны формата и CI/CD инфраструктура
> - Пользователь самостоятельно добавляет свои ресурсы перевода
> - Авторы проекта **не несут ответственности** за использование данных инструментов
> - Проект распространяется по лицензии MIT — «как есть», без гарантий

## Контекст задачи

Starfield на PS5 не поддерживает русский язык. Движок Scaleform GFx использует шрифтовые библиотеки SWF, содержащие только латинские глифы. При отображении кириллицы вместо букв появляются пустые прямоугольники.

Этот проект предоставляет инструменты для:
1. **Замены шрифтов** — SWF-файл с кириллическими глифами
2. **Замены строковых таблиц** — файлы `.STRINGS` / `.DLSTRINGS` / `.ILSTRINGS` с русским переводом
3. **Замены UI-переводов** — файл `translate_en.txt` с русскими строками интерфейса
4. **Сборки мода** — ESM-плагин + BA2-архивы, совместимые с Bethesda Creations

## Установка

### PS5 (через Bethesda Creations) — экспериментально
1. В главном меню Starfield → **Creations**
2. Найдите мод «Starfield Russian Translation»
3. Скачайте и активируйте

> ⚠️ Загрузка кириллических шрифтов на PS5 не проверена. Подробности в `docs/ARCHITECTURE.md`.
>
> Практический fallback — translit-вариант без кириллических шрифтов.

### PC (Nexus Mods)
1. Скачайте zip с [GitHub Releases](../../releases/latest)
2. Распакуйте содержимое в `Starfield/Data/`
3. Запустите игру

### Xbox
Аналогично PS5 через Bethesda Creations.

## Сборка из исходников

### Требования
- Rust (stable)
- .NET 8 SDK (для Spriggit)
- Git LFS

### Минимальная сборка CLI

```bash
git clone --recurse-submodules <repo-url>
cd starfield-rus-localization

cd tools/ba2-packer
cargo build --release
```

Бинарь: `target/release/ba2-packer`

### Полная сборка из репозиторных исходников

```bash
cd tools/ba2-packer

# Сборка ESM (требуются файлы плагина в src/plugin/)
dotnet tool install -g Spriggit.CLI
spriggit deserialize \
  --InputPath ../../src/plugin/StarfieldRussian \
  --OutputPath ../../dist/StarfieldRussian.esm \
  --GameRelease Starfield

# Упаковка BA2 (требуются файлы перевода в src/strings/ и src/interface/)
cargo run --release -- pack \
  --input-strings ../../src/strings \
  --input-interface ../../src/interface \
  --output-dir ../../dist

cargo run --release -- validate ../../dist --source-strings ../../src/strings --source-interface ../../src/interface
```

## CLI инструмент (ba2-packer)

Единый Rust CLI с восемью подкомандами:

| Команда | Описание |
|---------|----------|
| `pack` | Упаковка файлов перевода в BA2-архивы |
| `validate` | Валидация собранного мода (поддерживает профили `full` и `standard-font-translit`) |
| `rename` | Переименование файлов `_ru` → `_en` |
| `extract` | Извлечение строковых таблиц в JSONL для редактирования |
| `repack` | Сборка JSONL обратно в бинарные строковые таблицы |
| `create-esm` | Генерация минимального ESM-плагина |
| `transliterate` | Транслитерация кириллицы в латиницу |
| `smoke-test` | Локальный E2E smoke-тест: rename → transliterate → pack → validate |

```bash
ba2-packer pack --input-strings <DIR> --input-interface <DIR> --output-dir <DIR>
ba2-packer validate <DIST_DIR> [--source-strings <DIR>] [--source-interface <DIR>] [--profile full|standard-font-translit]
ba2-packer rename --input-dir <DIR> --output-dir <DIR>
ba2-packer extract --input <FILE_OR_DIR> --output-dir <DIR>
ba2-packer repack --input <FILE_OR_DIR> --output-dir <DIR>
ba2-packer create-esm --output <PATH>
ba2-packer transliterate --input-dir <DIR> --output-dir <DIR> [--credit <AUTHOR>]
ba2-packer smoke-test --input-dir <DIR> [--output-dir <DIR>] [--interface-dir <DIR>] [--credit <AUTHOR>]
```

## Структура проекта

```
src/strings/          # Строковые таблицы (12 файлов, Git LFS)
src/interface/        # Шрифт, fontconfig, translate (Git LFS)
src/plugin/           # Spriggit ESM плагин
tools/ba2-packer/     # Rust CLI инструмент
docs/                 # Документация
.github/workflows/    # CI/CD
```

## Вклад

См. [CONTRIBUTING.md](CONTRIBUTING.md).

## Лицензия

MIT — см. [LICENSE](LICENSE).

# Starfield — Русская локализация (PS5)

## Что это

Rust-инструментарий для сборки и валидации мода русской локализации Starfield из пользовательских ресурсов. Не содержит текстов перевода и проприетарных шрифтов.

## Что это НЕ

- Готовый перевод — текст пользователь добавляет самостоятельно
- Шрифтовой пакет — проприетарные шрифты запрещены, используются свободные (SIL OFL)
- Скачиваемый мод — здесь только инструменты сборки

## Для кого

- **У вас есть файлы `_ru`** (свой перевод или файлы с разрешением автора) — соберите мод за 4 команды
- **У вас есть оригинальные файлы игры** — извлеките строки, отредактируйте, соберите мод
- **Нужно только проверить/упаковать** готовый набор файлов — validate + pack

## Быстрый старт

Выберите свой сценарий:

1. [У меня есть файлы `_ru`](docs/WORKFLOW.md#сценарий-1-есть-файлы-_ru) — самый частый путь
2. [У меня есть оригинальные файлы и нужно подготовить ресурсы](docs/WORKFLOW.md#сценарий-2-есть-оригинальные-файлы)
3. [Хочу только проверить/упаковать готовый набор](docs/WORKFLOW.md#сценарий-3-валидация-и-упаковка)

### Какую команду использовать?

| Задача | Команда |
|--------|---------|
| Есть файлы `_ru` — переименовать в `_en` | `rename` |
| Нужно отредактировать строки перевода | `extract` / `repack` |
| Нельзя использовать кириллицу (нет шрифтов) | `transliterate` |
| Создать ESM-плагин | `create-esm` |
| Упаковать в BA2-архивы | `pack` |
| Проверить готовность мода | `validate` |

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

### Шаги

```bash
git clone --recurse-submodules <repo-url>
cd starfield-rus-localization

# Сборка CLI инструмента
cd tools/ba2-packer
cargo build --release

# Сборка ESM (требуются файлы плагина в src/plugin/)
dotnet tool install -g Spriggit.CLI
spriggit deserialize \
  --InputPath src/plugin/StarfieldRussian \
  --OutputPath dist/StarfieldRussian.esm \
  --GameRelease Starfield

# Упаковка BA2 (требуются файлы перевода в src/strings/ и src/interface/)
cargo run --release -- pack \
  --input-strings ../../src/strings \
  --input-interface ../../src/interface \
  --output-dir ../../dist

# Валидация
cargo run --release -- validate ../../dist --source-strings ../../src/strings --source-interface ../../src/interface
```

## CLI инструмент (ba2-packer)

Единый Rust CLI с семью подкомандами:

| Команда | Описание |
|---------|----------|
| `pack` | Упаковка файлов перевода в BA2-архивы |
| `validate` | Валидация собранного мода (13 проверок) |
| `rename` | Переименование файлов `_ru` → `_en` |
| `extract` | Извлечение строковых таблиц в JSONL для редактирования |
| `repack` | Сборка JSONL обратно в бинарные строковые таблицы |
| `create-esm` | Генерация минимального ESM-плагина |
| `transliterate` | Транслитерация кириллицы в латиницу |

```bash
ba2-packer pack --input-strings <DIR> --input-interface <DIR> --output-dir <DIR>
ba2-packer validate <DIST_DIR> [--source-strings <DIR>] [--source-interface <DIR>]
ba2-packer rename --input-dir <DIR> --output-dir <DIR>
ba2-packer extract --input <FILE_OR_DIR> --output-dir <DIR>
ba2-packer repack --input <FILE_OR_DIR> --output-dir <DIR>
ba2-packer create-esm --output <PATH>
ba2-packer transliterate --input-dir <DIR> --output-dir <DIR> [--credit <AUTHOR>]
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

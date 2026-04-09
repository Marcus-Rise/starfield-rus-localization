# Техническая архитектура

## Обзор

Мод русской локализации Starfield для PS5 состоит из:
- **ESM-плагин** (`StarfieldRussian.esm`) — минимальный Small Master с флагами ESM + Localized Strings
- **BA2-архив строк** (`StarfieldRussian - Main.ba2`) — 12 файлов строковых таблиц
- **BA2-архив интерфейса** (`StarfieldRussian - Interface.ba2`) — шрифты, fontconfig, UI переводы

## Строковые таблицы

- Формат: бинарный, [документация UESP](https://en.uesp.net/wiki/Skyrim_Mod:String_Table_File_Format)
- `.STRINGS` — null-terminated UTF-8 (имена)
- `.DLSTRINGS` — uint32 length-prefixed UTF-8 (книги, заметки)
- `.ILSTRINGS` — uint32 length-prefixed UTF-8 (диалоги/субтитры)
- Структура: 8-byte header (uint32 count + uint32 data_size) → directory (id + offset) → string data
- **Файлы именуются `_en`** (не `_ru`) — PS5 не позволяет переключить `sLanguage`, всегда загружает `_en`

### Список файлов (4 плагина × 3 типа = 12)

| Плагин | STRINGS | DLSTRINGS | ILSTRINGS |
|--------|---------|-----------|-----------|
| starfield | starfield_en.STRINGS | starfield_en.DLSTRINGS | starfield_en.ILSTRINGS |
| blueprintships-starfield | blueprintships-starfield_en.STRINGS | blueprintships-starfield_en.DLSTRINGS | blueprintships-starfield_en.ILSTRINGS |
| constellation | constellation_en.STRINGS | constellation_en.DLSTRINGS | constellation_en.ILSTRINGS |
| oldmars | oldmars_en.STRINGS | oldmars_en.DLSTRINGS | oldmars_en.ILSTRINGS |

## UI Translation File

- Путь: `Data/Interface/translate_en.txt`
- Кодировка: **UTF-16LE с BOM** (0xFF 0xFE)
- Формат: `$KEY\tValue\n` (tab-separated, одна пара на строку)

## Шрифтовая система

- Движок: Autodesk Scaleform GFx (Flash/SWF UI)
- Библиотека шрифтов: `fonts_en.swf` с векторными глифами
- Ванильные шрифты: NB Architekt Light, NB Grotesk, Handwritten_Institute — только латиница
- Кириллический диапазон: U+0400–U+04FF (94+ символов)
- Инструмент: JPEXS FFDec

### Формат fontconfig_en.txt

```
fontlib "fonts_en"
map "$MAIN_Font" = "FontName" Normal
map "$HandwrittenFont" = "FontName" Normal
map "$Controller_Buttons" = "Controller Buttons" Normal
validNameChars "...АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя..."
validBookChars "...АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя..."
```

## ESM Plugin

- Формат: Bethesda plugin (TES4 header record)
- Флаги: ESM (0x01) + Localized Strings (0x80)
- HEDR version: 0.96 (Starfield)
- Internal version: 0x022F (559)
- Master: `Starfield.esm`
- Тип: Small Master

## BA2 Archives

- Формат: Bethesda Archive v2/v3
- Тип: **General** (GNRL), не DDS/Texture
- Naming: `ModName - Main.ba2` (строки), `ModName - Interface.ba2` (UI/шрифты)
- Инструмент: `ba2` crate (bsa-rs)
- Параметры: version=v2, format=GNRL, compression=Zip, strings=true

## Маппинг файлов (reference _ru → production _en)

| Reference (\_ru) | Production (\_en) | Архив |
|---|---|---|
| `starfield_ru.*` | `starfield_en.*` | Main.ba2 |
| `blueprintships-starfield_ru.*` | `blueprintships-starfield_en.*` | Main.ba2 |
| `constellation_ru.*` | `constellation_en.*` | Main.ba2 |
| `oldmars_ru.*` | `oldmars_en.*` | Main.ba2 |
| `fonts_ru.swf` | `fonts_en.swf` | Interface.ba2 |
| `fontconfig_ru.txt` | `fontconfig_en.txt` | Interface.ba2 |
| `translate_ru.txt` | `translate_en.txt` | Interface.ba2 |

## Валидация (13 проверок)

1. ESM: ESM flag set
2. ESM: Localized Strings flag (0x80)
3. ESM: HEDR version = 0.96
4. ESM: references Starfield.esm as master
5. Все 12 строковых файлов присутствуют
6. Строковые файлы парсятся (header count = directory entries)
7. translate_en.txt: UTF-16LE с BOM
8. translate_en.txt: формат $KEY\tValue
9. fontconfig_en.txt: fontlib "fonts_en"
10. fontconfig_en.txt: кириллица в validNameChars
11. fonts_en.swf: валидный SWF (FWS/CWS/ZWS)
12. BA2: валидный Starfield формат (BTDX v2/v3)
13. Общий размер < 2 GB

## Инструменты и ссылки

- [JPEXS FFDec](https://github.com/jindrapetrik/jpexs-decompiler) — редактор SWF
- [Spriggit](https://github.com/Mutagen-Modding/Spriggit) — сериализация ESM (.NET)
- [bsa-rs](https://github.com/Ryan-rsm-McKenzie/bsa-rs) — BA2 packing (Rust)
- [UESP String Tables](https://en.uesp.net/wiki/Skyrim_Mod:String_Table_File_Format)
- [UESP TES4 Header](https://en.uesp.net/wiki/Skyrim_Mod:Mod_File_Format/TES4)
- [Scaleform Fonts Guide](https://stepmodifications.org/wiki/Guide:UnderstandingFonts)

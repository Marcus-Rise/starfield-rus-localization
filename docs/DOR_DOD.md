# Definition of Ready / Definition of Done

## Итерация 1: Общий механизм (tooling)

### DoD (текущая итерация)

- [x] Rust CLI (`ba2-packer`) компилируется и проходит линтинг
- [x] `cargo fmt --check` + `cargo clippy -- -D warnings` — чисто
- [x] `cargo test` — все тесты проходят (unit + integration)
- [x] Подкоманды: `pack`, `validate`, `rename`, `extract`, `repack`, `create-esm`, `transliterate`, `smoke-test`
- [x] Структура директорий соответствует спецификации
- [ ] CI/CD build workflow проходит (зелёная галочка)
- [ ] Artifact `starfield-russian-mod` загружен в GitHub Actions

---

## Итерация 2: Перевод

### DoR (Definition of Ready)

- [ ] Файлы строковых таблиц (12 шт: 4 plugins × 3 types) в формате `.STRINGS` / `.DLSTRINGS` / `.ILSTRINGS`
- [ ] Источник перевода определён (собственный перевод / лицензия от автора)
- [ ] `fonts_en.swf` с кириллическими глифами — создан через JPEXS FFDec со свободными шрифтами (SIL OFL)
- [ ] `translate_en.txt` — заполнен реальными UI переводами (UTF-16LE BOM)
- [ ] `fontconfig_en.txt` — настроен с реальными именами шрифтов
- [ ] Spriggit ESM plugin — сериализован в `src/plugin/StarfieldRussian/`

### DoD (Definition of Done)

- [ ] Все 12 строковых файлов в `src/strings/`, проходят validate
- [ ] `fonts_en.swf` в `src/interface/`, валидный SWF
- [ ] `translate_en.txt` заполнен, UTF-16LE BOM, формат `$KEY\tValue`
- [ ] ESM плагин: ESM flag + Localized Strings (0x80), HEDR 0.96, master `Starfield.esm`
- [ ] `ba2-packer validate dist/ --source-strings src/strings --source-interface src/interface` — все 13 проверок + 1 предупреждение (font preloading) проходят
- [ ] Release tag → GitHub Release с zip
- [ ] Тестирование на PS5

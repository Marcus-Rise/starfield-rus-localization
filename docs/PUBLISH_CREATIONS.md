# Публикация в Bethesda Creations

> **Экспериментальный статус (PS5)**: загрузка кириллических шрифтов
> на PS5 через Creations **не проверена**. Если Creations не регистрирует
> `Interface.ba2` в `sResourceStartUpArchiveList`, кириллица не отрисуется.
> Ручная правка INI на PS5 вызывает зависания, логируемые Sony
> (риск бана по железу). См. `ARCHITECTURE.md` §Предзагрузка шрифтов.

Руководство для со-автора по загрузке мода в Bethesda Creations (PS5, Xbox, PC).

> **Агент-проводник**: `.claude/agents/publish-to-creations.md` — пошаговый чеклист для публикации.

## Требования

- Windows PC
- Starfield (Steam, ~125 ГБ)
- Creation Kit 2 (бесплатно в Steam)
- Аккаунт Bethesda.net

## Первая публикация

### 1. Скачайте артефакты

Скачайте zip из [GitHub Releases](../../releases/latest):
```
StarfieldRussian-vX.Y.Z.zip
├── StarfieldRussian.esm
├── StarfieldRussian - Main.ba2
└── StarfieldRussian - Interface.ba2
```

### 2. Разместите файлы

Распакуйте содержимое в директорию `Data/` игры:
```
Starfield/Data/
├── StarfieldRussian.esm
├── StarfieldRussian - Main.ba2
└── StarfieldRussian - Interface.ba2
```

### 3. Откройте в Creation Kit 2

1. Запустите Creation Kit 2
2. **File → Data**
3. Выберите `StarfieldRussian.esm`
4. **Set as Active File → OK**

### Авторство перевода

Если используется чужой перевод:
- [ ] Получено разрешение автора на переупаковку
- [ ] При сборке указан `--credit "Имя автора"`
- [ ] В `dist/CREDITS.txt` корректно указан автор
- [ ] В описании мода на площадке указан автор перевода

### 4. Загрузите в Creations

1. **File → Upload to Creations**
2. Заполните:
   - **Название**: Starfield — Русская локализация
   - **Описание**: Полная русская локализация интерфейса и текстов
   - **Теги**: Localization, Small Master
   - **Обложка и скриншоты**: приложите
3. **Submit**

Мод станет доступен на PS5, Xbox и PC.

## Обновление

1. Скачайте новый zip из GitHub Releases
2. Замените файлы в `Starfield/Data/`
3. Откройте в CK2 → Upload update

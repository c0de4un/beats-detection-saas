# 🎵 Korgi.Beats OSS

**High-performance audio beat detection API and Web UI for automated Reels/TikTok editing. Built with Rust, Axum, and Vue 3.**

[![Rust](https://img.shields.io/badge/Rust-1.97+-orange?logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-Latest-blue?logo=rust)](https://github.com/tokio-rs/axum)
[![Vue 3](https://img.shields.io/badge/Vue_3-TypeScript-green?logo=vue.js)](https://vuejs.org/)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue?logo=docker)](https://www.docker.com/)
[![SQLite](https://img.shields.io/badge/SQLite-WAL-blue)](https://www.sqlite.org/index.html)
[![License](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)

---

## 🚀 О проекте
Open-source сервис для автоматического обнаружения битов (Beat Detection) и анализа ритма аудиодорожек.
Проект решает главную боль видеомонтажеров и рилс-мейкеров: ручной поиск пиков громкости и смены ритма на таймлайне. Система анализирует трек и возвращает точные таймкоды битов (Onsets), которые можно использовать для автоматической склейки видео под музыку в CapCut, Premiere Pro или через кастомные API-скрипты.

> 💡 **Бизнес-кейс:** Проект создан для автоматизации монтажа лидогенерирующих роликов (формат "Хук → Эмоция → Продукт → CTA") под динамичную музыку, что кратно ускоряет производство контента для e-commerce и SMM.

## ✨ Ключевые возможности

- ⚡ **Blazing Fast:** Написан на Rust. Обработка трека занимает миллисекунды, без задержек Garbage Collector.
- 🎵 **Точный анализ:** Использование DSP-алгоритмов (Energy-based detection, Spectral Flux) для поиска Onsets и расчета BPM.
- 🖥️ **Интерактивный UI:** Визуализация аудио-волны (waveform) с маркерами битов на Vue 3. Возможность ручного редактирования (Human-in-the-loop).
- 📤 **Экспорт в NLE:** Скачивание результатов в JSON, SRT, CSV, а также Premiere Pro EDL / Final Cut XML.
- 🔌 **RESTful API:** Готовый бэкенд для интеграции в сторонние видеоредакторы и AI-пайплайны.

## 🛠 Технологический стек

| Компонент | Технология | Назначение |
| :--- | :--- | :--- |
| **Backend** | Rust, Axum | Асинхронный HTTP-сервер, DSP-обработка аудио |
| **Frontend** | Vue 3, TypeScript | Адаптивный SPA-интерфейс, Canvas-рендеринг волны |
| **Database** | SQLite (WAL mode) | Хранение пользователей, истории анализов, кэш |
| **Infrastructure**| Docker, Docker Compose | Контейнеризация и быстрое развертывание |

## 💻 Требования
Благодаря оптимизации на Rust, сервису не требуется GPU или мощные серверы.

### Аппаратные требования (Hardware)
* **RAM:** 4 GB (рекомендуется)
* **CPU:** 2 ядра (x86_64 или ARM)
* **Disk:** 500 MB свободного места

### Программные требования (Software)
**Для запуска через Docker (Рекомендуется):**
* Docker Engine 20.10+
* Docker Compose v2+

**Для локальной разработки (Local Dev):**
* Rust `1.97+` (rustc, cargo)
* Node.js `18+` (для сборки Vue 3 фронтенда)
* FFmpeg (опционально, для извлечения аудио из видеофайлов)

## 🏁 Быстрый старт (Quick Start)

### Вариант 1: Docker (Самый простой)
Клонируйте репозиторий и запустите одной командой:

```bash
git clone https://github.com/yourusername/beats-detection-saas.git
cd beats-detection-saas
docker-compose up -d
```

### Вариант 2: Локальная сборка (Local Build)
1. Запуск Backend (Rust):
2. Запуск Frontend (Vue 3):

## 🔌 API Документация (Кратко)
Сервис предоставляет REST API для интеграции.
GET /api/health — Проверка статуса сервиса.
POST /api/v1/auth/register — Регистрация нового пользователя.
POST /api/v1/auth/login — Получение JWT токена.
POST /api/v1/analyze — Загрузка аудио/видео файла и получение JSON с таймкодами битов.
GET /api/v1/history — Получение истории анализов текущего пользователя.
(Полная документация API доступна по адресу /api/docs после запуска Swagger UI)

## 🌌 Экосистема Korgi
Этот проект является частью open-source экосистемы Korgi.tech

## 🤝 Contributing
Pull Requests приветствуются! Если вы нашли баг или хотите добавить новый алгоритм детекции битов — создавайте Issue или Fork.

## 📄 License
Этот проект распространяется под лицензией MIT. См. файл LICENSE для подробностей.
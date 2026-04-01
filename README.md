# IMDb 2000 Recommender

🎓 **Bachelor's Capstone Project** – A desktop application that recommends movies/shows from the IMDb Top 2000 list using a hybrid recommendation engine powered by **DeepSeek** and **RoBERTa**.

![Rust](https://img.shields.io/badge/Rust-95.3%25-orange)
![CSS](https://img.shields.io/badge/CSS-4.7%25-blue)
![License](https://img.shields.io/badge/License-Apache%202.0-green)

---

## 📖 Overview

This project was developed as my bachelor's computer science capstone. It combines state-of-the-art language models to generate personalized recommendations from the IMDb 2000 dataset. The application features a clean, responsive UI built with **Dioxus** (Rust) and uses a two‑step recommendation pipeline:

- **RoBERTa** – extracts semantic embeddings from movie descriptions and user preferences.
- **DeepSeek** – performs reasoning and final ranking to suggest the most relevant titles.

The result is a desktop app that helps users discover hidden gems from the IMDb Top 2000.

---

## ✨ Features

- **Hybrid Recommendations** – Combines content‑based filtering (RoBERTa embeddings) with reasoning capabilities (DeepSeek).
- **Modern GUI** – Built with Dioxus, providing a native look and feel on Windows, and Linux.
- **IMDb 2000 Dataset** – Includes metadata, genres, ratings, and plot summaries for the top 2000 movies.
- **Cross‑Platform** – Ready‑to‑use binaries for Windows, plus Flatpak support for Linux.


## 🧰 Tech Stack

| Component       | Technology                          |
|-----------------|-------------------------------------|
| Language        | Rust                                |
| GUI Framework   | Dioxus                              |
| ML Models       | RoBERTa (via `bert-burn`), DeepSeek |
---

## 🚀 Running:

#### Linux

- download the linux zip from the releases page
- flatpak install --bundle com.example.imdb2000_recommender.flatpak
- flatpak run com.example.imdb2000_recommender

#### Windows

- download the windows zip from the releases page
- run `imdb2000_recommender.exe`

---


## 🛠️ Build from source:

- install the latest dioxus cli 
`cargo install --git https://github.com/DioxusLabs/dioxus dioxus-cli --locked`
- dx serve --desktop


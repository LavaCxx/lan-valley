# LanValley 🌾

A terminal-based farming game built with Rust and ratatui.

## Features

- 🌱 **Farming**: Till soil, water crops, plant seeds, and harvest
- 🌾 **9 Crop Types**: Potato, Melon, Pumpkin, Yam, Blueberry, Hot Pepper, Cranberry, Parsnip, Green Bean
- 🏪 **Shop System**: Buy seeds with gold
- 🎒 **Inventory**: Manage your crops, seeds, and dishes
- 🏗️ **Buildings**:
  - 💦 Sprinkler - Auto-waters nearby crops
  - 🏠 Junimo Hut - Auto-harvests crops
  - 🫙 Jam Maker - Produces jelly from crops
  - 📦 Shipping Box - Quick sell crops
  - 🍳 Kitchen - Cook dishes
- 🍳 **Cooking**: 9 dishes to cook with various ingredients
- 🌦️ **Weather System**: Sunny, Cloudy, Rainy, Stormy
- 🗺️ **IP Biome System**: Different biomes based on your IP
- 💾 **Auto-save**: Progress saved automatically

## Installation

```bash
git clone https://github.com/LavaCxx/lan-valley.git
cd lan-valley
cargo run --release
```

## Controls

| Key | Action |
|-----|--------|
| Arrow Keys | Move cursor / Navigate menu |
| T | Till soil |
| W | Water |
| P | Plant |
| H | Harvest |
| S | Open shop |
| I | Open inventory |
| B | Open building menu |
| C | Open cooking menu |
| Enter | Confirm selection |
| ESC | Close menu |
| Q | Quit game |

## Requirements

- Rust 1.70+
- Terminal with Unicode support

## License

MIT

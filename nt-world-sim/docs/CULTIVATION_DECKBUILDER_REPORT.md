# Cultivation Deckbuilder Demo Report

## Overview
Minimal but complete playable card game demo built as a single HTML file. No external dependencies, works in any browser.

## File
- **Path**: `nt-world-sim/dist/cultivation_deckbuilder.html`
- **Lines**: 435 (under 1000 limit)
- **Size**: ~15KB
- **Encoding**: ASCII-safe

## Features Implemented

### Game Screens (6)
| Screen | Status | Description |
|--------|--------|-------------|
| Main Menu | Implemented | Title, subtitle, start button |
| Map Screen | Implemented | Procedural node-based map, 3 floors, 5-7 nodes per floor |
| Combat Screen | Implemented | Full turn-based card combat |
| Shop Screen | Implemented | Buy cards, potions, relics with gold |
| Rest Screen | Implemented | Heal or upgrade cards |
| Game Over | Implemented | Shows floor, score, kills |

### Combat System
- 3 energy per turn (configurable via relics)
- Draw 5 cards per turn
- End turn button + keyboard shortcut (E)
- Player block resets each turn
- Enemy block resets each turn
- Combat log with scrolling history

### Cards (10 Starter Cards)
| Card | Cost | Effect |
|------|------|--------|
| Strike | 1 | Deal 6 damage |
| Defend | 1 | Gain 5 block |
| Bash | 2 | Deal 8 damage, apply 2 Vulnerable |
| Cleave | 1 | Deal 5 damage to all enemies |
| Iron Wave | 1 | Deal 5 damage, gain 5 block |
| Shrug It Off | 1 | Gain 8 block |
| Pommel Strike | 1 | Deal 9 damage, draw 1 |
| Twin Strike | 1 | Deal 5 damage twice |
| Rampage | 1 | Deal 8 damage, +5 each play |
| Pommel Strike+ | 1 | Deal 11 damage, draw 1 |

### Enemies (3 Types + 2 Elite + 2 Boss)
| Enemy | HP | Attack | Special |
|-------|-----|--------|---------|
| Cultivator | 40 | 8-12 | Basic enemy |
| Spirit Beast | 30 | 6-10 | Sometimes buffs |
| Demon Lord | 80 | 10-15 | Sometimes debuffs |
| Shadow Cultivator | 60 | 10-14 | Elite |
| Ancient Spirit | 55 | 9-13 | Elite |
| Heavenly Demon | 120 | 12-18 | Boss |
| Void Emperor | 100 | 14-20 | Boss |

### Map System
- 3 floors with increasing node count (5/6/7)
- Node types: Combat, Elite, Shop, Rest, Boss
- Progress left to right
- Visual indicators: current (glowing), visited (dimmed), locked (faded)

### Shop System
- 4 random cards for purchase
- Health Potion (20 HP, 15 gold)
- Strength Relic (+1 max energy, 50 gold)
- Dynamic pricing (20-50 gold)

### Card Mechanics
- Vulnerable: Take 50% more damage
- Rampage: Damage increases by 5 each play
- Multi-hit: Attacks twice
- AOE: Hits all enemies
- Draw: Extra card draw

## Technical Details
- Pure HTML/CSS/JS, no frameworks
- Single file, no dependencies
- Responsive layout
- Keyboard shortcut (E for end turn)
- Game state object (G) manages all data
- Functions: 25 game logic functions

## Visual Style
- Background: #1a0f0a (dark brown)
- Border: #d4a050 (gold)
- Text: #f0d080 (warm yellow)
- Card hover: translateY(-20px) with glow
- Enemy sprites: Unicode icons (sword, shield, star, curse)

## Game Flow
1. Start at Main Menu
2. Click "Begin Cultivation"
3. Navigate map nodes left to right
4. Combat: play cards, end turn, defeat enemies
5. Shop: spend gold on cards/potions/relics
6. Rest: heal 30% HP or upgrade a card
7. Boss: defeat final boss to win
8. Game Over: shows final score

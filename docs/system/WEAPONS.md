# Weapons — 구조·연구·아트 방향

[제작물](../items/CRAFTABLES.md) · [장착](../items/EQUIPABLE.md) · [연구](RESEARCH.md)

## 연구와 MVP 분리

장기 무기 카탈로그는 **금속가공 중심 60종**의 계열·실루엣을 검토했으며, 초기 출시에서 전체 60종 제작을 보장하지 않는다. 책·보주·토템처럼 대장장이가 직접 만들기 어려운 마법 물품은 Core Set에서 제외하고 투척 무기(투척 칼/도끼/창, 차크람, 쿠나이, 슈리켄, 칼트롭스 등)를 넣는 방향을 합의했다. 활/석궁도 종류마다 가변 형태다.

**Core visual tiers:** Common → Refined → Masterwork 3단계. 그보다 특별한 제작품은 Art Pack 또는 Unique/Named 별도 디자인. 이는 무기의 시각 티어이며 경제 품질·무기 이해도·카드 랭크와 동일하지 않다.

## Core set 60 archetypes — 아트 연구 카탈로그 (MVP 의무 수량 아님)

| Family | 60 종류 후보 |
|---|---|
| Daggers / knives (10) | Dagger, Stiletto, Dirk, Hunting Knife, Rondel Dagger, Push Dagger, Kukri, Karambit, Kris, Main Gauche |
| Swords (12) | Shortsword, Arming Sword, Longsword, Greatsword, Rapier, Scimitar, Saber, Cutlass, Falchion, Estoc, Katana, Zweihander |
| Axes / blunt (12) | Hand Axe, Battle Axe, Great Axe, Bearded Axe, Francisca, Mace, Morningstar, War Hammer, Flail, War Pick, Maul, Club |
| Polearms (10) | Spear, Pike, Halberd, Glaive, Poleaxe, Trident, Partisan, Bardiche, Billhook, War Scythe |
| Ranged (8) | Shortbow, Longbow, Recurve Bow, Composite Bow, Crossbow, Heavy Crossbow, Hand Crossbow, Repeating Crossbow |
| Throwing (8) | Throwing Knife, Throwing Axe, Javelin, Chakram, Kunai, Shuriken, Caltrops, Throwing Hammer |

## Asset representation contract

- 1 logical visual slot = **96×96 pixels**, item art canvas variable size (e.g. dagger 1×2, stiletto 1×3, kukri/karambit 2×2, Main Gauche 2×3). **Not inventory slots.**
- Bow and crossbow canvas size varies by silhouette; longbow tall vs crossbow wide.
- Core art: cute stylized low-poly-inspired miniature pixel weapons, readable silhouettes, restrained teal metal/warm wood palette. No blanket gold/holy decoration.
- Tier variation prioritizes visible geometry, edges, forging precision, balanced shape and subtle steel shine. Common should still be visually appealing.
- Runtime NPC purchase: use the same visual identity, reduced scene LOD and grip anchor, walk out carrying the item.
- Keep big source art, detail and record-card presentation separate from in-scene scaling; sprite sheets packed at build time.
- Free licensed low-poly source models can accelerate asset creation via Blender normalization→orthographic rendering→pixel conversion; verify rights, attribution and NoAI terms per item.

## Daggers: initial variable slots

| Archetype | Grid | Canvas |
|---|---|---|
| Dagger / Hunting Knife | 1×2 | 96×192 |
| Stiletto / Dirk / Rondel Dagger / Kris | 1×3 | 96×288 |
| Push Dagger / Kukri / Karambit | 2×2 | 192×192 |
| Main Gauche | 2×3 | 192×288 |

## Early-play content boundary

Earlier *candidate* for first playable loop: dagger, short sword, hand axe, battle axe, hunting knife, spear, sickle and pickaxe (8 distinct craftables). More recent content MVP also includes **nine repair-tool types across farmer/carpenter/leatherworker**, three per shop, and one unlockable shop product per shop. These are **different planning horizons**: do not silently force 8 starter items and 9 unique repair tools into one Foundation completion gate. Foundation only requires a tiny tested craft→sale interaction, then content onboarding.

## Unique/art packs

Assassin, Druid, Ranger, Nomad, Arcane, Sacred etc. are expansion concepts, not shipped recipes. Art pack does not equal higher quality tier; unique can have its own silhouette/identity/story. No runtime procedural weapon generator in Foundation.

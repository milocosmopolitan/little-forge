# Research — insight-gated discovery

[경험](EXPRIENCE.md) · [무기](WEAPONS.md) · [제작](CRAFTING.md)

## 확정 메커니즘

연구는 **무기 종류, 재련/제련 재료, 제작·제련 기술, 마법·속성(향후)** 같은 교차 분야를 다룬다. 무기 구조·재료·기술·카드를 구별한다. 사용자는 관심 분야를 자유롭게 확장하며, 경로를 영구적으로 잠그는 클래스 선택은 하지 않는다.

- 제작/수리 완료 시 관련 Mastery XP 확정 획득(설계 기준), 해당 분야 Insight는 확률적 발견 및 실패 보정.
- 튜토리얼 관련 제작 3회 이내 이해도 Lv.1 보장.
- **Unknown / Discovered / Unlocked** 구분; 이해도 도달은 연구 자격이며 제작법 즉시 소유가 아니다.
- 계획형 연구: 요구 이해도·선행 노드를 충족하면 연구 가능 후보 공개. 발견형 연구: 적격 제작/재료 실험으로 단서/비용 감소 등 발견. **필수 진행 노드를 RNG로 봉쇄하지 않는다.**
- 재료·기술 연구와 무기 구조 연구를 조합해 레시피 해금. 예: 장검 + 강철 + 열처리.
- 도구 실사용 피드백도 해당 소재/제작 이해도 연구로 연결하며 **도구의 실제 마모 시뮬레이션은 없음**.

## 초기 확장 트리 연구 제안 (콘텐츠 미확정)

6 family tracks: swords, axes, bows/crossbows, polearms, armor, magical metal tooling as later content. 형태/기술/재료의 조합으로 변형을 늘리고 모든 조합을 고유 노드로 만들지 않는다. **코어 무기 아트에서는 책·비금속 중심 마법 도구를 제외**했지만 마법·룬 연구 자체를 폐기한 것은 아니다.

Tier progression and research XP values, fail-pity thresholds, exact node prerequisites/recipes/time/gold costs are balance candidates only; no automatic fixed 5 levels or 60-node tree in MVP.

## Data boundaries

`research_node_id`, `domain`, `insight_requirements`, `prerequisites`, `discovery_triggers`, `cost`, `duration`, `unlocks` as stable data IDs. Persist XP, insight levels, failed eligible discoveries, discovered clues, active timers, completed nodes, RNG, event IDs. Test idempotence, graph cycles, invalid material dependencies, save/load replay.

## Cards and availability

Perk cards are ongoing effects; Enhance cards are per-craft consumption candidates. Duplicate-card fusion planned later. Cards do not replace weapon/material knowledge and basic crafting does not require a card.

## 미지 무기 수리 연구 경로 (2026-10-15)

[DISCOVERY_REPAIR](DISCOVERY_REPAIR.md)에서 처음 보는 무기의 수리를 연구 중심으로 수행하면 신규 구조·재료·제련 기술 단서를 발견할 수 있다. 납기 실패나 수리 실패여도 **분석한 지식은 보존**된다. 연구 단서는 즉시 제작법 해금과 다르며 연구 자격/완료가 따로 필요하다. 첫 발견 보상과 반복 학습 보상은 구별해 중복 파밍을 방지한다. Perk는 제작 경험/연구에서, Enhance는 퀘스트/수리/Salvaging에서 획득한다.

# Progression — 제작 경험, 기술 숙련, 연구 노드

[첫 유산](../mvp/FIRST_LEGACY.md) · [연구](RESEARCH.md) · [경험](EXPRIENCE.md) · [제작](CRAFTING.md) · [도감](SMITHS_CODEX.md)

## 핵심 개념 분리

- **General Crafting XP:** 플레이어 대장장이의 전체 제작 성장/CRAFTED 능력치 진행에 기여한다.
- **Technique Proficiency XP + Level:** 반복된 작업에서 체득한 재사용 가능한 부품·가공 기술(예: `blade_forging`). 무기마다 독립 XP를 만들지 않는다.
- **Research Node XP:** 연구 트리의 특정 노드에서 축적되는 진행도. 기술 숙련과 별도이며 어느 노드에 XP가 들어가는지 규칙이 명확해야 한다.
- **Research Clue:** 블라인드 노드의 존재/정체를 **발견**하게 하는 조건. 즉시 제작법 해금이 아니다.
- **Smith's Codex:** 품목별 제작 기록·현장 평가·최고 품질을 저장한다. 별도의 무기 XP 지갑이 아니다.

기존 `EXPRIENCE.md`의 '이해도'는 도메인/기술 숙련과 인사이트 기록으로 구체화한다. 동일한 완료 이벤트에서 **General XP, 해당 Technique XP, 적격 Research Node XP, Codex 기록**을 일관되게 정산하되 중복 보상을 금지한다.

## 기술 숙련 8가지 — 첫 콘텐츠 검증 후보

| 기술 ID | 해석 | 생활 도구 → 기본 무기 |
|---|---|---|
| `blade_forging` | 칼날 단조 | Kitchen Knife → Dagger / Shortsword |
| `edge_geometry` | 날 형상·절삭 | Sickle/Saw → Curved Blade |
| `handle_fitting` | 손잡이 결합 | Kitchen Knife/Leather Punch → Dagger |
| `hafting` | 자루/장대 결합 | Rake/Hatchet → Spear / Axe |
| `point_forging` | 뾰족한 끝단 | Punch/Awl/Arrows → Spear / Push Dagger |
| `impact_geometry` | 충격·무게 중심 | Hatchet/Cleaver → Hand Axe |
| `serration` | 톱니 가공 | Saw → Serrated Blade |
| `bow_assembly` | 활과 화살 복합 조립 | Hunter Bow & Arrows → Shortbow |

`bow_assembly`는 제련이 아니라 복합 부품 조립이다. 활·화살은 목재/현/깃/화살촉 등의 공급망을 가질 수 있지만 별도 목공 게임이나 세밀한 물리 엔진은 만들지 않는다.

## 첫 아이템별 적립 매핑 — **플레이테스트용 가정, 미확정**

| 의뢰 NPC | 제작물 | 일반 XP | 기술 XP 매핑 | 연구 방향 |
|---|---|---:|---|---|
| Chef | Kitchen Knife | 10 | blade_forging +6; handle_fitting +3 | Dagger; 이후 Shortsword |
| Chef | Cleaver | 12 | blade_forging +4; impact_geometry +5 | Heavy Blade |
| Farmer | Rake | 10 | hafting +6; point_forging +3 | Spear |
| Farmer | Sickle | 12 | edge_geometry +6; blade_forging +3 | Curved Blade |
| Leatherworker | Leather Punch | 10 | point_forging +6; handle_fitting +3 | Push Dagger |
| Leatherworker | Awl | 8 | point_forging +5; handle_fitting +3 | Stiletto |
| Carpenter | Hatchet | 12 | impact_geometry +6; hafting +3 | Hand Axe |
| Carpenter | Saw | 12 | serration +6; edge_geometry +3 | Serrated Blade |
| Hunter | Hunting Knife | 15 | blade_forging +5; edge_geometry +4 | Hunting Knife |
| Hunter | Shortbow | 18 | bow_assembly +8 | Shortbow |
| Hunter | Arrows (bundle) | 8 | point_forging +4; bow_assembly +2 | Arrowheads/Ammo |

**대표 첫 6 제작물:** Kitchen Knife, Rake, Leather Punch, Hatchet, Shortbow, Arrows; 다음 콘텐츠 Cleaver, Sickle, Awl, Saw, Hunting Knife. Hunter에게는 주문/기본 장비/첫 파견이 유기적으로 연결되어야 한다. 기존 주민 각 3가지 고유 도구 목표를 폐기하지 않는다.

## 연구 트리 노드맵

- 별도 관리 화면: 노드 간 연결선, 분야 필터, 줌/팬(필요 시), 선택 노드 상세 패널에 **노드 XP/목표 XP, 요구 기술 Lv., 선행 연구, 단서, 제작법 해금, 출처**를 보여준다.
- 상태: **Hidden(???) → Discovered → Locked(요구조건 부족) → Available/In Progress → Completed**. Discovered 와 Locked는 진행 가능 여부를 나타내는 다른 차원일 수 있으므로 상태 머신 구현 시 `discovered`, `prerequisite_met`, `progress`, `completed` 같은 독립 필드를 권장한다.
- 단서는 Hidden을 공개할 뿐 연구 비용·XP·기술 숙련 요구량을 기본적으로 면제하지 않는다.
- 필수 진행 노드는 RNG가 없어도 도달 가능하게 만든다. 희귀 연구는 NPC 미지 무기 분석·탐험 단서와 연결한다.
- **연구 노드의 XP는 적격 작업/지정 연구 활동에서만** 적립. 한 행동을 전체 무기 트리에 방송하지 않는다.
- **노드 XP 가득 참 ≠ 완료**: 선행조건 충족과 완료 트랜잭션이 필요할 수 있다. 완료 후 제작법이 사용 가능해진다.

## 기술 레벨과 연구의 상호작용

기술 XP는 반복 제작으로 쌓이고, 연구는 높은 기술 레벨 상한이나 특수 공정을 열 수 있다. 예시: Blade Forging Lv.1–2 기본, Lv.3–4 중급 기술 연구 완료 후, Lv.5 고급 기술 연구 후. **숫자·상한 구간 모두 미확정.** XP 상한 도달 후 손실/이월 정책도 조정할 것.

**순환 잠금 금지:** Lv.3을 열 연구가 Lv.3을 선행조건으로 요구해서는 안 된다. 노드 의존성 DAG 및 기술 게이트의 도달 가능성을 콘텐츠 검증기로 테스트한다.

제작 가능 조건은 `recipe_research_completed AND technique_levels_met AND materials_available`가 기본. 연구 발견, 노드 진행, 기술 레벨, 제작 가능 여부를 혼동하지 않는다.

## 사용자 인터페이스

1. **Skill Journal:** 기술별 Lv/XP/레벨 상한·다음 단계 연구 조건, 적용 제작물 목록.
2. **Research Tree:** 발견된/숨겨진 노드 맵, 선택 노드 XP와 해금 조건, 연결 기술.
3. **Smith's Codex:** 제작물별 최초 제작, 최고 품질, 제작 횟수, 평가 및 세계 기여.

작은 960×540 관리 창에서 세 정보 집합을 한 화면에 동시 배치하지 않는다. 현재 UI epic #58과 통합하되 제작 중 창 축소와 포커스 정책을 유지한다.

## QA

이벤트별 보상 idempotency, 저장/로드·오프라인 동일 결과, 스텝 분할 불변성, 3회 튜토리얼 이해도 보장, 연구 DAG 유효성, 기술 게이트 순환 불가, 숨긴 노드의 정보 누출 방지, 품목 XP 누락 검증, 제작법을 발견만 했는데 사용 가능한 버그 방지.


## Expedition combat slice (2026-10-18)

The first expedition now uses deterministic five-wave automatic combat. See [combat rules](EXPEDITION.md) and the implementation boundary in [Hunter return](FIRST_EXPEDITION.md). Weapon IDs retain actual field-use and defeat counts; five-wave completion reopens the forest and awards eight timber once. Full research/technique progression and the broader Codex UI remain separate open work, not completion claims from this combat slice.

# 초기 완제품 카탈로그 v1

상태: **완제품 에셋 22종 제작 완료**. 현재 main의 `assets/items/finished/finished-goods-catalog.json`은 22종 모두 `status: done`으로 기록한다. `assets/items/finished/icons/` 및 `finished-v1.png`가 존재한다. 이는 게임플레이 레시피 구현 완료를 뜻하지 않는다. 정본: [완제품 카탈로그](../../assets/items/finished/finished-goods-catalog.json), [완제품 아트 명세](../design/FINISHED_GOODS_V1.md).

| 단계 | ID | 품목 |
|---|---|---|
| P0 | wooden_club | 몽둥이 |
| P0 | wooden_staff | 스태프 |
| P0 | wooden_shield | 나무 방패 |
| P0 | kitchen_knife | 부엌칼 |
| P0 | frying_pan | 주방 팬 |
| P0 | farming_hoe | 괭이 |
| P0 | sickle | 낫 |
| P0 | woodcutting_axe | 벌목 도끼 |
| P1 | cleaver | 식칼 |
| P1 | hand_saw | 톱 |
| P1 | shears | 가위 |
| P1 | fleshing_knife | 무두칼 (양손 드로나이프) |
| P1 | leather_awl | 가죽 송곳 |
| P1 | leather_punch | 가죽 펀치 |
| P1 | wood_chisel | 목공 끌 |
| P2 | dagger | 단도 |
| P2 | hunting_knife | 사냥칼 |
| P2 | hand_axe | 손도끼 |
| P2 | spear | 창 (짧은 봉) |
| P2 | pike | 파이크 (긴 봉) |
| P2 | halberd | 할버드 (긴 봉 + 창촉 + 도끼날) |
| P2 | arrow | 화살 묶음 |

**합계 22개** (P0 8, P1 7, P2 7). **후속 3개**: shortbow, reinforced_shield, magic_wand — 활대/활시위, 방패 보강 연구, 특수 마법 재료가 선행되어야 한다.

## 제작된 아틀라스와 후속 작업

- `finished-v1.png`: 256×128, 8×4 셀. P0/P1/P2 총 22개 완성, 마지막 행은 P3 예약.
- 나무 방패는 **사각형**으로 확정: 세로 판자와 가죽 스트랩, 철 리벳. 이전의 형태 미정 설명은 더 이상 적용되지 않는다.
- 다음 작업은 22종 재생성이 아니라 **재질별 완제품 변형, 런타임 카탈로그 연결, 레시피/인벤토리/연구 검증**이다.

## 제작 에셋 규격
32×32 트루 픽셀 RGBA, 최대 16색, 1px 투명 테두리, 좌상단 광원, 재료/부품 팔레트 재사용. 아이콘은 2× 정수 확대를 기본으로 한다. 부품의 재질별 파생은 별도 고정 레시피가 아닌 속성 기반. 무기 상세·장면용 스프라이트는 아이콘과 별도 LOD이며, 32×32를 비정수 배율로 축소하지 않는다.

## 작업 게이트
1. 제작된 22종의 1×/2× UI 식별성을 검증한다.
2. 완제품별 금속·목재·가죽 복합 팔레트 변형을 설계한다.
3. 현재 JSON 아이템 ID를 Rust 인벤토리·제작·연구에 연결한다.
4. 기존 16개 연구 노드, NPC 9종 도구 목표, 탐험 장비 소유권과 레시피 해금/재고 연결을 검증한다.

[자원](RESOURCE_CATALOG_V1.md) · [부품](INITIAL_PARTS_V1.md) · [레시피](INITIAL_RECIPES_V1.md).

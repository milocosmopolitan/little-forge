# 초기 완제품 카탈로그 v1

상태: **다음 제작 에셋 계획**. 재료 40종(PR #126)과 기본 부품 23종(PR #127)은 병합되었으나, 아래 완제품 32×32 PNG는 별도 제작 대상이다.

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

## 제작 에셋 규격
32×32 트루 픽셀 RGBA, 최대 16색, 1px 투명 테두리, 좌상단 광원, 재료/부품 팔레트 재사용. 아이콘은 2× 정수 확대를 기본으로 한다. 부품의 재질별 파생은 별도 고정 레시피가 아닌 속성 기반. 무기 상세·장면용 스프라이트는 아이콘과 별도 LOD이며, 32×32를 비정수 배율로 축소하지 않는다.

## 작업 게이트
1. 방패 외형(사각형/원형) 결정; `shield_core`는 보류.
2. P0 8개 제작 및 실제 UI 1×/2× 검증.
3. P1 7개, P2 7개 제작. 창/파이크는 아이콘에서 길이 차이가 읽혀야 한다.
4. `finished-goods-catalog.json`, 아틀라스, 출처, 검증 시트, 재질 변형 규칙을 부품 파이프라인과 같은 방식으로 제공.
5. 기존 16개 연구 노드, NPC 9종 도구 목표, 탐험 장비 소유권과 레시피 해금/재고 연결을 별도 검증한다.

[자원](RESOURCE_CATALOG_V1.md) · [부품](INITIAL_PARTS_V1.md) · [레시피](INITIAL_RECIPES_V1.md).

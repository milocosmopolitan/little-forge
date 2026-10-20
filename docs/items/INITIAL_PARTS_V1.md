# 초기 제작 부품 카탈로그 v1

상태: **PR #127 부품 에셋 병합 완료; 제작 경제는 설계안**. 정본: [parts-catalog.json](../../assets/items/parts/parts-catalog.json), [아트 명세](../design/CRAFTING_PARTS_V1.md).

32×32 RGBA, 최대 16색, 1px 투명 여백, 좌상단 광원. `parts-v1.png`는 256×128 (8×4).

| 분류 | 부품 ID | 수 |
|---|---|---:|
| 목재 | wooden_pole, wooden_plank, wooden_rod, arrow_shaft, wooden_handle, leather_wrapped_handle | 6 |
| 금속 날·촉 | straight_blade, wide_blade, awl_point, axe_head, hoe_head, saw_blade, sickle_blade, arrowhead, spearhead, shear_blade | 10 |
| 금속 결합 | metal_rivet, metal_fitting, shield_boss | 3 |
| 가죽 | leather_strip, leather_grip, leather_strap, leather_reinforcement | 4 |
| **합계** | **제작된 기본 부품 아이콘** | **23** |

`shield_core`는 카탈로그에 정의되었지만 형태 미확정으로 **보류**되어 아이콘이 없다. `pan_body`는 삭제되었으며 주방 팬은 철 주괴 ×2로 직접 제작한다.

## 대표 가공식
| 투입 | 결과 |
|---|---|
| 수종별 원목 | 봉 ×1 (length=short/long), 판자 ×2 |
| wood_branch | 짧은 막대 ×2 또는 화살대 ×3 |
| 원목 + pine_resin | 나무 손잡이 |
| 원목 + pine_resin + 종별 가공가죽 | 가죽 감싼 손잡이 |
| iron_ingot | 칼날/넓은 칼날/송곳날/도끼날/괭이날/톱날/낫날/화살촉/창촉/가윗날 |
| iron/bronze ingot | 리벳 |
| iron/bronze/brass ingot | 결합구 |
| 가공가죽 | 가죽 끈/그립/스트랩/보강재 |

정확한 수량·수율은 parts-catalog.json의 현재 정의를 참고하고 게임 경제 적용 전에 검증한다. 강철 절삭 부품은 연구 후 허용; 청동·황동을 범용 칼날 재료로 사용하지 않는다.

부품은 독립 인벤토리 아이템이며 재질(목재 수종·금속·가죽 동물)을 보존한다. 봉의 길이(short/long)는 **속성**이고 아이콘은 공유한다. 완제품에 사용한 부품을 다시 원재료로 중복 청구하지 않는다. 재질 변형은 팔레트로 표현하되 복합재 손잡이는 두 팔레트 처리가 필요하다.

관련: [자원](RESOURCE_CATALOG_V1.md) · [레시피](INITIAL_RECIPES_V1.md) · [완제품](INITIAL_FINISHED_GOODS_V1.md).

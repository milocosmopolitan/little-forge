# Hunter neutral asset

기본 자세와 공통 표정·동작을 제작했습니다. 애니메이션은 `previews/animation.html`에서 확인합니다. 모든 기본층·표정 부품·장비·프레임은 RGBA 128×160, 발 기준점 `(64,140)`이며 공유 팔레트 상한은 64색입니다.

- `neutral.png`: 전체 착용 합성본
- `base-neutral.png`: 장비 없는 완전한 머리·셔츠·맨손·바지·부츠
- `equipment/`: 활, 화살통/화살, 털모자/꼬리, 망토, 가죽 의상 묶음, 장갑, 부츠 털장식
- `previews/index.html`: 1배/4배, 밝은/어두운 배경, 개별 장비 토글
- `previews/comparison-4x.png`: 원본·기본층·전체 착용 비교
- `character-spec.yaml`: 좌표, 소켓, 레이어 순서와 제작 범위

가죽 코트·가슴끈·벨트·가방·칼집은 하나의 장비로 묶었습니다. 참조에서 서로 가려진 가죽 부품을 임의로 독립된 완성품처럼 등록하지 않았습니다. 모자 안 머리, 의상 안 셔츠, 장갑 안 맨손과 털장식 안 부츠는 이미지 생성으로 복원했습니다.

이미지 생성은 내장 `image_gen`을 사용했습니다. 원본과 생성 후보는 `source/`에 보존했고 실제 프롬프트는 `source/generation-prompts.json`, 네이티브 등록·국소 정리는 `source/build-character.py`에 기록했습니다.

```sh
rtk proxy python3 assets/npc/hunter/source/build-character.py
rtk proxy python3 assets/npc/hunter/qa/verify-neutral.py
```

`atlas.png`는 중립 한 프레임의 제작용 아틀라스입니다. 표정·동작 명세와 검증은 `ANIMATION.md`, `qa/animation-validation.json`에 있습니다. 게임 런타임은 연결하지 않았습니다. 구조 검증 결과는 `qa/validation.json`에 있습니다.

## 목 연결 수정

턱수염·목·셔츠 깃을 내장 이미지 생성 도구로 함께 복원했습니다. 이전 사선 목 채움 코드는 제거했습니다. 원본 후보와 실제 프롬프트는 `source/neck-repair/`, 수정 전후 비교는 `previews/neck-repair-before-after.png`, 변경 범위 검증은 `qa/neck-repair-validation.json`에 보존했습니다.

## 정수리와 손 픽셀 보완

모자 없는 정수리를 네이티브 3px 높여 둥근 볼륨을 복원하고, `(45,91)`, `(61,92)`의 투명 픽셀 두 곳을 인접 피부색으로 채웠습니다. 얼굴·목과 전체 착용 합성본은 이전과 픽셀 단위로 같습니다. 생성 기록은 `source/hair-hand-repair/prompt.json`, 비교는 `previews/hair-hands-before-after.png`, 검증은 `qa/verify-hair-hands.py`에 있습니다.

## 모자 없는 헤어스타일 교체

이전 정수리 확대안 대신 둥근 옆가르마를 새로 생성했습니다. 세 갈래 돌출부를 없애고 큰 머리카락 덩어리의 흐름을 정리했습니다. 62×50 머리 후보를 원래 좌표 `(34,16)`에 등록했으며 정수리를 늘리는 변형은 제거했습니다. 실제 프롬프트는 `source/hair-restyle/prompt.json`, 비교는 `previews/hair-restyle-before-after.png`에 있습니다.

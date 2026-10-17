# 플레이어 기본 자세

여자·남자 각 1개의 오른쪽 3/4 방향 기본 자세다. 현재 모든 캐릭터 부품과 합성 PNG는 **Grid B 8×10칸, 128×160 RGBA, 발 기준점 `(64,140)`**를 공유한다. NPC의 공통 캔버스와 같은 규격이다. 이동·공격·방향 전환이나 게임 컨트롤 연결은 이번 제작 범위에 포함되지 않는다.

원본 제작 좌표 96×128에 `(16,16)`을 더해 공개 캔버스로 옮긴다. 여백 확장으로 기존 얼굴을 확대하거나 리샘플링하지 않는다. `source/reference-native.png`와 `source/revision-0/`는 이전 제작 좌표의 입력/기록이며, 게임용 부품이 아니다. 명세의 원점·소켓·얼굴 ROI·발 위치와 `character.json`도 공개 좌표로 갱신했다.

## 현재 얼굴과 목 보정, revision-3

이전 입 ROI가 원본의 올라간 입꼬리를 누락했다. 원본의 닫힌 미소를 기준으로 코·볼·귀·눈과 목 연결을 내장 image_gen 편집 모드로 함께 보정했다. 같은 완성 얼굴과 눈/입 제거 후보를 동일 좌표로 등록하며, 제거 후보는 표정 아래 피부에만 사용한다. 여자 생성 격자의 수평 위상 0.125 네이티브 픽셀 보정은 얼굴 전체에 공통 적용한다. 눈/입의 개별 확대나 이동은 없다. 최종 네이티브 픽셀은 정수 좌표와 nearest 샘플링이다.

남자 턱/목 접점과 노출 목 피부는 국소 보정했다. 셔츠·어깨·팔·하체·장비는 유지한다. 검수기는 지정된 남자 목 영역 외 몸과 장비의 픽셀 일치, 두 행으로 연결된 미소와 등록 원본의 재합성 일치를 확인한다. 상체 파일 전체가 이전과 동일하다는 주장은 현재 보정에는 적용되지 않는다.

[현재 얼굴](previews/heads-current.png), [이번 수정 전후](previews/heads-before-after.png), [정확한 생성 요청과 경로](source/revision-3/prompts.json). 각 캐릭터의 `source/revision-3/`에 후보·등록 원본·수정 전 부품·변경 마스크를 보존한다. 채택 범위는 `source/restore-face.mjs`에 정의한다.

아래 revision-2와 몸 보정 단계의 눈·입·목 불변 기록은 이전 단계의 기록이다.

## 이전 얼굴 클러스터 정리, revision-2

내장 image_gen의 편집 모드로 머리카락·피부 후보를 제작했다. 생성된 머리 내부의 연결된 색 면만 기존 팔레트로 등록하고, 기존 머리 알파 외곽·목 소켓·눈·입은 유지했다. 몸·앞치마·벨트·장갑은 수정 직전 부품과 픽셀 일치를 검사한다. 이전 문단의 얼굴 전체 픽셀 고정은 몸 보정 단계의 기록이며, 현재 머리 내부 색 면은 revision-2 결과다.

- [현재 얼굴 확대](previews/heads-current.png)
- [얼굴 수정 전후, BEFORE / AFTER](previews/heads-before-after.png)
- [정확한 편집 요청과 생성 경로](source/revision-2/prompts.json)

각 `source/revision-2/generated-head.png`는 생성 후보이고 `head-edit-target.png`는 입력이다. 최종 적용은 `source/clean-head.mjs`에서 외곽과 소켓을 보존하며 처리한다. `source/revision-2/before/`에 수정 직전 부품을 보존했다.

## 미리보기

- [작업복 착용 남녀](previews/players-equipped.png)
- [장비 없는 남녀](previews/players-base.png)
- [망치 착용 남녀](previews/players-with-hammer.png)
- [NPC 4명과 플레이어 비교, 공통 2배율](previews/npc-player-comparison-2x.png)
- [수정 전후, 왼쪽 이전 / 오른쪽 현재](previews/quality-before-after.png)

각 캐릭터의 `previews/hand-registration.png`는 위쪽 가까운 손/아래쪽 먼 손, 왼쪽 맨손/가운데 등록된 장갑/오른쪽 합성 결과다.

각 캐릭터의 `previews/reference-base-equipped.png`는 왼쪽부터 등록된 레퍼런스, 장비 없는 몸, 작업복, 망치 착용이다. `face-comparison.png`는 레퍼런스/장비 없음/착용 얼굴을 같은 좌표·배율로 비교한다. `layers-light.png`, `layers-dark.png`는 부품별 검수 이미지다. 모든 확대는 nearest 정수 배율이다.

## 파일과 합성

`female/`과 `male/` 각각:

- `head-base.png`: 피부로 채워진 얼굴과 완전한 머리. 눈·눈썹·입·분리 가능한 머리 장식 제외.
- `eyes/open.png`, `mouth/rest.png`: 원본 레퍼런스의 특징을 같은 위치에서 분리.
- `body/upper-base.png`, `body/lower-base.png`: 셔츠·맨손과 완전한 바지·부츠. `body/arms-front.png`는 앞치마 위 팔 순서용이며 별도 팔을 추가한 그림이 아니다.
- `equipment/`: 앞치마, 벨트/도구, 장갑, 머리 장식. 남자는 수건도 별도 분리.
- `props/hammer.png`: 선택 가능한 망치.
- `base-neutral.png`, `neutral.png`, `neutral-with-hammer.png`: 레이어에서 생성한 합성본.
- `character.json`: 합성 순서와 파일 경로. `character-spec.yaml`: 인물별 규격과 네이티브 측정값. JSON 문법의 유효한 YAML로 저장한다.

하체 → 상체 → 머리 → 눈/입 → 앞치마 → 벨트/도구 → 수건(남자) → 앞 팔 → 장갑 → 머리 장식 → 망치 순서다. 플레이어 런타임을 변경하지 않았으며, 현재 출력은 제작용 부품과 합성 PNG다.

## 출처와 복원

여자는 `ChatGPT Image Oct 17, 2026, 09_00_18 AM.png`, 남자는 `Cozy Fantasy Blacksmith Sprite.png`를 정체성·비율·화풍의 기준으로 사용했다. 원본 복사본은 각 `source/reference.png`다. 등록된 레퍼런스를 96×128 공통 축척과 발 위치로 샘플링하고 팔레트를 적용했다. 고해상도 원본 RGB와 바이트가 같다는 의미는 아니다.

내장 `image_gen`으로 장비 아래 셔츠·골반·맨손, 머리 장식 아래 머리카락, 눈/입 아래 피부, 장비에 가려진 앞치마와 완전한 장갑을 복원했다. 원본 얼굴·코·볼·머리의 나머지 부분을 유지하고 눈/입은 독립 확대나 이동 없이 원본 네이티브 위치에서 합성한다. 복원한 부분은 관찰한 원본과 구분해 명세에 기록했다.

앞치마는 몸과 벨트가 없는 완전한 가죽 후보에서 등록하고, 원본의 어깨끈/버클을 연결한다. 장갑은 맨손 자세를 참조한 별도 완전한 후보를 사용한다. 벨트/도구·수건·망치는 보이는 원본 경계에서 분리하며, 이 자세에 맞춘 제작 레이어다. 새로운 손 위치나 동작에서 그대로 맞는다고 가정하지 않는다.

공유 팔레트 상한은 캐릭터별 80색이다. 이전 얼굴·눈·입·머리 장식의 팔레트와 픽셀을 고정하고, NPC 수준의 셔츠 주름·가죽·강철 부츠·황동 디테일용 재질색만 추가했다. 레이어마다 80색씩 따로 배정하지 않는다. 불투명 알파는 255, 나머지는 0이다.

## NPC 비교 후 보정

농부·가죽세공사·요리사·목수를 같은 128×160 캔버스, 동일 2배율, 같은 발 위치에서 비교했다. 가죽세공사와 목수의 의상 경계·옷 주름·부츠 재질을 스타일 참조로 사용했고, 플레이어의 얼굴/체형을 NPC 것으로 바꾸지 않았다.

내장 image_gen으로 셔츠·맨손·바지·부츠 선과 재질 후보를 다시 생성하고, 몸만 새 후보에서 취했다. 머리·눈·입·머리 장식은 이전 부품을 그대로 사용한다. 얼굴 비교는 공개 이미지에서 `(16,16)` 패딩을 뺀 뒤 이전 부품과 픽셀 일치를 검사한다.

공통 장갑에 성별별 이동값만 주던 방식을 제거했다. 실제 맨손 픽셀로 손목/손가락 영역을 잡고, 가까운 손/먼 손마다 장갑 후보를 독립 등록한다. 손 영역의 맨살 노출은 0픽셀이며, 손에서 2px보다 멀리 나간 부분은 짧은 손목 커프를 포함해 각 장갑 최대 4픽셀로 제한한다. 최종 장갑 크기·위치·피부 노출 수·망치 접점 보정은 각 `source/glove-registration.json`에 기록한다. 셔츠 몸통이 앞치마 앞으로 올라오지 않도록 앞 팔의 재질 소유도 제한한다.

품질 후보와 정확한 생성 요청은 [source/revision-1/prompts.json](source/revision-1/prompts.json)에 보존한다. `source/revision-0/`는 이전 부품의 기록이며 빌더가 고정 얼굴/팔레트 입력으로 사용한다. 새 후보의 얼굴은 최종 머리로 사용하지 않는다.

정확한 생성 요청은 [source/prompts.json](source/prompts.json), 원본 등록은 각 `source/reference-registration.json`, 후보 등록과 부위 경계는 `qa/coverage-report.json`, `source/ownership.json`에 남겼다.

## 재현 및 검수

```sh
rtk proxy node assets/characters/source/prepare-reference.mjs
rtk proxy node assets/characters/source/build-characters.mjs
rtk proxy python3 assets/characters/qa/verify-characters.py
```

검수는 128×160 규격·전체 팔레트·이진 알파, 부위 재합성, 눈 아래 피부, 머리 실루엣·목·눈·입 불변, 승인된 몸·장갑·앞치마·벨트의 픽셀 일치, 어깨 경계와 몸 분리 손실, 장갑의 손 피부 가림, 셔츠 전경 조각 금지, 모든 장비 토글의 바탕 손실, 발 위치, 정확한 4배 확대, 바이트 동일 재빌드를 확인한다. 시각 비교는 구조 검사와 따로 수행하며 결과는 `qa/validation.json`에 기록한다.

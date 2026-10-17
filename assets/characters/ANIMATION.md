# 플레이어 표정과 동작

남녀 각각 128×160 RGBA 네이티브, 발 피벗 `(64,140)`, 54개 포즈/표정 프레임과 17개 재생 클립이다. 승인된 `neutral.png`, `base-neutral.png`, `head-base.png`, `eyes/open.png`, `mouth/rest.png`는 유지한다. 캐릭터 기본 빌더는 기준 자세를 담당하고 애니메이션 빌더는 별도 출력만 생성한다.

## 바로 보기

- [독립 미리보기](previews/animation.html): 남녀 동시 재생, 클립 선택, 장비/기물 토글, 1/2/4배 nearest, 정상/0.5/0.25 속도, 프레임 시크·일시정지·재시작, 이벤트 표식.
- [여자 8표정](female/previews/expressions-8.png), [남자 8표정](male/previews/expressions-8.png).
- [여자 보행 6키](female/previews/walk-6-base.png), [남자 보행 6키](male/previews/walk-6-base.png).
- [여자 작업 5종](female/previews/work-actions-review.png), [남자 작업 5종](male/previews/work-actions-review.png).
- 각 `previews/*-base.gif`, `*.gif`는 장비 없는/착용 캐릭터, `*-scene.gif`는 기물 합성이다. 시트의 0/3/5키만으로 전체 동작 검수를 대체하지 않는다.

HTTP로 연다. 예: `rtk proxy python3 -m http.server 8773 --bind 127.0.0.1`, 이후 `http://127.0.0.1:8773/assets/characters/previews/animation.html`. PNG를 갱신한 뒤 이미 열린 페이지는 새로고침한다.

## 제작한 세트

공통 8표정은 neutral, talk-small, talk-wide, talk-round, blink, happy, surprised, relaxed다. 눈 5종/입 4종과 20조합 비교본을 보존한다. 기본 미소를 바꾸지 않으며 표정 간 몸·머리·장비 변경은 특징 픽셀 합집합으로 제한한다.

표정 후보에서 추출할 때 피부색 판정만으로 머리·턱 경계를 보호할 수 없다. 현재 빌더는 등록된 얼굴에서 측정한 눈/눈썹 두 자리와 입 상태별 내부 범위만 추출한다. 여자 blink의 머리 주변 변화·입꼬리 점과 남녀 talk의 이중 턱선을 이 범위 제한으로 제거했다. 검증기는 표정별 머리 기본층 일치, 머리·턱 보호 영역 불변, blink/happy/relaxed의 입 영역 불변도 확인한다. [수정 후 얼굴 비교](qa/expression-contour-review.png)는 위 여자/아래 남자, 왼쪽부터 중립·blink·talk-small·talk-wide·talk-round다.

공통 클립은 idle, blink, talk, talk_blink, happy, surprised, relaxed, nod, shake_head, give, walk다. 방향 기준 front-neutral/back-neutral과 정면 front_blink도 추가했다. 후면에는 눈/입이 없다.

작업은 hammering, quenching, forging, researching, salvaging이며 각각 6키다. `forging`은 화로 가열이다. `quenching`과 같은 별도 tempering 클립을 만들지 않았다. 연구는 정면, 분해는 후면이며 특정 무기 형태를 드러내지 않는다.

실제 클립 시간·반복 범위·이벤트·소켓·기물 원점·가림 순서는 `female/animation.json`, `male/animation.json`이 권위 원본이다. give의 `transfer`는 빈손 전달 제스처의 타이밍 표식이다. 망치질 `hammer_contact`, 담금질 `water_enter`/`water_exit`, 가열 `heat_enter`/`heat_exit`, 분해 `parts_place`는 결과 지급이나 게임 명령을 실행하지 않는다.

보행은 신체 기준 가까운 쪽이 오른쪽이다. 좌접지에서는 오른팔 앞/오른발 뒤, 우접지에서는 왼팔 앞/왼발 뒤다. 원본 첫 보행 후보의 반복된 팔 스윙을 채택하지 않고 반대 위상 후보를 함께 등록했다. 통과 키·루프 연결은 6키를 유지한다.

## 레이어와 출처

`poses/<id>/`에 머리, 눈/입, 상/하체, 앞 팔, 장비, 손에 든 도구, 바탕/합성본을 보존한다. `frames/base/`와 `frames/`는 여기서 파생한다. 표정은 기존 바탕에 특징을 교체하고, 방향이 달라진 머리는 그 방향의 일관된 생성 얼굴에서 눈/입과 피부를 분리한다. 눈·입을 각각 확대해 맞추지 않는다.

[정확한 image_gen 편집/생성 요청과 원본 경로](source/animation/prompts.json)를 모두 보존한다. 기본 정체성은 현재 채택 레이어와 고해상도 원본이다. 대화의 작업 구도 프리뷰는 동작 참고이며 캐릭터 디자인으로 교체하지 않았다. `*-smith-compact.png`는 남자 날 길이를 줄인 채택 후보, 이전 `male-smith.png`는 출처 기록이다.

이전 `work-repair` 동작은 사용자가 머리 고정, 어깨 파손, 허리 분리를 지적해 폐기했다. 현재 작업 5종은 [hip-pivot/prompts.json](source/animation/hip-pivot/prompts.json)의 18개 생성·편집 원본에서 다시 만든 검토본이다. 골반과 접지점을 기준으로 몸을 숙이며 머리·목·어깨·손이 함께 이동한다. 중립 머리를 화면에 고정하거나 중립 앞치마를 숙인 몸 위에 붙이지 않는다.

각 자세는 기본 의상과 작업복 착용의 완전한 원본을 각각 보존한다. 몸 동작에서 부위를 분리한 뒤 3/4 머리·눈·입은 승인된 기본층의 픽셀을 그대로 사용하고 목 접점에 맞춰 함께 이동한다. 연구는 정면 몸통에 승인된 3/4 머리를 돌려 쓰며 후면 머리는 기존 동작 원본을 유지한다. 최종 기본층 재합성은 등록된 바탕과 정확히 일치해야 한다. 착용본의 장비와 도구는 `rendered_variant_layers`에 포함되며 교환 가능한 독립 장비층이라고 주장하지 않는다. `props/held.png`는 외부 도구의 가림 가이드이며 완전한 도구 원본은 아니다. 완전한 그림은 생성 원본과 합성본에 남는다. 기본층의 도구 뒤 빈 공간은 생성 편집으로 채우고 투명 구멍으로 삭제하지 않는다.

화로는 전체 기물 → 캐릭터 순서다. 담금질은 기물 좌표의 `[6,18,59,48]` 내부에 들어간 도구 픽셀만 가린 `frames/scene-*` 파생본을 사용하며 원본 프레임/도구층은 유지한다. 연구대는 긴 다리 편집본을 128×96으로 등록했고 필기 접점은 `(64,26)`이다. 캐릭터 발은 월드 y=156, 연구대 앞 다리 끝은 y=169, 분해대 다리 끝은 y=151로 배치해 앞뒤 바닥 관계를 보존한다. 연구대 위에 다시 합성하는 층은 손·팔이며 얼굴 조각을 포함하지 않는다.

시트는 알파 임계값 224로 읽고 공유 팔레트에 매핑한다. 작업 시트는 전체 이미지의 연결된 캐릭터 단위로 등록하여 셀 경계를 넘은 도구를 자르지 않는다. 시트별 공통 샘플 간격과 발 기준을 쓰며 프레임별 머리 고정이나 체형 크기 맞춤을 하지 않는다. 전체 도구를 포함하는 공통 배율을 사용하고 등록 원본 영역과 샘플 좌표는 각 포즈의 `source_registrations`에 기록한다.

기본 캐릭터 팔레트는 80색 상한을 유지한다. 애니메이션의 달군 금속과 강철 도구용 공유 색 6개를 추가해 상한 86색으로 명시한다. 불·물은 기물의 별도 층이다. 기물은 [기존 출처](../forge/workshop/PROVENANCE.md)의 후보에서 네이티브로 등록한 별도 `stations/` 에셋이며 기존 workshop catalog나 런타임 에셋을 덮어쓰지 않는다.

## 아틀라스와 검수

성별별 atlas.png에 기본/착용 프레임을 함께 패킹한다. 회전 없이 trim offset/source_size/pivot을 보존하고 시각적으로 동일한 rect만 공유한다. extrude 1px + 투명 gutter 1px를 각 방향에 둔다. `atlas-manifest.json`은 제작용 스키마이며 기존 Bevy 소비자에 자동 호환된다는 뜻이 아니다.

```sh
rtk proxy node assets/characters/source/build-characters.mjs
rtk proxy node assets/characters/source/animation/stations.mjs
rtk proxy node assets/characters/source/animation/build.mjs
rtk proxy node assets/characters/source/animation/scenes.mjs
rtk proxy python3 assets/characters/qa/verify-characters.py
rtk proxy python3 assets/characters/qa/verify-animation.py
```

네이티브 규격/알파/팔레트, 기본/착용 원본 각각의 무손실 부위 재합성, 숙임 시 머리·목 이동, 승인 중립 불변, 표정 외 변경 금지, 후면 얼굴 부재, 캔버스 잘림, 클립 시간/참조, 기물 접점 좌표 등록, 아틀라스 복원과 extrusion/gutter 내용을 검사한다. 좌표 등록 검사는 모든 키의 실제 손 그립과 동작 품질에 대한 시각 검수를 대신하지 않는다.

sprite-optimization 정적 결과는 각 `qa/sprite-audit.json`이다. GPU 배칭·드로우 콜·오버드로우·실기기 압축과 게임 컨트롤/시뮬레이션 연동은 미측정/미연동이다. PNG는 무손실 마스터이며 sampler nearest, mipmap 없음으로 기록한다.

검수 기록: `qa/animation-validation.json`, `qa/animation-rebuild.json`, `qa/browser-playback.json`, `qa/review-status.json`. 기존 34시나리오 기록은 이전 빌드의 재생 증거다. 작업 수정본은 `?qa=work`에서 5클립 × 정상/0.5배 × 착용/미착용 × 밝은/어두운 배경 = 40시나리오를 재생하며 `qa/work-repair-review.json`에 결과를 구분한다. 이것은 제작용 재생 검수이며 Bevy 런타임 성능 검증이 아니다. [작업 씬 비교](qa/work-scenes-repaired.png), 성별별 `previews/work-all-30.png`에서 모든 작업 키를 확인할 수 있다.

## 얼굴 정체성과 리서치 팔 복원

이전 hip-pivot-v2는 자세는 개선됐으나 얼굴과 화풍이 달라졌다는 사용자 지적으로 교체했다. `source/animation/style-restoration/`은 비교본과 정면·후면 각도 후보/빈 얼굴 후보와 정확한 생성 요청을 보존하지만 얼굴형이 달라 채택하지 않았다. 3/4 작업과 연구 24키는 승인된 루트 부품을 사용한다. 현재 머리에는 아래 소유권 마스크를 적용하고 눈·입과 함께 정수 이동한다. 마스크 외 머리와 눈·입의 원본 픽셀이 동일한지 검사한다. 이 이동은 키별 목 접점을 따라가므로 화면에 머리를 고정하지 않는다. 기존 셔츠·장갑·앞치마·하체 동작은 다시 생성하지 않았다.

리서치에서 책상 위에 합성하는 층은 맨살인 손목만이 아니라 양쪽 소매와 팔꿈치·손을 포함한다. 머리/얼굴은 이 층에서 제외한다. 정적 검수는 양쪽 소매와 팔의 존재, 기물 합성본의 앞 팔 픽셀 보존을 확인하고 재생 화면도 별도로 검토한다. 연구 몸통은 정면이고 머리는 기존 3/4 각도를 유지한다. 새 정면 얼굴은 납품 프레임에 사용하지 않는다.

이전 approved-face-v5 얼굴/팔 수정본 검수는 `qa/work-style-review.json`, 원본 얼굴 픽셀 비교와 장비 착용 목 연결은 `qa/animation-validation.json`, 리서치 팔 합성 검사는 `qa/work-style-validation.json`에 구분해 기록한다. 비교 시트는 `qa/face-style-restored.png`, `qa/research-arms-restored.png`다.

## 목·어깨 연결 수정

`approved-face-v5`의 수평 머리 분리가 높은 어깨를 지웠고, 얼굴 피부 최하단을 목 기준으로 삼아 머리를 앞으로 밀었다. 이전 한 점 인접 검사는 이 오류를 검출하지 못했다. 현재는 포즈 원본의 셔츠·옷깃·어깨를 보존하고 실제 목 피부와 옷깃의 접점으로 승인된 머리와 목 피부를 등록한다. 기존 턱 윤곽이 새 목 안에 남지 않도록 승인된 피부 연결층을 사용한다. 얼굴·눈·입은 같은 원본 픽셀을 유지한다.

`body/source-connection.png`와 `rendered/body/source-connection.png`는 원본에서 보존한 연결부, `body/neck-skin.png`와 `rendered/body/neck-skin.png`는 승인된 원본의 목 피부다. 최종 상체에 포함되며 별도 중복 렌더링하지 않는다. `head_identity`의 `root_collar`/`source_collar`는 실제 등록 좌표다. `qa/verify-neck-assembly.py`는 96개 기본/착용 변형에서 원본 셔츠 손실과 목 피부 위에 남은 옛 윤곽을 검사한다. 이 검사는 시각적 완성 판정을 대신하지 않는다.

```sh
rtk proxy python3 assets/characters/qa/verify-neck-assembly.py
```

미리보기의 비동기 이미지 로딩도 수정했다. 이전 렌더가 늦게 끝나면 다음 프레임 위에 그려지는 현상이 실제 브라우저에서 재현됐다. 이제 두 캐릭터를 화면 밖 캔버스에서 완성한 뒤 같은 렌더 번호일 때만 함께 표시한다. 프레임·장비·배경 조건도 렌더 시작 시 고정한다. `qa/verify-preview-race.mjs`는 로딩 순서를 의도적으로 뒤집어 오래된 렌더 폐기와 두 캐릭터 동시 표시를 검증한다.

이전 연결부 수정본은 `collar-v7`이며 `qa/neck-assembly-review.json`에 이전 오류와 정적/시각/브라우저 검수 범위를 기록한다. [지적 프레임 수정 전후](qa/neck-before-after.png), [전체 얼굴 연결부 48키](qa/neck-all-48-dark.png)에서 비교할 수 있다. 미리보기의 `?clip=quenching&frame=1&revision=collar-v7`처럼 특정 키를 정지 상태로 열 수 있다.

```sh
rtk proxy node assets/characters/qa/verify-preview-race.mjs
```

## 머리층 소유권 수정, head-ownership-v8

collar-v7도 타격 2·3·4키와 담금질에서 목·옷깃이 겹치고 옛 턱선이 돌출된다는 사용자 지적을 받았다. 기본 머리 파일의 짧은 목·옷깃 픽셀과 상체에 재삽입된 이전 얼굴 조각을 각각 제거했다. 밝은 귀·머리카락 한 픽셀을 셔츠로 오인하던 재질 마스크는 4픽셀 이상 연결된 셔츠 색 클러스터를 씨앗으로 사용한다. 한 픽셀 경계까지 원래 셔츠·옷깃을 보존하고 원래 목 피부에서 좁은 연결부를 구성한다. 바깥에 노출된 목 경계에만 기존 머리 윤곽색을 1px로 적용한다. 목 내부를 가로지르는 선은 만들지 않는다.

중립 기본층은 변경하지 않았다. 동작용 머리의 제외 좌표는 `source/animation/layer-ownership/assembly.mjs`와 각 성별 `animation.json`의 `head_ownership`에 기록한다. 남자 담금질 0·4·5키는 머리·눈·입을 함께 아래로 1 네이티브 픽셀 이동했다. 다른 키의 머리 등록은 유지했다. 얼굴과 눈·입의 남은 픽셀은 승인 원본과 동일하다.

`body/removed-old-face.png`와 `rendered/body/removed-old-face.png`는 제거 대상 진단층이며 합성하지 않는다. 현재 연결층의 피부는 해당 포즈의 네이티브 원본에서 가져오며 의상 경계와 좁은 목 다각형으로 제한한다. 예전 넓은 루트 목 피부 덧씌우기는 사용하지 않는다.

[지적 7키 전후 비교](qa/head-ownership-before-after.png), [머리·상체·목 단독 비교](qa/head-ownership-layers.png)를 함께 검수한다. `qa/head-ownership-validation.json`은 96개 머리 변형의 제외 좌표, 지적된 옛 귀/턱선 8점과 남자 담금질 이동을 검사한다. `qa/neck-assembly-validation.json`의 셔츠 검사는 같은 연결된 재질 클러스터를 대상으로 하며, 귀나 머리카락의 밝은 점을 셔츠 보존 대상으로 세지 않는다.

```sh
rtk proxy python3 assets/characters/qa/verify-head-ownership.py
rtk proxy python3 assets/characters/source/animation/layer-ownership/review.py
```

현재 검수 기록은 `qa/head-ownership-review.json`이다. collar-v7은 사용자 지적으로 반려된 기록으로 유지한다. 검수 통과와 사용자 시각 승인을 구분한다.

## 의상 재질과 수면 가림 수정, neck-cloth-v9

head-ownership-v8의 여성 열 작업과 리서치는 목 피부를 가장 가까운 피부색으로 채우며 어두운 의상 부분까지 침범했다. 이 범위의 48개 기본/착용 변형은 해당 자세의 같은 좌표에 실제 존재하는 피부만 연결층으로 사용한다. 원본 옷깃·어깨·앞치마 끈을 피부로 칠하지 않는다. 옷에 연결되지 않는 귀/얼굴의 작은 원본 조각은 목 연결층에서 제외한다. 다른 작업의 기존 연결 정책은 유지한다.

여성 담금질 2·3키는 머리를 3px 내렸다. 여성 포징 0·5키는 1px, 4키는 2px 내렸다. 여성 담금질 4키는 1px 내렸다. 여성 리서치 1·3키는 4px, 2키는 2px, 4키는 3px 내렸다. 남성 리서치 1·2·3키는 3px 내렸다. 눈·입은 승인된 픽셀을 유지하고 머리와 함께 이동한다. 조정량은 각 `head_identity`의 `anatomical_head_adjustment`에 기록한다. 이 좌표를 다른 캐릭터의 기본값으로 사용하지 않는다.

담금질의 실제 진입 경계는 통의 네이티브 y=18을 유지한다. 물 전체를 캐릭터 앞에 그리던 합성 순서가 조기 잘림의 원인이었다. 제작 씬과 HTML 미리보기 모두 물을 먼저 그린 다음 도구의 잠기는 부분만 가린 캐릭터를 그린다. 뒤쪽 수면(y<18) 위의 도구는 보이고 앞쪽 잠긴 부분은 가려진다. 통의 앞면은 마지막에 합성한다. 통·캐릭터의 위치나 접지점은 바꾸지 않았다.

`qa/verify-neck-material-water.py`는 해당 연결층 픽셀이 원본과 같은 좌표/색인지, 실제 씬에서 뒤쪽 물 위 도구가 보이는지 확인한다. `verify-neck-assembly.py`의 셔츠 보존 검사는 유지하되 목 피부 최소 3픽셀 조건은 제거했다. 숙여 목이 가려진 자세에 피부를 더 만들도록 유도하는 잘못된 조건이었다.

```sh
rtk proxy python3 assets/characters/qa/verify-neck-material-water.py
```

`neck-cloth-v9`는 사용자 검수에서 목·어깨 누락으로 반려되었다. 해당 보고서는 이전 기록이다. head-ownership-v8도 사용자 반려 기록으로 남긴다.


## 이전 상체 불투명 커버리지, solid-underlap-v10 (사용자 반려)

머리 제거 마스크가 목·옷깃 일부까지 삭제했고, 기존 검사는 남아 있는 의상/피부의 색만 확인해 투명 구멍을 놓쳤다. `source/animation/solid-underlap-v10/underlap.mjs`가 여성의 정면/우측 작업 키 24개, 착용·미착용 48종에 완결된 상체 겹침 영역을 추가한다. 기존 불투명 상체 픽셀과 머리·눈·입 변환은 유지한다. 남성, 공통 동작, 뒤돌아선 분해 동작은 이번 수정 대상이 아니다.

원본 의상은 같은 좌표의 RGBA로 복원한다. 머리 아래 목은 선언한 다각형 안에서 원본 피부 팔레트로 보충한다. 목·옷깃에 남은 닫힌 구멍은 가까운 원본 의상 재질로 연결한다. 새 연결부는 `body/underlap-*.png`, 필수 상체 영역은 `body/required-coverage.png`로 기록하며 착용 변형에도 대응 층을 저장한다. 기존 `neck-skin.png`와 `source_skin_only` 검사는 이전 단계만 검사하며 이 추가층 전체에 대한 동일좌표 피부 주장이 아니다.

`qa/verify-solid-underlap.py`는 고정 필수 커버리지, 상체 기존 픽셀 보존, 원본 상체가 불투명했던 닫힌 투명 구멍을 검사한다. 머리에 가려져도 상체층 누락이면 실패한다. 의도된 뒷머리 틈은 승인 원본의 네 좌표로 제한하고 실제 팔 사이 빈 공간은 유지한다. 반려된 v9 스냅샷이 이 검사에서 실패해야 하며 관련 없는 프레임 168개의 해시도 보존해야 한다.

[수정 전후](qa/solid-underlap-v10-before-after.png), [청록 배경 전체 48종](qa/solid-underlap-v10-all-48.png), [머리를 숨긴 상체](qa/solid-underlap-v10-body-layers.png), [픽셀 검사 결과](qa/solid-underlap-v10-validation.json)를 분리해 확인한다. 정적 검사 통과가 사용자 시각 승인을 의미하지 않는다.


## 현재 조립, joint-rebuild-v11

v10의 투명 구멍 개수 검사는 목 길이·옛 턱선·어깨 형상까지 검증하지 못했다. v10은 사용자 반려 기록이며 현재 시각 승인으로 인용하지 않는다.

현재 작업 키는 `joint-rebuild-v11/body.mjs`에서 원본 의상의 어깨와 짧은 목 연결부를 구성한다. 머리의 현재 알파·이동 위치를 입력받지 않는다. 옷의 연결부와 원본 그림자를 보존하고, 이전 머리 제거로 없어진 어깨 일부는 같은 원본 셔츠 재질로 복원한다. 목과 옷은 상체 소유이며, 머리층에는 승인된 얼굴·귀·머리카락·턱만 남긴다. `style-restoration/heads.mjs`의 부착점은 여성 `[62,66]`, 남성 `[57,64]`로 고정하고, 각 몸통의 목 위치에 정수 이동한다. 이전 키별 1~4px 추가 보정과 v10 삼각형 피부 연결은 현재 빌더 입력에서 제거했다.

[분리층과 반려본 비교](qa/joint-rebuild-v11-layers.png), [여성 전체 48종](qa/joint-rebuild-v11-female-48.png), [남성 전체 48종](qa/joint-rebuild-v11-male-48.png)를 확인한다. `verify-body-mount.py`는 머리를 숨긴 상태에서 몸통 필수 영역과 원본 의상 색, 일관된 머리 부착, 관련 없는 프레임 120개를 검사한다. `verify-animation.py`는 승인 얼굴·눈·입 원본과 아틀라스를 별도로 검사한다. v10의 `verify-solid-underlap.py`는 반려본을 기준으로 한 역사적 회귀 검사이며 현재 빌드의 통과 조건이 아니다.

현재 리서치는 사용자가 승인한 3/4 얼굴을 정면 몸통에 재사용한다. 이번 수정은 목·어깨 조립 수정이며 새로운 정면 얼굴 제작이나 게임 통합은 포함하지 않는다.

## Research layer finish v12

Research-only correction of the male left hanging cloth and desktop front-pass leak, female synthesized shoulder ledges and pigment discontinuity, and the female thinking key resting hand. Reuse the complete generated key-0 resting forearm and fingers at native coordinates, preserving the thinking sleeve/cuff. Each equipment variant owns its desktop arm mask. The intentional male cloth removal is recorded in `body/removed-research-prop.png` diagnostics; source garment checks allow only that explicit male equipped research region (x38–53, y93–109). Approved facial layers and 216 other action frames remain protected by exact hashes. See `source/animation/research-finish-v12/README.md` and `qa/verify-research-finish.py`. User acceptance remains pending.

## Female research garment materials v13

The user rejected v12: white-plate opacity did not prove leather strap continuity or a clean neckline. Female equipped research now restores explicit native source leather strips and buckles, places the source collar around a short neck without moving the head, and removes six old-source hair pixels at the thinking shoulder. Previous source-face jaw edges are excluded from the transferred collar. See `source/animation/research-material-v13/README.md` and `qa/verify-research-material.py`. Female base, approved facial layers and all other frames retain hash checks. User acceptance remains unset.

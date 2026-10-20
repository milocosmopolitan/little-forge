//! Korean / English strings for the kit-v2 management screens (Orders, Weapon Collection).
//! Korean is the default; `LITTLE_FORGE_LANG=en` switches to English. Other screens move here later.
use bevy::prelude::*;

#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Lang {
    #[default]
    Ko,
    En,
}
impl Lang {
    pub fn from_env() -> Self {
        match std::env::var("LITTLE_FORGE_LANG").as_deref() {
            Ok("en") => Lang::En,
            _ => Lang::Ko,
        }
    }
}

/// A string in both languages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct T(pub &'static str, pub &'static str);
impl T {
    pub fn get(self, l: Lang) -> &'static str {
        match l {
            Lang::Ko => self.0,
            Lang::En => self.1,
        }
    }
}

// Host navigation
pub const ORDERS: T = T("의뢰", "Orders");
pub const COLLECTION: T = T("무기도감", "Weapon Collection");
pub const BACK: T = T("돌아가기", "Back");

// Orders rail and list
pub const ALL: T = T("전체", "All");
pub const ACTIVE: T = T("진행 중", "In progress");
pub const DONE: T = T("완료 기록", "Completed");
pub const ORDER_LIST: T = T("의뢰 목록", "Orders");
pub const DAILY: T = T("일상 의뢰", "Daily");
pub const NO_ORDERS: T = T("이 분류에 의뢰가 없습니다.", "No orders here.");
pub const DEADLINE: T = T("납품 기한", "Deadline");
pub const REWARD: T = T("보상 금화", "Reward");
pub const NEEDS: T = T("필요 재료", "Materials");
pub const WORK: T = T("작업 시간", "Work");
pub const STATUS: T = T("상태", "Status");
pub const IRON_ORE: T = T("철광석", "Iron ore");

pub const CHEF: T = T("요리사", "Chef");
pub const FARMER: T = T("농부", "Farmer");
pub const CHEF_TITLE: T = T("요리사의 식칼", "The Chef's knife");
pub const FARMER_TITLE: T = T("농부의 낫 수리", "The farmer's sickle");
pub const CHEF_ITEM: T = T("식칼", "Kitchen knife");
pub const FARMER_ITEM: T = T("낡은 낫", "Worn sickle");
pub const CHEF_QUOTE: T = T(
    "“주방에서 쓸 균형 잡힌 칼이 필요하네. 자네 솜씨를 한번 보고 싶군.”",
    "“I need a balanced blade for the kitchen. Let me see your work.”",
);
pub const FARMER_QUOTE: T = T(
    "“낫의 날이 다 무뎌졌어요. 밭일을 할 수 있게 손봐 주실래요?”",
    "“My sickle has lost its edge. Can you get it ready for the fields?”",
);

// Phases
pub fn phase(l: Lang, p: forge_core::commission::OrderPhase) -> &'static str {
    use forge_core::commission::OrderPhase::*;
    match p {
        Offered => T("새 의뢰", "New").get(l),
        Accepted => T("수락함", "Accepted").get(l),
        Queued => T("작업 대기", "Queued").get(l),
        Working => T("작업 중", "In progress").get(l),
        Ready => T("납품 가능", "Ready").get(l),
        Delivered => T("완료", "Delivered").get(l),
    }
}

// Order actions
pub const ACCEPT: T = T("의뢰 수락", "Accept");
pub const QUEUE: T = T("작업 등록", "Queue work");
pub const DELIVER: T = T("납품", "Deliver");
pub const BUY_ORE: T = T("광석 구매 · 4G", "Buy ore · 4g");
pub const AT_BENCH: T = T(
    "작업대에서 만드는 중입니다. 휴식하면 멈췄다가 다시 이어집니다.",
    "At the workbench. Resting pauses work, then it resumes.",
);
pub const IN_QUEUE: T = T(
    "작업 대기열에 있습니다. 차례가 되면 자동으로 시작합니다.",
    "In the work queue. It starts automatically.",
);
pub const THANKS: T = T("납품을 마쳤습니다.", "Delivered.");

pub fn gold(l: Lang, g: u64) -> String {
    match l {
        Lang::Ko => format!("{g} 금화"),
        Lang::En => format!("{g} gold"),
    }
}
pub fn favor(l: Lang, f: u8) -> String {
    match l {
        Lang::Ko => format!("호감 +{f}"),
        Lang::En => format!("Favor +{f}"),
    }
}
pub fn seconds(l: Lang, s: u64) -> String {
    let (h, m, sec) = (s / 3600, s / 60 % 60, s % 60);
    match l {
        Lang::Ko if h > 0 => format!("{h}시간 {m}분"),
        Lang::Ko if m > 0 => format!("{m}분 {sec}초"),
        Lang::Ko => format!("{sec}초"),
        Lang::En if h > 0 => format!("{h}h {m}m"),
        Lang::En if m > 0 => format!("{m}m {sec}s"),
        Lang::En => format!("{sec}s"),
    }
}
pub fn left(l: Lang, s: u64) -> String {
    match l {
        Lang::Ko => format!("{} 남음", seconds(l, s)),
        Lang::En => format!("{} left", seconds(l, s)),
    }
}
pub const NO_DEADLINE: T = T("기한 없음", "No deadline");
pub const OVERDUE: T = T("기한 초과", "Overdue");
pub const ON_TIME: T = T("기한 내 완료", "Finished on time");
pub const LATE: T = T("늦게 완료", "Finished late");
pub fn after_accept(l: Lang, s: u64) -> String {
    match l {
        Lang::Ko => format!("수락 후 {}", seconds(l, s)),
        Lang::En => format!("{} after accepting", seconds(l, s)),
    }
}
pub fn have_need(l: Lang, have: u32, need: u32) -> String {
    match l {
        Lang::Ko => format!("보유 {have} / 필요 {need}"),
        Lang::En => format!("{have} / {need}"),
    }
}

pub fn error(l: Lang, e: forge_core::Error) -> &'static str {
    use forge_core::Error::*;
    match e {
        InsufficientResources => {
            T("재료나 금화가 부족합니다.", "Not enough materials or gold.").get(l)
        }
        Busy => T(
            "작업대가 바쁩니다. 자리가 나면 다시 시도하세요.",
            "The workbench is busy.",
        )
        .get(l),
        DisplayFull => T("진열대에 빈 자리가 없습니다.", "No free display slot.").get(l),
        ResearchLocked => T("연구가 필요합니다.", "Research required.").get(l),
        _ => T("지금은 할 수 없습니다.", "Not available now.").get(l),
    }
}

// Collection
pub const ALL_WEAPONS: T = T("전체 무기", "All weapons");
pub const COLLECTION_HINT: T = T(
    "만들었거나 손에 넣은 무기 기록",
    "Weapons you have made or found",
);
pub const PROGRESS: T = T("수집 현황", "Found");
pub const UNKNOWN: T = T("???", "???");
pub const UNDISCOVERED: T = T(
    "아직 만들어 본 적 없는 무기입니다.",
    "You have not made this weapon yet.",
);
pub const NOT_IN_GAME: T = T(
    "제작법이 아직 알려지지 않았습니다.",
    "Its recipe is not known yet.",
);
pub const OWNED: T = T("보유", "Owned");
pub const MADE: T = T("제작", "Made");
pub const SOLD: T = T("판매", "Sold");
pub const BEST_QUALITY: T = T("최고 품질", "Best quality");
pub const FIELD_USES: T = T("출정 사용", "Expeditions");
pub const KILLS: T = T("처치", "Defeated");
pub const BEST_WAVE: T = T("최고 웨이브", "Best wave");
pub const ORE_USED: T = T("사용 광석", "Ore used");
pub const SOLD_NOTE: T = T(
    "판매한 무기의 개별 기록은 아직 남지 않습니다.",
    "Records of sold weapons are not kept yet.",
);
pub fn family(l: Lang, f: &str) -> &'static str {
    match f {
        "dagger" => T("단검", "Daggers").get(l),
        "axe" => T("도끼", "Axes").get(l),
        "polearm" => T("창", "Polearms").get(l),
        "blunt" => T("둔기", "Blunt").get(l),
        "shield" => T("방패", "Shields").get(l),
        "bow" => T("활", "Bows").get(l),
        _ => T("기타", "Other").get(l),
    }
}
pub fn weapon_name(l: Lang, id: &str) -> &'static str {
    match id {
        "dagger" => T("철 단도", "Iron dagger").get(l),
        "hunting_knife" => T("사냥칼", "Hunting knife").get(l),
        "hand_axe" => T("손도끼", "Hand axe").get(l),
        "spear" => T("창", "Spear").get(l),
        "pike" => T("파이크", "Pike").get(l),
        "halberd" => T("할버드", "Halberd").get(l),
        "wooden_club" => T("몽둥이", "Wooden club").get(l),
        "wooden_staff" => T("나무 지팡이", "Wooden staff").get(l),
        "wooden_shield" => T("나무 방패", "Wooden shield").get(l),
        "shortbow" => T("단궁", "Shortbow").get(l),
        _ => "?",
    }
}
pub fn weapon_blurb(l: Lang, id: &str) -> &'static str {
    match id {
        "dagger" => T(
            "양날에 작은 가드가 달린 짧은 칼. 대장간의 첫 무기.",
            "A short double-edged blade with a small guard. The forge's first weapon.",
        )
        .get(l),
        "hunting_knife" => T(
            "클립 포인트 외날에 손잡이를 감은 사냥용 칼.",
            "A single-edged clip-point blade with a wrapped grip.",
        )
        .get(l),
        "shortbow" => T(
            "사냥꾼이 숲에서 쓰는 짧은 활.",
            "A short bow the hunter carries into the forest.",
        )
        .get(l),
        _ => "",
    }
}
pub fn appearance(l: Lang, a: forge_core::weapon::Appearance) -> &'static str {
    use forge_core::weapon::Appearance::*;
    match a {
        Common => T("일반", "Common").get(l),
        Refined => T("정교", "Refined").get(l),
        Masterwork => T("걸작", "Masterwork").get(l),
    }
}

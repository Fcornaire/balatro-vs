use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum AreaType {
    #[default]
    Consumeables,
    PackCards,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum CardType {
    #[default]
    Card,
    Joker,
    Consumeable,
    Booster,
    Voucher,
}

impl CardType {
    pub fn lua_name(&self) -> &'static str {
        match self {
            CardType::Joker => "joker",
            CardType::Card => "card",
            CardType::Consumeable => "consumeable",
            CardType::Voucher => "voucher",
            CardType::Booster => "booster",
        }
    }

    pub fn from_lua_name(name: &str) -> Option<Self> {
        Some(match name {
            "joker" => CardType::Joker,
            "card" => CardType::Card,
            "consumeable" => CardType::Consumeable,
            "voucher" => CardType::Voucher,
            "booster" => CardType::Booster,
            _ => return None,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct CardConf {
    type_: CardType,
    label: String,
    location: String,
    stay_flipped: bool,
    edition: String,
    center: String,
    card: String,
    center_key: String,
    ability: String,
    versus_center_id: u32,
}

impl CardConf {
    pub fn get_type(&self) -> CardType {
        self.type_.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct LuaCardConf {
    pub label: String,
    pub type_: String,
    pub center: String,
    pub card: String,
    pub center_key: String,
    pub versus_center_id: u32,
    pub ability: String,
    pub edition: String,
    pub location: String,
    pub stay_flipped: bool,
}

impl TryFrom<LuaCardConf> for CardConf {
    type Error = String;

    fn try_from(t: LuaCardConf) -> Result<Self, String> {
        let type_ = CardType::from_lua_name(&t.type_)
            .ok_or_else(|| format!("Unknown card type: '{}'", t.type_))?;

        let (location, stay_flipped) = match type_ {
            CardType::Joker => (t.location, t.stay_flipped),
            _ => (String::new(), false),
        };

        let edition = match type_ {
            CardType::Joker | CardType::Card => t.edition,
            _ => String::new(),
        };

        Ok(CardConf {
            type_,
            label: t.label,
            location,
            stay_flipped,
            edition,
            center: t.center,
            card: t.card,
            center_key: t.center_key,
            ability: t.ability,
            versus_center_id: t.versus_center_id,
        })
    }
}

impl From<&CardConf> for LuaCardConf {
    fn from(c: &CardConf) -> Self {
        LuaCardConf {
            label: c.label.clone(),
            type_: c.type_.lua_name().to_string(),
            center: c.center.clone(),
            card: c.card.clone(),
            center_key: c.center_key.clone(),
            versus_center_id: c.versus_center_id,
            ability: c.ability.clone(),
            edition: c.edition.clone(),
            location: c.location.clone(),
            stay_flipped: c.stay_flipped,
        }
    }
}

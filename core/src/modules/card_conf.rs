use mlua::{IntoLua, Lua, Result, Table};
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

impl From<Table> for CardConf {
    fn from(table: Table) -> Self {
        let label = table.get::<String>("label".to_string()).unwrap();
        let type_ = table.get::<String>("type_".to_string()).unwrap();
        let center = table.get::<String>("center".to_string()).unwrap();
        let card = table.get::<String>("card".to_string()).unwrap();
        let center_key = table
            .get::<String>("center_key".to_string())
            .unwrap_or_default();
        let versus_center_id = table
            .get::<u32>("versus_center_id".to_string())
            .unwrap_or_default();
        let ability = table
            .get::<String>("ability".to_string())
            .unwrap_or_default();

        match type_.as_str() {
            "joker" => {
                let location = table
                    .get::<String>("location".to_string())
                    .unwrap_or_default();
                let stay_flipped = table
                    .get::<mlua::Value>("stay_flipped".to_string())
                    .unwrap_or(mlua::Value::Boolean(false))
                    .as_boolean()
                    .unwrap_or(false);
                let edition = table
                    .get::<String>("edition".to_string())
                    .unwrap_or_default();

                return CardConf {
                    type_: CardType::Joker,
                    label,
                    location,
                    stay_flipped,
                    edition,
                    center,
                    card,
                    center_key,
                    ability,
                    versus_center_id,
                };
            }
            "card" => {
                return CardConf {
                    type_: CardType::Card,
                    label,
                    location: "".to_string(),
                    stay_flipped: false,
                    edition: "".to_string(),
                    center,
                    card,
                    center_key,
                    ability,
                    versus_center_id,
                };
            }
            "consumeable" => {
                return CardConf {
                    type_: CardType::Consumeable,
                    label,
                    location: "".to_string(),
                    stay_flipped: false,
                    edition: "".to_string(),
                    center,
                    card,
                    center_key,
                    ability,
                    versus_center_id,
                };
            }
            "voucher" => {
                return CardConf {
                    type_: CardType::Voucher,
                    label,
                    location: "".to_string(),
                    stay_flipped: false,
                    edition: "".to_string(),
                    center,
                    card,
                    center_key,
                    ability,
                    versus_center_id,
                };
            }
            "booster" => {
                return CardConf {
                    type_: CardType::Booster,
                    label,
                    location: "".to_string(),
                    stay_flipped: false,
                    edition: "".to_string(),
                    center,
                    card,
                    center_key,
                    ability,
                    versus_center_id,
                };
            }
            _ => {
                panic!("Unknown card type: {}", type_);
            }
        }
    }
}

impl CardConf {
    pub fn to_table(&self, lua: &Lua) -> Result<Table> {
        let table = lua.create_table()?;

        table.set("label", self.label.clone())?;
        table.set("location", self.location.clone())?;
        table.set("stay_flipped", self.stay_flipped)?;
        table.set("edition", self.edition.clone())?;
        table.set("center", self.center.clone())?;
        table.set("card", self.card.clone())?;
        table.set("center_key", self.center_key.clone())?;
        table.set("ability", self.ability.clone())?;
        table.set("versus_center_id", self.versus_center_id)?;

        table.set(
            "type_",
            match self.type_ {
                CardType::Joker => "joker",
                CardType::Card => "card",
                CardType::Consumeable => "consumeable",
                CardType::Voucher => "voucher",
                CardType::Booster => "booster",
            },
        )?;

        Ok(table)
    }
}

impl IntoLua for CardConf {
    fn into_lua(self, lua: &Lua) -> Result<mlua::Value> {
        self.to_table(lua)?.into_lua(lua)
    }
}

use std::{collections::HashMap, str::FromStr};

use itertools::Itertools;
use rlua::{Context, Error as LuaError, FromLua, Table, Value};

/* Data */

#[derive(Debug)]
pub struct Data {
    pub items: HashMap<ItemId, Item>,
    pub recipes: HashMap<RecipeId, Recipe>,

    pub as_input: HashMap<ItemId, Vec<RecipeId>>,
    pub as_output: HashMap<ItemId, Vec<RecipeId>>,

    pub item_by_name: HashMap<String, ItemId>,
    pub recipes_by_name: HashMap<String, RecipeId>,
}

impl<'lua> FromLua<'lua> for Data {
    fn from_lua(value: Value<'lua>, _lua: Context<'lua>) -> Result<Self, LuaError> {
        match value {
            Value::Table(table) => {
                let items = table.get::<_, HashMap<ItemId, Item>>("game_items")?;
                let recipes = table
                    .get::<_, Vec<RecipeTuple>>("game_recipes")?
                    .into_iter()
                    .map(|RecipeTuple(id, recipe)| (id, recipe))
                    .collect::<HashMap<RecipeId, Recipe>>();

                let mut as_input = HashMap::<ItemId, Vec<RecipeId>>::new();
                for (iid, rid) in recipes
                    .iter()
                    .flat_map(|(rid, r)| r.inputs.iter().map(|i| (i.id, *rid)))
                {
                    as_input.entry(iid).or_default().push(rid);
                }

                let mut as_output = HashMap::<ItemId, Vec<RecipeId>>::new();
                for (iid, rid) in recipes
                    .iter()
                    .flat_map(|(rid, r)| r.outputs.iter().map(|i| (i.id, *rid)))
                {
                    as_output.entry(iid).or_default().push(rid);
                }

                let item_by_name = items
                    .iter()
                    .map(|(iid, i)| (i.name.clone(), *iid))
                    .collect();

                let recipes_by_name = recipes
                    .iter()
                    .map(|(rid, r)| (r.name.clone(), *rid))
                    .collect();

                Ok(Self {
                    items,
                    recipes,
                    as_input,
                    as_output,
                    item_by_name,
                    recipes_by_name,
                })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Data",
                to: "Data",
                message: None,
            }),
        }
    }
}

/* ItemId */

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ItemId(pub usize);

impl FromStr for ItemId {
    type Err = <usize as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(usize::from_str(s)?))
    }
}

impl<'lua> FromLua<'lua> for ItemId {
    fn from_lua(value: Value<'lua>, lua: Context<'lua>) -> Result<Self, LuaError> {
        Ok(Self(usize::from_lua(value, lua)?))
    }
}

/* Item */

#[derive(Debug)]
pub struct Item {
    pub name: String,
    pub type_: ItemType,
}

impl<'lua> FromLua<'lua> for Item {
    fn from_lua(value: Value<'lua>, _lua: Context<'lua>) -> Result<Self, LuaError> {
        match value {
            Value::Table(table) => {
                let name = table.get("name")?;
                let type_ = table.get("type")?;

                Ok(Self { name, type_ })
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "Item",
                to: "Item",
                message: None,
            }),
        }
    }
}

/* ItemType */

#[derive(Debug, Eq, PartialEq)]
pub enum ItemType {
    Material,
    Matrix,
    Product,
    Production,
    Resource,
    Component,
    Logistics,
    Unknown(String),
}

impl<'lua> FromLua<'lua> for ItemType {
    fn from_lua(value: Value<'lua>, _lua: Context<'lua>) -> Result<Self, LuaError> {
        match value {
            Value::String(s) => Ok(match s.to_str()? {
                "MATERIAL" => Self::Material,
                "MATRIX" => Self::Matrix,
                "PRODUCT" => Self::Product,
                "PRODUCTION" => Self::Production,
                "RESOURCE" => Self::Resource,
                "COMPONENT" => Self::Component,
                "LOGISTICS" => Self::Logistics,
                s => Self::Unknown(s.into()),
            }),
            _ => Err(LuaError::FromLuaConversionError {
                from: "ItemType",
                to: "ItemType",
                message: None,
            }),
        }
    }
}

/* RecipeTuple */

pub struct RecipeTuple(pub RecipeId, pub Recipe);

impl<'lua> FromLua<'lua> for RecipeTuple {
    fn from_lua(value: Value<'lua>, _lua: Context<'lua>) -> Result<Self, LuaError> {
        match value {
            Value::Table(table) => {
                let id = table.get("id")?;

                let name = table.get("name")?;
                let type_ = table.get("type")?;
                let seconds = table.get("seconds")?;
                let explicit = table.get("explicit").unwrap_or(false);
                let inputs = table
                    .get::<_, Table>("inputs")?
                    .sequence_values::<usize>()
                    .tuples::<(_, _)>()
                    .map(ItemAmount::from_tuple)
                    .collect::<Result<_, _>>()?;
                let outputs = table
                    .get::<_, Table>("outputs")?
                    .sequence_values::<usize>()
                    .tuples::<(_, _)>()
                    .map(ItemAmount::from_tuple)
                    .collect::<Result<_, _>>()?;

                Ok(Self(
                    id,
                    Recipe {
                        name,
                        type_,
                        seconds,
                        explicit,
                        inputs,
                        outputs,
                    },
                ))
            }
            _ => Err(LuaError::FromLuaConversionError {
                from: "ItemType",
                to: "ItemType",
                message: None,
            }),
        }
    }
}

/* RecipeId */

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RecipeId(pub usize);

impl FromStr for RecipeId {
    type Err = <usize as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(usize::from_str(s)?))
    }
}

impl<'lua> FromLua<'lua> for RecipeId {
    fn from_lua(value: Value<'lua>, lua: Context<'lua>) -> Result<Self, LuaError> {
        Ok(Self(usize::from_lua(value, lua)?))
    }
}

/* Recipe */

#[derive(Debug)]
pub struct Recipe {
    pub name: String,
    pub type_: RecipeType,
    pub seconds: f64,
    pub explicit: bool,
    pub inputs: Vec<ItemAmount>,
    pub outputs: Vec<ItemAmount>,
}

/* ItemAmount */

#[derive(Debug)]
pub struct ItemAmount {
    pub id: ItemId,
    pub amount: usize,
}

impl ItemAmount {
    fn from_tuple(
        (id, amount): (Result<usize, LuaError>, Result<usize, LuaError>),
    ) -> Result<Self, LuaError> {
        Ok(Self {
            id: ItemId(id?),
            amount: amount?,
        })
    }
}

/* RecipeType */

#[derive(Debug)]
pub enum RecipeType {
    Assemble,
    Chemical,
    Fractionate,
    Particle,
    Refine,
    Research,
    Smelt,
    Unknown(String),
}

impl<'lua> FromLua<'lua> for RecipeType {
    fn from_lua(value: Value<'lua>, _lua: Context<'lua>) -> Result<Self, LuaError> {
        match value {
            Value::String(s) => Ok(match s.to_str()? {
                "ASSEMBLE" => Self::Assemble,
                "CHEMICAL" => Self::Chemical,
                "FRACTIONATE" => Self::Fractionate,
                "PARTICLE" => Self::Particle,
                "REFINE" => Self::Refine,
                "RESEARCH" => Self::Research,
                "SMELT" => Self::Smelt,
                s => Self::Unknown(dbg!(s.into())),
            }),
            _ => Err(LuaError::FromLuaConversionError {
                from: "ItemType",
                to: "ItemType",
                message: None,
            }),
        }
    }
}

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::{fs::read_to_string, str::FromStr};

use rlua::Lua;
use structopt::StructOpt;

use crate::data::{ItemType, RecipeId};
use crate::{
    data::{Data, ItemId},
    error::Error,
};

#[derive(Debug, StructOpt)]
pub struct CreateProductionGraph {
    /// Products to include in the graph.
    #[structopt(short = "i", long = "items")]
    pub items: Vec<String>,

    /// Recipes to exclude from the graph.
    #[structopt(long = "ignore")]
    pub ignore: Vec<String>,

    /// Resolve the dependencies of the products.
    #[structopt(short = "r", long = "resolve-deps")]
    pub resolve_deps: bool,

    /// File to load the product data and recipes from.
    #[structopt(short = "d", long = "data", default_value = "data.lua")]
    pub data_path: PathBuf,
}

impl CreateProductionGraph {
    pub fn exec(self) -> Result<(), Error> {
        let Self {
            items,
            ignore,
            resolve_deps,
            data_path,
        } = self;

        log::info!("Load data from {:#?}", &data_path);
        let data = read_to_string(data_path)?;
        let lua = Lua::new();
        let data = lua.context(move |lua| lua.load(&data).eval::<Data>())?;

        log::info!("  loaded {} items", data.items.len());
        log::info!("  loaded {} recipes", data.recipes.len());

        log::info!("Parse items");
        let mut items = parse_ids(&data, &items, true)?
            .into_iter()
            .map(ItemId)
            .collect::<BTreeSet<_>>();
        log::info!("  loaded {} items", items.len());

        log::info!("Parse ignored recipes");
        let ignore = parse_ids(&data, &ignore, false)?
            .into_iter()
            .collect::<BTreeSet<_>>();
        log::info!("  loaded {} ignored recipes", ignore.len());

        log::info!("Resolve recipes");
        let mut recipes = BTreeSet::<RecipeId>::new();
        for item in items.clone() {
            resolve_item_dependencies(&data, &mut recipes, &mut items, &ignore, item, resolve_deps);
        }
        log::info!("  use {} items", items.len());
        log::info!("  use {} recipes", recipes.len());

        log::info!("Generate graph");

        println!("strict digraph DSP {{");
        println!("    graph [ rankdir=LR ]");

        println!();
        println!("    /* Recipes */");

        for rid in &recipes {
            if let Some(recipe) = data.recipes.get(rid) {
                println!();
                println!("    /* {} */", recipe.name);
                println!(
                    "    \"{}\" [ label=\"{}\" shape=point width=0.1 ]",
                    rid.0, recipe.seconds
                );

                for i in &recipe.inputs {
                    if let Some(item) = data.items.get(&i.id) {
                        println!(
                            "    \"{}\" -> \"{}\" [ name=\"{}\" ]",
                            item.name, rid.0, i.amount
                        );
                    }
                }

                for o in &recipe.outputs {
                    if let Some(item) = data.items.get(&o.id) {
                        println!(
                            "    \"{}\" -> \"{}\" [ name=\"{}\" ]",
                            rid.0, item.name, o.amount
                        );
                    }
                }
            }
        }

        println!("}}");

        Ok(())
    }
}

fn parse_ids(data: &Data, items: &[String], items_only: bool) -> Result<Vec<usize>, Error> {
    let mut ret = Vec::<usize>::new();

    for item in items {
        if let Ok(id) = usize::from_str(item) {
            ret.push(id);
        } else if let Some(id) = data.item_by_name.get(item) {
            ret.push(id.0);
        } else if let Some(id) = data.recipes_by_name.get(item) {
            if !items_only {
                ret.push(id.0);
            }
        } else {
            let item_type = match item.to_lowercase().as_str() {
                "all" => {
                    ret.extend(data.items.keys().map(|id| id.0));

                    if !items_only {
                        ret.extend(data.recipes.keys().map(|id| id.0));
                    }

                    continue;
                }
                "explicit" if !items_only => {
                    ret.extend(data.recipes.iter().filter_map(|(rid, r)| {
                        if r.explicit {
                            Some(rid.0)
                        } else {
                            None
                        }
                    }));

                    continue;
                }
                "advanced" if !items_only => {
                    ret.extend(
                        ADVANCED_RECIPES
                            .iter()
                            .filter_map(|name| data.recipes_by_name.get(*name).map(|rid| rid.0)),
                    );

                    continue;
                }
                "material" => ItemType::Material,
                "matrix" => ItemType::Matrix,
                "product" => ItemType::Product,
                "production" => ItemType::Production,
                "resource" => ItemType::Resource,
                "component" => ItemType::Component,
                "logistics" => ItemType::Logistics,
                s => {
                    return Err(Error::custom(format!("Invalid or unknown item: {}", s)));
                }
            };

            ret.extend(data.items.iter().filter_map(|(iid, i)| {
                if i.type_ == item_type {
                    Some(iid.0)
                } else {
                    None
                }
            }));
        }
    }

    Ok(ret)
}

fn resolve_item_dependencies(
    data: &Data,
    recipes: &mut BTreeSet<RecipeId>,
    items: &mut BTreeSet<ItemId>,
    exclude: &BTreeSet<usize>,
    iid: ItemId,
    resolve_deps: bool,
) {
    if let Some(rids) = data.as_output.get(&iid) {
        for rid in rids {
            if !exclude.contains(&rid.0) && recipes.insert(*rid) {
                if let Some(r) = data.recipes.get(rid) {
                    for input in &r.inputs {
                        let iid = input.id;
                        if !exclude.contains(&iid.0) {
                            items.insert(iid);

                            if resolve_deps {
                                resolve_item_dependencies(data, recipes, items, exclude, iid, true);
                            }
                        }
                    }
                }
            }
        }
    }
}

const ADVANCED_RECIPES: &[&str] = &[
    "Casimir Crystal (Advanced)",
    "Organic Crystal (Original)",
    "Crystal Silicon (Advanced)",
    "Photon Combiner (Advanced)",
    "Space Warper (Advanced)",
    "Particle Container (Advanced)",
    "Graphene (Advanced)",
    "Carbon Nanotube (Advanced)",
    "Diamond (Advanced)",
];

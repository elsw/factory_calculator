mod recipe;
mod menu;
mod flowchart;

use std::{fs, println};

use serde::Deserialize;

use crate::recipe::{RecipeCollection, parse_file};

#[derive(Debug, Clone,Deserialize)]
pub struct GlobalSettings {
    game_version: String,
    use_blueprints: bool,
    recipe_blacklist: Vec<String>
}

fn init_settings(path: &str) -> GlobalSettings {
    let content: String = match std::fs::read_to_string(path) {
		Ok(c) => c,
		Err(_e) => {
			println!("Failed to read file {}",path);
			std::process::exit(1);
		}
	};
    let global_settings: GlobalSettings = match toml::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            println!("Unable to deserialise {}, check the toml is valid: {}",path,e);
            std::process::exit(1);
        }
    };
    global_settings
}

fn init_recipe_library(settings: &GlobalSettings) -> RecipeCollection {
    println!("Initialising Recipe Library");
    let folder = format!("recipes/{}",&settings.game_version);
    let paths = match fs::read_dir(folder){
        Ok(p) => p,
        Err(e) => {
            println!("Failed to read folder: {}",e);
			std::process::exit(1);
        }
    };

    let mut recipes = RecipeCollection::new();
    for path in paths {
        //Generate path to config file
        let path_buf = path.expect("invalid path").path();
        let path_str = path_buf.to_str().expect("invalid_string");

        //Read in recipes
        let read_in = parse_file(&path_str,settings);
        recipes.append(read_in);
    }

    println!("Read in {} recipes",recipes.recipes.len());

    // for (key, value) in recipes.output_lookup.iter() {
    //     if value.len() > 1 {
    //         let mut list_str: String = String::new();
    //         for v in value {
    //             list_str += v.as_str();
    //             list_str += ",";
    //         }
    //         println!("Warning: Multiple Recipes create {}, ({})",key,list_str);
    //     }
    // }
    recipes
}

fn main() {
    let settings = init_settings("settings.toml");
    let recipes = init_recipe_library(&settings);
    menu::run(&recipes);
    flowchart::plot_flowchart();
}

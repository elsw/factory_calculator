mod recipe;

use std::{fs, println};

use crate::recipe::{RecipeCollection, parse_file};

fn init_recipe_library() -> RecipeCollection {
    println!("Initialising Recipe Library");
    let paths = match fs::read_dir("recipes/satisfactory_plus/"){
        Ok(p) => p,
        Err(e) => {
            println!("Failed to read folder: {}",e);
			std::process::exit(1);
        }
    };

    let mut recipes = RecipeCollection::new();
    for path in paths {
        let path_buf = path.expect("invalid path").path();
        let path_str = path_buf.to_str().expect("invalid_string");
        let read_in = parse_file(&path_str);
        recipes.append(read_in);
    }

    println!("Read in {} recipes",recipes.recipes.len());

    for (key, value) in recipes.output_lookup.iter() {
        if value.len() > 1 {
            let mut list_str: String = String::new();
            for v in value {
                list_str += v.as_str();
                list_str += ",";
            }
            println!("Warning: Multiple Recipes create {}, ({})",key,list_str);
        }
    }
    recipes
}

fn main() {
    let recipes = init_recipe_library();
}

use std::println;

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Recipe {
    output: String,
	output_rates: Vec<f32>,
	byproducts: Vec<String>,
    inputs: Vec<String>,
	input_rates: Vec<f32>,
	produced_in: Vec<String>
}

impl Recipe {
    pub fn new() -> Self {
		let output: String = String::new();
		let output_rates: Vec<f32> = Vec::new();
		let byproducts: Vec<String> = Vec::new();
		let inputs: Vec<String> = Vec::new();
		let input_rates: Vec<f32> = Vec::new();
		let produced_in: Vec<String> = Vec::new();
        Self { output, output_rates, byproducts, inputs, input_rates, produced_in }
    }

	// Apply any matching fields over the top of the current values
	pub fn import(&mut self,name: &String,toml_values: toml::Table) {
		//default output to it's own name
		self.output = name.clone();
		for (key, value) in toml_values.into_iter() {
			match key.as_str() {
				"output" => self.output = value.try_into().expect("Error setting output"),
				"output_rates" => self.output_rates = value.try_into().expect("Error setting output_rates"),
				"byproducts" => self.byproducts = value.try_into().expect("Error setting byproducts"),
				"inputs" => self.inputs = value.try_into().expect("Error setting inputs"),
				"input_rates" => self.input_rates = value.try_into().expect("Error setting inputs_rates"),
				"produced_in" => self.produced_in = value.try_into().expect("Error setting produced_by"),
				_ => continue
			}
		}
	}
}

pub struct RecipeCollection {
	// A map of all the recipes. <Recipe Name,Recipe Struct>
	pub recipes: HashMap<String,Recipe>,
	// Lookup map of outputs, and which recipes have them as an output
	pub output_lookup: HashMap<String,Vec<String>>
}

impl RecipeCollection {
    pub fn new() -> Self {
        let recipes: HashMap<String,Recipe> = HashMap::new();
		let output_lookup: HashMap<String,Vec<String>> = HashMap::new();
        Self { recipes, output_lookup }
    }
	pub fn append(&mut self,other: RecipeCollection) {
		self.recipes.extend(other.recipes);
		self.output_lookup.extend(other.output_lookup);
	}
}

pub fn parse_file(path: &str) -> RecipeCollection  {
	let mut recipe_collection: RecipeCollection = RecipeCollection::new();

	let content: String = match std::fs::read_to_string(path) {
		Ok(c) => c,
		Err(_e) => {
			println!("Failed to read file {}",path);
			std::process::exit(1);
		}
	};

	let values : toml::Table = match toml::from_str(&content) {
		Ok(v) => v,
		Err(_e) => {
			println!("Failed to parse file {}, check the .toml is valid",path);
			std::process::exit(1);
		}
	};

	//Load the defaults
	let mut default: Recipe = Recipe::new();
	if let Some(d) = values.get("default") {
		default.import(&String::from("default"),d.as_table().expect("not a table").clone());
	}
	
	for (key, value) in values.into_iter() {
		if key == "default" {
			continue;
		} 

		let mut recipe: Recipe = default.clone();
		recipe.import(&key,value.as_table().expect("not a table").clone());
		
		if !recipe_collection.output_lookup.contains_key(&recipe.output) {
			recipe_collection.output_lookup.insert(recipe.output.clone(), Vec::new());
		}
		recipe_collection.output_lookup.get_mut(&recipe.output).expect("Interal logic error").push(key.clone());
		recipe_collection.recipes.insert(key,recipe);
	}

	recipe_collection
}
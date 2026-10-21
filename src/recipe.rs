use std::println;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Recipe {
    output: String,
	output_rates: Vec<f32>,
	byproducts: Vec<String>,
    inputs: Vec<String>,
	inputs_rates: Vec<f32>,
	produced_by: String
}

pub fn parse_file(path: &str) -> Vec<Recipe> {
	let recipes: Vec<Recipe> = Vec::new();

	let content	 = std::fs::read_to_string(path).expect("Failed to read file");

	let values : toml::Table = toml::from_str(&content).unwrap();

	for (key, value) in values.into_iter() {
		println!("{} / {}", key, value);
	}

	//recipes.push(recipe);

	recipes
}
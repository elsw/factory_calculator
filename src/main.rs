mod recipe;

fn main() {
    println!("Initialising Recipe Library");

    let recipes = recipe::parse_file("recipes/satisfactory_plus/smelter.toml");
}

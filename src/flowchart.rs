
use std::str::FromStr;
use std::{format, println};
use crate::recipe::Recipe;

pub fn plot_flowchart(output_path: &str) {
    let mut test_string = String::new();
    test_string += "Smelter[Smelter x2]  --> |60/min Iron Ingot| Iron_Ingot[Done]\n";
    test_string += "Crusher[Crusher x1]  --> |60/min Crushed Siterite| Smelter[Smelter x2]\n";
    test_string += "Modular_Miner[Modular Miner x1]  --> |60/min Siterite Ore| Crusher[Crusher x1]\n";

	let Ok(flow) = flowmaid::render_svg(test_string.as_str()) else {
        println!("Internal error generating flowchart");
        return;
    };
    if !flow.starts_with("<svg") {
        println!("Internal error with generated flowchart");
        return;
    }

    match std::fs::write(output_path, flow) {
        Err(_e) => println!("Unable to write to result"),
        Ok(_e) => println!("Output Result to result.svg")
    }
}

pub fn append_link(current_str : &mut Vec<String>, building :&String, building_amount :f32,target_output :&str, output_rate :f32) {
    let building_unique_name = building.replace(" ", "_");
    let connect_to_name = target_output.replace(" ", "_");

    let node_flow_str = format!("{}[{} x{}] ",building_unique_name,building,building_amount);
    let output_flow_str = format!("|{}/min {}|",output_rate,target_output);

    current_str.push(format!("{} --> {} {}\n",node_flow_str,output_flow_str,connect_to_name));
}
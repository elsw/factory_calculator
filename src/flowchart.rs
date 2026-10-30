
use std::str::FromStr;
use std::{format, println};

pub struct Flowchart {
    pub output_path: String,
    pub links: Vec<String>,
    pub last_node_str: String,
}

impl Flowchart {
    pub fn new(output_path: String) -> Self {
        let links: Vec<String> = Vec::new();
        let last_node_str: String = String::from_str("Done[Done]").expect("Internal error");
        Self { output_path, links, last_node_str} 
    }

    pub fn plot_flowchart(&mut self) {
        let mut complete_string = String::new();
        for s in self.links.iter() {
            complete_string.push_str(s.as_str());
            complete_string.push_str("\n");
        }

        let Ok(flow) = flowmaid::render_svg(complete_string.as_str()) else {
            println!("Internal error generating flowchart");
            return;
        };
        if !flow.starts_with("<svg") {
            println!("Internal error with generated flowchart");
            return;
        }

        match std::fs::write(&self.output_path, flow) {
            Err(_e) => println!("Unable to write to result"),
            Ok(_e) => println!("Output Result to result.svg")
        }
    }

    pub fn append_link(&mut self, building :&String, building_amount :f32,target_output :&str, output_rate :f32) {
        let building_unique_name = building.replace(" ", "_");

        let node_str = format!("{}[{} x{}] ",building_unique_name,building,building_amount);
        let output_str = format!("|{}/min {}|",output_rate,target_output);

        self.links.push(format!("{} --> {} {}\n",node_str,output_str,self.last_node_str));
        self.last_node_str = node_str;
    }
}
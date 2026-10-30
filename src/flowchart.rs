
use std::str::FromStr;
use std::{format, println};

pub struct Flowchart {
    pub output_path: String,
    pub links: Vec<String>
}

impl Flowchart {
    pub fn new(output_path: String) -> Self {
        let links: Vec<String> = Vec::new();
        let last_node_str: String = String::from_str("Done[Done]").expect("Internal error");
        Self { output_path, links } 
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

    pub fn append_link(&mut self,building_unique_name: &String, building :&String, building_amount :f32,target_output :&str, output_rate :f32,parent_unique_name: &String) {
        let node_str = format!("{}[{} x{}] ",building_unique_name,building,building_amount);
        let output_str = format!("|{}/min {}|",output_rate,target_output);

        self.links.push(format!("{} --> {} {}\n",node_str,output_str,parent_unique_name));
    }
}
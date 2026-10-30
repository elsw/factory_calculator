use std::collections::HashMap;
use crate::flowchart::Flowchart;
use crate::recipe::RecipeCollection;

use std::format;
use std::rc::Rc;

pub struct Solver {
    pub collection: Rc<RecipeCollection>,
    pub flowchart: Flowchart,
    pub log_messages: Vec<String>,
    //Map of buildings required
    pub buildings: HashMap<String,u32>
}

impl Solver {
    pub fn new(collection: &Rc<RecipeCollection>,flow_out_path: String) -> Self {
        let log_messages = Vec::new();
        let flowchart= Flowchart::new(flow_out_path);
        let buildings: HashMap<String,u32> = HashMap::new();
        let collection_ref = collection.clone();
        Self {collection : collection_ref,flowchart,log_messages,buildings}
    }

    pub fn solve(&mut self,target_output: &str, target_per_min: f32) {
        let end_node = format!("Done");
        self.solve_step(target_output,target_per_min,&end_node)
    }

    fn solve_step(&mut self,target_output: &str, target_per_min: f32,parent_unique_name: &String) {
        //self.log_messages.push(format!("Running step, target output {target_output} at {target_per_min}"));

        //Take our own handle on the collection so recipe borrows don't tie up self
        let collection = Rc::clone(&self.collection);

        //Look up the recipes that produces the desired output
        let Some(producer_str_list) = collection.output_lookup.get(target_output) else {
            self.log_messages.push(format!("Error! could not find any recipes producing {target_output}"));
            return;
        };

        //Only handling single producers right now, grab the first entry
        let Some(producer_str) = producer_str_list.get(0) else {
            self.log_messages.push(format!("Internal Error! Got {target_output} has an entry but no listed outputters"));
            return;
        };

        //Grab the recipe from lookup
        let Some(producer) = collection.recipes.get(producer_str) else {
            self.log_messages.push(format!("Internal Error! Got {target_output} lookup but could not get recipe"));
            return;
        };
        if producer.inputs.len() != producer.input_rates.len() {
            self.log_messages.push(format!("Internal Error! {producer_str} has {} inputs, but {} input rates",producer.inputs.len(),producer.input_rates.len()));
            return;
        }

        //Calculate how many of this producer buildings we need
        let Some(produced_in) = producer.produced_in.get(0) else {
            self.log_messages.push(format!("Internal Error, {} is missing produced in",producer_str));
            return;
        };
        let Some(base_output_per_min) = producer.output_rates.get(0) else {
            self.log_messages.push(format!("Internal Error, {} is missing output rates",producer_str));
            return;
        };

        let machines_required= target_per_min / base_output_per_min;

        //Add the buildings to the BOM and use this to generate a unique name for the fl//owchart
        let rounded_machines_required = machines_required.ceil() as u32;
        let mut unique_name = String::new();
        match self.buildings.get_mut(produced_in) {
            Some(s) => {
                *s += rounded_machines_required;
                unique_name = format!("{}{}",produced_in.replace(" ", "_"),s);
            },
            None => {
                self.buildings.insert(produced_in.clone(), rounded_machines_required);
                unique_name = format!("{}{}",produced_in.replace(" ", "_"),rounded_machines_required);
            }
        };
        
        self.log_messages.push(format!("We require {} {} machines set to {}, sending to {}",machines_required,produced_in,producer.output,parent_unique_name));

        self.flowchart.append_link(&unique_name,produced_in,machines_required,
            target_output,target_per_min,&parent_unique_name);
        
        //Resolve inputs to this producer
        for i in 0..producer.inputs.len() {
            let Some(input) = producer.inputs.get(i) else {
                self.log_messages.push(format!("Internal logic error"));
                return;
            };
            let Some(base_input_per_min) = producer.input_rates.get(i) else {
                self.log_messages.push(format!("Internal logic error"));
                return;
            };
            let target_input_per_min =  base_input_per_min * machines_required;

            self.solve_step(input,target_input_per_min,&unique_name);
        }
    }
}


use crate::flowchart::Flowchart;
use crate::recipe::RecipeCollection;

use std::format;
use std::rc::Rc;

pub fn solve(collection: &Rc<RecipeCollection>,flowchart: &mut Flowchart, target_output: &str, target_per_min: f32) -> Vec<String> {
    let mut logs: Vec<String> = Vec::new();
    logs.push(format!("Running step, target output {target_output} at {target_per_min}"));

    let Some(producer_str_list) = collection.output_lookup.get(target_output) else {
        logs.push(format!("Error! could not find any recipes producing {target_output}"));
        return logs;
    };

    //Only handling single producers right now, grab the first entry
    let Some(producer_str) = producer_str_list.get(0) else {
         logs.push(format!("Internal Error! Got {target_output} has an entry but no listed outputters"));
         return logs;
    };

    //Grab the recipe from lookup
    let Some(producer) = collection.recipes.get(producer_str) else {
        logs.push(format!("Internal Error! Got {target_output} lookup but could not get recipe"));
        return logs;
    };
    if producer.inputs.len() != producer.input_rates.len() {
        logs.push(format!("Internal Error! {producer_str} has {} inputs, but {} input rates",producer.inputs.len(),producer.input_rates.len()));
        return logs;
    }

    //Calculate how many of this producer buildings we need
    let Some(produced_in) = producer.produced_in.get(0) else {
        logs.push(format!("Internal Error, {} is missing produced in",producer_str));
        return logs;
    };
    let Some(base_output_per_min) = producer.output_rates.get(0) else {
        logs.push(format!("Internal Error, {} is missing output rates",producer_str));
        return logs;
    };

    let machines_required= target_per_min / base_output_per_min;
    logs.push(format!("Requires {} {} machines set to {}",machines_required,produced_in,producer.output));

    flowchart.append_link(produced_in,machines_required,target_output,target_per_min);
    
    //Resolve inputs to this producer
    for i in 0..producer.inputs.len() {
        let Some(input) = producer.inputs.get(i) else {
            logs.push(format!("Internal logic error"));
            return logs;
        };
        let Some(base_input_per_min) = producer.input_rates.get(i) else {
            logs.push(format!("Internal logic error"));
            return logs;
        };
        let target_input_per_min =  base_input_per_min * machines_required;

        let mut inner_logs = solve(&collection,flowchart,input,target_input_per_min);
        logs.append(&mut inner_logs);
    }

    logs
}


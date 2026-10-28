
pub fn plot_flowchart() {
	let flow = flowmaid::render_svg("flowchart TD\nA[Start] --> B[Done]").unwrap();
    assert!(flow.starts_with("<svg"));

    match std::fs::write("result.svg", flow) {
        Err(_e) => println!("Unable to write to result"),
        Ok(_e) => println!("Output Result to result.svg")
    }
}
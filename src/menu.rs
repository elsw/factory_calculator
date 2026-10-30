// Alternative to menu.rs: shares the recipe list between callbacks by capturing a
// reference-counted pointer in each closure, instead of storing it as Cursive user data.
//
// Note: this uses Arc rather than Rc. Cursive requires every callback to be
// `'static + Send + Sync` (views can be moved between threads), and Rc is neither
// Send nor Sync because its reference count isn't atomic. Arc is the thread-safe
// version of Rc; otherwise they behave identically.

//TODO rully understand closures and capturing in callbacks works https://doc.rust-lang.org/book/ch13-01-closures.html
//Todo fully understand shared objects

use std::{format, println};
use std::str::FromStr;
use std::sync::Arc;
use std::rc::Rc;

use cursive::Cursive;
use cursive::align::HAlign;
use cursive::traits::Scrollable;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, EditView, LinearLayout, SelectView, TextView};

use crate::recipe::RecipeCollection;
use crate::flowchart::Flowchart;
use crate::solver;

fn generate_recipe_list(recipes : &Rc<RecipeCollection>) -> Vec<String> {
    let mut recipe_list: Vec<String> = recipes.output_lookup.keys().cloned().collect();
    recipe_list.sort();
    recipe_list
}

pub fn run(recipes : Rc<RecipeCollection>) {
    let mut siv = cursive::default();

    // Arc::new moves the Vec onto the heap with a reference count of 1.
    // The Vec is never copied after this; only the pointer is.
    let recipe_names: Arc<Vec<String>> = Arc::new(generate_recipe_list(&recipes));

    // Why not just capture `&recipe_names`? The callbacks must be 'static: they live
    // inside the view tree, which outlives this stack frame, so they can't hold a
    // borrow of a local. They have to *own* what they use.
    //
    // Arc::clone bumps the reference count (1 -> 2) and returns a second pointer to
    // the same Vec. Each closure gets its own pointer, so each can own one.
    let names_for_edit = Arc::clone(&recipe_names);

    siv.add_layer(
        LinearLayout::vertical()
            .child(
                Dialog::around(
                    TextView::new(format!("Read in {} Items\n",recipe_names.len()))
                    .with_name("info_box")
                ).fixed_height(20)
                .fixed_width(100)
            )
            .child(
                Dialog::around(
                    LinearLayout::vertical()
                        // the query box is on the top
                        .child(
                            EditView::new()
                                // `move` transfers ownership of names_for_edit into the closure.
                                // Without `move`, the closure would try to borrow it and fail the
                                // 'static bound. After this line names_for_edit can't be used here.
                                .on_edit(move |siv, query, _cursor| on_edit(siv, &names_for_edit, query))
                                // submit the focused (first) item of the matches
                                .on_submit(on_submit)
                                .with_name("query"),
                        )
                        // search results below the input
                        .child(
                            SelectView::new()
                                // with_all_str copies each &String into the view, so a plain
                                // borrow is fine here: nothing is kept after this call.
                                .with_all_str(recipe_names.iter())
                                // Sets the callback for when "Enter" is pressed.
                                .on_submit(show_next_window)
                                // Center the text horizontally
                                .h_align(HAlign::Center)
                                .with_name("matches")
                                .scrollable(),
                        )
                        .fixed_height(10),
                )
                .button("Quit", Cursive::quit)
                .title("Select a Recipe"),
            )
    );

    //Give Cursive access to the recipe library
    let recipe_collection: Rc<RecipeCollection> = Rc::clone(&recipes);
    siv.set_user_data(recipe_collection);
    siv.run();
}

fn log_message(siv: &mut Cursive,message: String){
    siv.call_on_name("info_box", |t: &mut TextView| {
        t.append(format!("{}\n",message));
    });
}

// Update results according to the query.
// Takes a plain &[String]: &Arc<Vec<String>> auto-derefs to it, so this function
// doesn't need to know the data is shared.
fn on_edit(siv: &mut Cursive, recipe_names: &[String], query: &str) {
    // Unlike the user-data version there is no borrow of `siv` here, so the matches
    // can stay as &str borrowed from recipe_names.
    let matches = search_fn(recipe_names.iter().map(String::as_str), query);
    // Update the `matches` view with the filtered array of recipes
    siv.call_on_name("matches", |v: &mut SelectView| {
        v.clear();
        v.add_all_str(matches);
    });
}

// Filter recipes with names containing query string. You can implement your own logic here!
fn search_fn<'a, 'b, T: std::iter::IntoIterator<Item = &'a str>>(
    items: T,
    query: &'b str,
) -> Vec<&'a str> {
    items
        .into_iter()
        .filter(|&item| {
            let item = item.to_lowercase();
            let query = query.to_lowercase();
            item.contains(&query)
        })
        .collect()
}

fn on_submit(siv: &mut Cursive, query: &str) {
    let matches = siv.find_name::<SelectView>("matches").unwrap();
    if matches.is_empty() {
        show_next_window(siv, query);
    } else {
        // pressing "Enter" without moving the focus into the `matches` view will submit the first match result
        let recipe_str = &*matches.selection().unwrap();
        show_next_window(siv, recipe_str);
    };
}

fn show_next_window(siv: &mut Cursive, recipe_name: &str) {
    let recipe_name = recipe_name.to_string();
    siv.add_layer(Dialog::around(EditView::new()
        .on_submit(move |s, query| run_solver(s, query, &recipe_name))
        .with_name("rate"))
        .title("Input desired Items per Minute"));
}

fn run_solver(siv: &mut Cursive,produce_per_minute: &str,recipe_name: &str) {
    siv.pop_layer();
    log_message(siv, format!("Goal is {} {} per minute",produce_per_minute,recipe_name));
    log_message(siv,format!("Cogitating...."));

    //Grab the recipe library
    let recipe_library: &Rc<RecipeCollection> = match siv.user_data::<Rc<RecipeCollection>>() {
        Some(data) => data,
        None => {
			println!("Interal Error accessing recipe library");
			std::process::exit(1);
		}
    };

    let Ok(target_per_min) = f32::from_str(produce_per_minute) else {
        log_message(siv,format!("Invalid input entered to per min target!"));
        return;
    };

    //Run solver
    let mut flowchart = Flowchart::new(format!("result.svg"));
    let logs = solver::solve(recipe_library,&mut flowchart, recipe_name, target_per_min);

    for message in logs {
        log_message(siv, message);
    }

    //Output flowchart
    //plot_flowchart("result.svg");  
    //log_message(siv,format!("Flowchart result outputted to result.svg"));
}

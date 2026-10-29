// Alternative to menu.rs: shares the recipe list between callbacks by capturing a
// reference-counted pointer in each closure, instead of storing it as Cursive user data.
//
// Note: this uses Arc rather than Rc. Cursive requires every callback to be
// `'static + Send + Sync` (views can be moved between threads), and Rc is neither
// Send nor Sync because its reference count isn't atomic. Arc is the thread-safe
// version of Rc; otherwise they behave identically.

//TODO rully understand closures and capturing in callbacks works https://doc.rust-lang.org/book/ch13-01-closures.html
//Todo fully understand shared objects

use std::sync::Arc;

use cursive::Cursive;
use cursive::align::HAlign;
use cursive::traits::Scrollable;
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, EditView, LinearLayout, SelectView, TextView};

use crate::recipe::RecipeCollection;

fn generate_recipe_list(recipes : &RecipeCollection) -> Vec<String> {
    let mut recipe_list: Vec<String> = recipes.recipes.keys().cloned().collect();
    recipe_list.sort();
    recipe_list
}

pub fn run(recipes : &RecipeCollection) {
    let mut siv = cursive::default();

    // Arc::new moves the Vec onto the heap with a reference count of 1.
    // The Vec is never copied after this; only the pointer is.
    let recipe_names: Arc<Vec<String>> = Arc::new(generate_recipe_list(recipes));

    // Why not just capture `&recipe_names`? The callbacks must be 'static: they live
    // inside the view tree, which outlives this stack frame, so they can't hold a
    // borrow of a local. They have to *own* what they use.
    //
    // Arc::clone bumps the reference count (1 -> 2) and returns a second pointer to
    // the same Vec. Each closure gets its own pointer, so each can own one.
    let names_for_edit = Arc::clone(&recipe_names);

    siv.add_layer(
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
    );

    // The count is now 2: one here, one inside the on_edit closure.
    // When `recipe_names` goes out of scope at the end of run() the count drops to 1,
    // and the Vec is freed only when the closure itself is dropped (when its view is).
    siv.run();
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
    siv.pop_layer();
    let text = format!("TODO resolve {recipe_name} !");
    siv.add_layer(Dialog::around(TextView::new(text)).button("Quit", |s| s.quit()));
}

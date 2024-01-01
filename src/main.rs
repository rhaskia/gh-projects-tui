mod app;
mod github;
mod project;
mod ui;

fn main() {
    let app = app::App::new();

    let t = ui::draw_app(app);
    
    // Make sure the user's terminal doesn't break if
    // an error happens
    ui::disable_terminal(); 
    println!("{:#?}", t);
}

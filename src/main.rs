use gtk::{
    ContainerExt, Entry, EntryExt, EntryIconPosition, Grid, GridExt, GtkWindowExt, Inhibit, Label,
    LabelExt, WidgetExt, Window, WindowType,
};
use permutator::CartesianProduct;

mod eval;
use crate::eval::eval;

// useful trait
trait AddIfNotExists<T> {
    fn add_if_not_exists(&mut self, _e: T) {}
}

impl<T: PartialEq> AddIfNotExists<T> for Vec<T> {
    fn add_if_not_exists(&mut self, e: T) {
        if !self.contains(&e) {
            self.push(e);
        }
    }
}

fn create_win(title: &str) -> Window {
    let win = Window::new(WindowType::Toplevel);
    win.set_resizable(false);
    win.set_title(title);
    win.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    win
}
fn label_with_markup(label_text: &str) -> Label {
    let label = Label::new(None);
    label.set_markup(&format!("<b>{}</b>", label_text));
    label.set_max_width_chars(80);
    label
}

struct FirstPage {
    win: Window,
    entry: Entry,
}
struct SecondPage {
    win: Window,
    grid: Grid,
}

struct Pages {
    first_page: FirstPage,
    second_page: SecondPage,
}

impl FirstPage {
    fn new() -> Self {
        let win = create_win("Enter formula");
        let entry = Self::_entry_formula();

        win.add(&entry);
        win.show_all();

        Self { win, entry }
    }

    fn _entry_formula() -> Entry {
        let entry = Entry::new();
        entry.set_icon_from_icon_name(EntryIconPosition::Secondary, "object-select");
        entry.set_placeholder_text("Exp: xANDyOR(zANDx)");

        entry
    }
}
impl SecondPage {
    fn new() -> Self {
        let win = create_win("Truth Table");
        let grid = Self::create_grid();
        win.add(&grid);
        Self { win, grid }
    }
    fn create_grid() -> Grid {
        let grid = Grid::new();
        grid.set_column_spacing(10);
        grid.set_margin_start(20);
        grid
    }
}

impl Pages {
    fn new() -> Self {
        Self {
            first_page: FirstPage::new(),
            second_page: SecondPage::new(),
        }
    }
    fn connect_callbacks(&self) {
        let first_page_win = self.first_page.win.clone();
        let second_page_win = self.second_page.win.clone();
        let second_page_grid = self.second_page.grid.clone();

        self.first_page
            .entry
            .connect_icon_press(move |entry, _icon_pos, _event_btn| {
                first_page_win.hide();
                second_page_win.show_all();
                let entry_text = entry.get_text().expect("Error while reading entry formula");
                let (var_labels, var_values) = parse(&entry_text);
                let grid = second_page_grid.clone();
                Self::fill_grid(&grid, &var_labels, &var_values);
            });
    }
    fn fill_grid(grid: &Grid, labels: &[char], values: &[(Vec<&'static str>, String)]) {
        let last_col = labels.len() as i32;
        for (idx, label) in labels.iter().enumerate() {
            let label_text = label.to_string();
            let label = label_with_markup(label_text.as_str());
            grid.attach(&label, idx as i32, 0, 1, 1);
        }
        grid.attach(&label_with_markup("Output"), last_col, 0, 1, 1);

        for (row, (inputs, output)) in values.iter().rev().enumerate() {
            let row = (row + 1) as i32;
            let inputs: Vec<&str> = inputs
                .iter()
                .map(|v| match *v {
                    "true" => "1",
                    "false" => "0",
                    _ => unreachable!(),
                })
                .collect();

            for (col, input) in inputs.iter().enumerate() {
                let col = col as i32;
                grid.attach(&label_with_markup(input), col, row, 1, 1);
            }

            let output = match output.trim() {
                "true" => "1",
                "false" => "0",
                _ => unreachable!(),
            };

            grid.attach(&label_with_markup(output), last_col, row, 1, 1);
        }

        grid.show_all();
    }
}
fn parse(text: &str) -> (Vec<char>, Vec<(Vec<&'static str>, String)>) {
    let text = {
        // make parsing easier
        let text = text.to_lowercase().replace(" ", "");
        // en
        let text = text
            .replace("and", "&&")
            .replace("or", "||")
            .replace("not", "!");
        // also fr cause why not
        text.replace("et", "&&")
            .replace("ou", "||")
            .replace("non", "!")
    };
    let mut vars = Vec::new();
    let mut results = Vec::new();

    // find variables
    for ch in text.chars() {
        if let 'a'...'z' = ch {
            vars.add_if_not_exists(ch);
        }
    }

    {
        if !vars.is_empty() {
            // Given the input variables, generate all possibilities
            let mut all_possibilities: Vec<Vec<&str>> = Vec::new();

            let vars_bin: Vec<Vec<&str>> = vars.iter().map(|_| vec!["true", "false"]).collect();

            let vars_bin: Vec<&[&str]> = vars_bin.iter().map(|x| x.as_slice()).collect();
            vars_bin.as_slice().cart_prod().for_each(|p| {
                let p: Vec<&str> = p.iter().map(|x| **x).collect();
                all_possibilities.push(p);
            });

            for poss in all_possibilities {
                let mut formula = text.to_string();
                let mut counter = 0;
                while counter < vars.len() {
                    formula = formula.replace(vars[counter], &poss[counter].to_string());
                    counter += 1;
                }

                //formula = handle_bool_manually(&formula);
                let formula = formula.replace("0", "false").replace("1", "true");

                results.push((poss, eval(&formula)));
            }
        } else {
            //let fixed_text = handle_bool_manually(&text);
            let fixed_text = text.replace("0", "false").replace("1", "true");

            results.push((vec![], (eval(&fixed_text))));
        }
    }

    (vars, results)
}

fn main() {
    gtk::init().expect("Failed while trying to initialize gtk");

    let main_page = Pages::new();
    main_page.connect_callbacks();

    gtk::main();
}

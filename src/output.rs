use std::collections::HashMap;

use prettytable::{Attr, Cell, Row, Table};

use crate::github;

fn insert_headers(header_one: &mut Vec<Cell>, header_two: &mut Vec<Cell>, repo: &str) {
    let before = header_two.len();
    header_two.push(Cell::new("Last Updated").with_style(Attr::Bold));
    header_two.push(Cell::new("Updated By").with_style(Attr::Bold));
    header_two.push(Cell::new("PR").with_style(Attr::Bold));
    let after = header_two.len();
    header_one.push(Cell::new(repo).with_style(Attr::Bold).with_hspan(after - before));
}

fn insert_cells(cells: &mut Vec<Cell>, value: &HashMap<String, github::Branch>, repo: &str) {
    match value.get(repo) {
        Some(repo) => {
            cells.push(Cell::new(&repo.last_updated));
            cells.push(Cell::new(&repo.last_updated_by));
            match repo.pr {
                Some(pr) => {
                    let pr_string = format!("{}", pr);
                    cells.push(Cell::new(&pr_string));
                }
                None => cells.push(Cell::new("")),
            };
        }
        None => {
            cells.push(Cell::new(""));
            cells.push(Cell::new(""));
            cells.push(Cell::new(""));
        }
    }
}

pub fn print(branches: Vec<(&String, &HashMap<String, github::Branch>)>, repos: Vec<&str>) {
    let mut header_one = vec![Cell::new("Branch").with_style(Attr::Bold)];
    let mut header_two = vec![Cell::new("")];

    for repo in &repos {
        insert_headers(&mut header_one, &mut header_two, &repo);
    }

    let mut table = Table::new();
    table.add_row(Row::new(header_one));
    table.add_row(Row::new(header_two));

    for (key, value) in branches {
        let mut cells = vec![Cell::new(key)];
        for repo in &repos {
            insert_cells(&mut cells, value, repo);
        }
        table.add_row(Row::new(cells));
    }

    table.printstd();
}
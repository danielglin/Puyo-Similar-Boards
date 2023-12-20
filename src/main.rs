mod parse;
mod boards;

use crate::parse::parse_sheet;
use crate::boards::{TextQryBoard, run_qry};
use calamine::{Xlsx, open_workbook};
use logru::textual::TextualUniverse;

fn main() {
    // load db
    let book = "base_db.xlsx";
    let mut excel: Xlsx<_> = open_workbook(book).unwrap();
    let mut tu = TextualUniverse::new();
    parse_sheet(&mut excel, "key", &mut tu, false);
    parse_sheet(&mut excel, "l-shape", &mut tu, false);
    parse_sheet(&mut excel, "flat", &mut tu, false);

    // load qry
    let qry = parse_sheet(&mut excel, "query", &mut tu, true);
    match qry {
        Some(s) => {
            // run qry
            let qry_board = TextQryBoard::from_str(&s);
            let nb_solns = run_qry(&mut tu, &qry_board);

            let mut index = 0;

            for soln in &nb_solns {
                println!("Solution {}: \n{}", index, soln.to_color_str());
                index += 1;
            }            
        }
        None => println!("Invalid Query!")
    }
    
}
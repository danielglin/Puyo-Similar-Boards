use calamine::{Reader, Xlsx, open_workbook, DataType};
use logru::ast::{self, Rule, Term};
use logru::textual::TextualUniverse;
use logru::solver::{query_dfs};
// use std::collections::{HashMap, HashSet};
use crate::boards::load_str_board;

/// parses boards in an excel sheet and loads them into 
/// a text universe
/// book is an opened workbook
/// sheet is the name of the worksheet to parse
/// tu is the text universe to load the parsed boards to
/// qry is true if parsing a qry, in which case the func returns `Some(String)`
pub fn parse_sheet<RS>(book: &mut Xlsx<RS>, sheet: &str, tu: &mut TextualUniverse,
    qry: bool) 
    -> Option<String>
    where RS: std::io::Read + std::io::Seek
{
    let mut board = String::new();

    if let Some(Ok(r)) = book.worksheet_range(sheet) {
        for row in r.rows() {

//             println!("range height: {}", r.height());
//             println!("curr row ind: {}", row_counter);
//             println!("board so far: {:?}", board);
            let mut parsed_row = String::from("\n");

            'elem:
            for elem in row {
    //             println!("elem: {:?}", elem);
    //             print_type_of(&elem);
                match elem {
                    DataType::Empty => parsed_row.push(' '),
                    DataType::String(s) => {
                        if s == "end" {
                            if !qry {
                                load_str_board(tu, &board);
                                board = String::new();               
    
                                // continue to the next row immediately 
                                break 'elem;
                            } else {
                                return Some(board);
                            }

                        } else if s.len() == 1 {
                            parsed_row.push_str(s); 
                        } else {
                            // useful for debugging invalid boards
                            println!("{:?}", s);

                            // reset the board to avoid adding too many
                            // empty lines at the start
                            board = String::new();
                        }

                    }
                    _ => ()
                }
            } // end of row loop
            // add parsed row to the curr board
            if (parsed_row != "\n     ") && (parsed_row != "\n") {
                board.push_str(&parsed_row);
            }
        } // end of looping over the range
        None
    } else {
        println!("Invalid sheet: {}", sheet);
        None
    }
    
}


#[cfg(test)]
mod tests {
    use calamine::{Xlsx, open_workbook};
    use logru::textual::TextualUniverse;
    use super::*;

    fn get_rslts(book: &str, sheet: &str) -> Vec<String> {
        let mut excel: Xlsx<_> = open_workbook(book).unwrap();
        let mut tu = TextualUniverse::new();
        let mut rslts = Vec::new();
        parse_sheet(&mut excel, sheet, &mut tu, false);

        for rule in tu.inner().rules() { // is very messy if we don't use the prettifier
            let pretty_rule = tu.pretty().rule_to_string(rule);
            rslts.push(pretty_rule);
        }
        rslts        
    }

    fn get_qry(book: &str, sheet: &str) -> String {
        let mut excel: Xlsx<_> = open_workbook(book).unwrap();
        let mut tu = TextualUniverse::new();
        let qry = parse_sheet(&mut excel, sheet, &mut tu, true).unwrap();
        qry
    }

    #[test]
    fn test_simple_boards_1() {
        let rslts = get_rslts("test_excel.xlsx", "Sheet1");

        assert_eq!(rslts.len(), 2);
        assert_eq!(rslts[0], "board(row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, b, l), row(r, b, g, b, y, b), row(r, r, b, g, g, b), row(b, b, g, y, y, y)).");
        assert_eq!(rslts[1], "board(row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(b, b, r, l, l, l)).");
    }

    // a sheet w/ a single board that has a row in the middle
    #[test]
    fn single_float() {
        let rslts = get_rslts("test_excel.xlsx", "single_floating");

        assert_eq!(rslts.len(), 1);
        assert_eq!(rslts[0], "board(row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, r, r, b, b), row(l, l, l, l, l, l)).");
    }

    // a sheet w/ a single board that has a non-full row in the middle
    #[test]
    fn float_incomplete() {
        let rslts = get_rslts("test_excel.xlsx", "float_incomplete_row");

        assert_eq!(rslts.len(), 1);
        assert_eq!(rslts[0], "board(row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, r, r, b, l), row(l, l, l, l, l, l)).");
    }

    // a sheet w/ a single board that has a non-full row in the middle 
    // and "end" at the end of a row
    // and doesn't have the non-full row ended w/ a " "
    #[test]
    fn float_incomplete_end() {
        let rslts = get_rslts("test_excel.xlsx", "float_incomplete_row_end_end");

        assert_eq!(rslts.len(), 1);
        assert_eq!(rslts[0], "board(row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, r, r, b, l), row(l, l, l, l, l, l)).");
    }    

    // a sheet w/ a single invalid board
    #[test]
    fn single_invalid() {
        let rslts = get_rslts("test_excel.xlsx", "single_invalid");
        assert_eq!(rslts.len(), 0);
    }

    // a sheet w/ a qry board
    #[test]
    fn qry_1() {
        let qry = get_qry("test_excel.xlsx", "qry_1");
        let expected_qry = String::from("\n   rr \n      ");
        assert_eq!(qry, expected_qry);
    }

    // a sheet w/ valid boards but w/ lots of lines b/w boards
    // this should be parsed w/o all the extra lines causing
    // parsing to fail
    #[test]
    fn extra_lines() {
        let rslts = get_rslts("test_fail_1.xlsx", "broken_key");

        assert_eq!(rslts.len(), 2);
        assert_eq!(rslts[0], "board(row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, g, l, g), row(l, l, l, r, g, g), row(b, g, y, b, r, r), row(b, b, g, y, y, r), row(g, g, y, b, b, b)).");
        assert_eq!(rslts[1], "board(row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, l), row(l, l, l, l, l, y), row(l, l, l, l, g, y), row(l, l, l, l, y, g), row(y, p, b, g, p, g), row(y, y, p, b, b, p), row(p, p, b, y, p, p)).");
    }
}

use logru::ast::{self, Rule, Term};
use logru::textual::TextualUniverse;
use logru::solver::{query_dfs};
use std::collections::{HashMap, HashSet};

// types to rep a normal board, a db board, and a qry board
// this type has methods to create both db boards and qry boards
#[derive(Debug)]
pub enum Puyo {
    B,
    G,
    P,
    R,
    Y,
    Empty,
    Garbage
}

#[derive(Debug)]
pub struct InvalidPuyoError {
    invalid_puyo: String
}

// a normal row is a fixed tuple of 6 puyo
#[derive(Debug)]
pub struct NormalRow {
    puyo: Vec<Puyo>
}

// a normal board is a variable-length vec of rows
// only have a fixed length when returning a db or qry board
#[derive(Debug)]
pub struct NormalBoard {
    rows: Vec<NormalRow>
}

pub struct DBBoard {
    // like:
    //     vec![
    //         ast::app(row, vec![b.into(), b.into(), y.into(), p.into(), r.into(), r.into()]),
    //         ast::app(row, vec![r.into(), r.into(), b.into(), y.into(), y.into(), r.into()]),
    //         ast::app(row, vec![r.into(), b.into(), y.into(), p.into(), r.into(), b.into()]),
    //     ])
    padded_board: Vec<ast::Term> // padded to fixed length of 13 rows
}

pub struct QryBoard {
    // like:
//     vec![
//         ast::app(row, vec![g_var.into(), g_var.into(), b_var.into(), r_var.into(), y_var.into(), y_var.into()]),
//         rslt_row1.into(),
//         rslt_row2.into()
//     ] 
    padded_board: Vec<ast::Term>
}

// ***************************************************
// for now, trying out the text universe, so making types for text db and text qrys
// ***************************************************

// helper types for decoding solutions (going fr '$0' -> 'g')
#[derive(Debug)]
pub enum SymbolRow {
    IndivSymbols(Vec<String>),
    EntireRow(String)
}

pub struct TextDBRow {
    row: String
}

pub struct TextDBBoard {
    board: String
}

pub struct TextQryBoard {
    qry_str: String,
    sym_to_char: HashMap<String, char>, 
    sym_board: Vec<SymbolRow>
}

// ***************************************************
// impl.s
// ***************************************************

impl Puyo {
    // takes in a str of length 1 and lower-case letters and returns a Puyo
    // puyo: a string slice of length 1, rep'ing a puyo
    pub fn from_str(puyo: &str) -> Result<Self, InvalidPuyoError> {
        match puyo {
            "b" => Ok(Puyo::B),
            "g" => Ok(Puyo::G),
            "p" => Ok(Puyo::P),
            "r" => Ok(Puyo::R),
            "y" => Ok(Puyo::Y),
            " " => Ok(Puyo::Empty),
            "j" => Ok(Puyo::Garbage),
            invalid => Err(InvalidPuyoError { invalid_puyo: invalid.to_string()})
        }
    }
    
    pub fn to_str(&self) -> String {
        match self {
            &Puyo::B => String::from("b"),
            &Puyo::G => String::from("g"),
            &Puyo::P => String::from("p"),
            &Puyo::R => String::from("r"),
            &Puyo::Y => String::from("y"),
            &Puyo::Empty => String::from(" "),
            &Puyo::Garbage => String::from("j")
        }
    }
    
    pub fn to_color_str(&self) -> String {
        // println!("\x1b[42;1mHello\x1b[0m");
        match self {
            &Puyo::B => String::from("\x1b[46;1mb\x1b[0m"),
            &Puyo::G => String::from("\x1b[42;1mg\x1b[0m"),
            &Puyo::P => String::from("\x1b[45;1mp\x1b[0m"),
            &Puyo::R => String::from("\x1b[41;1mr\x1b[0m"),
            &Puyo::Y => String::from("\x1b[43;1my\x1b[0m"),
            &Puyo::Empty => String::from(" "),
            &Puyo::Garbage => String::from("\x1b[47;1mj\x1b[0m")
        } 
    }
    
    pub fn to_text_db_str(&self) -> String {
        match self {
            &Puyo::B => String::from("b"),
            &Puyo::G => String::from("g"),
            &Puyo::P => String::from("p"),
            &Puyo::R => String::from("r"),
            &Puyo::Y => String::from("y"),
            &Puyo::Empty => String::from("l"),
            &Puyo::Garbage => String::from("j")
        }        
    }
}

impl NormalRow {
    // takes in a row as a string like "rggb y"
    // and returns a NormalRow
    // row is a string slice w/ 6 letters, one for each col in the row
    pub fn from_str(row: &str) -> Result<Self, InvalidPuyoError> {
        let mut row_vec = vec![];
        
        for puyo in row.chars() {
            match puyo {
                p => {
                    let indiv_puyo = Puyo::from_str(&p.to_string())?;
                    row_vec.push(indiv_puyo);
                }
            }
        }
        
        if row_vec.len() != 6 {
            return Err(InvalidPuyoError { invalid_puyo: format!("Invalid number of puyo: {}", row_vec.len()) })
        }
        
        Ok(NormalRow { puyo: row_vec })
    }
    
    pub fn to_str(&self) -> String {
        let mut row_str = String::new();
        
        for puyo in &self.puyo {
            row_str.push_str(&puyo.to_str());
        }
        
        row_str
    }
    
    pub fn to_color_str(&self) -> String {
        let mut row_str = String::new();
        
        for puyo in &self.puyo {
            row_str.push_str(&puyo.to_color_str());
        }
        
        row_str
    }
    
    // creates a TextDBRow
    // TextDBRow vals are like "row(r, r, g, b, y, y)"
    pub fn to_text_db_row(&self) -> TextDBRow {
        let mut row_str = String::from("row(");
        let mut i = 0;
        
        for puyo in &self.puyo {
            row_str.push_str(&puyo.to_text_db_str());
            if i < 5 {
                row_str.push_str(", ");
            }            
            i += 1;
        }
        
        row_str.push_str(")");
        TextDBRow{ row: row_str }
    }
}

impl NormalBoard {
    /// takes in a str that has up to 13 lines, w/ 6 chars in a line
    /// to rep a puyo board
    /// the beginning of the str reps the top of the board,
    /// and the end of the str reps the bottom of the board
    pub fn from_str(board: &str) -> Result<Self, InvalidPuyoError> {
        let mut board_vec = vec![];
        let cleaned_board = trim_newlines(board);
        
        for line in cleaned_board.split("\n") {
            board_vec.push(NormalRow::from_str(line)?);
        }
        
        if board_vec.len() > 13 {
            return Err(InvalidPuyoError { invalid_puyo: format!("Invalid number of rows: {}", board_vec.len()) })
        }
        
        Ok(NormalBoard { rows: board_vec })
    }
    
    pub fn to_str(&self) -> String {
        let mut board_str = String::new();
        
        for row in &self.rows {
            board_str.push_str(&row.to_str());
            board_str.push_str("\n");
        }
        
        board_str
    }
    
    pub fn to_color_str(&self) -> String {
        let mut board_str = String::new();
        
        for row in &self.rows {
            board_str.push_str(&row.to_color_str());
            board_str.push_str("\n");
        }
        
        board_str
    }
    
    // creates a TextDBBoard
    pub fn to_text_db_board(&self) -> TextDBBoard {
        let mut board_str = String::new();
        let mut i = 0;
        
        for (ind, row) in self.rows.iter().enumerate() {
            board_str.push_str(&row.to_text_db_row().row);
            if ind < (self.rows.len()-1) {
                board_str.push_str(", ");
            }
            i += 1;
        }
        
        
        // if don't have to full 13 rows, need to pad out to 13 rows at the
        // start of the str
        // i here is num of rows we have
        while i < 13 {
            board_str.insert_str(0, "row(l, l, l, l, l, l), ");
            i += 1;
        }
        board_str.push_str(").");
        board_str.insert_str(0, "board(");
        TextDBBoard { board: board_str }
    }
}

impl TextQryBoard {
    pub fn from_str(s: &str) -> Self {
        let mut qry_str = String::new();
        let mut char_to_sym: HashMap<char, String> = HashMap::with_capacity(4);
        let mut sym_to_char = HashMap::with_capacity(4);
        // let mut chars_assigned = HashSet::with_capacity(4); // keeps track of what chars have already been assigned a symbol
        let mut sym_board: Vec<SymbolRow> = Vec::new();
        let mut color_index = 0;
        let mut dummy_index = 20;

        let mut char_counter = 0;
        let mut row_counter = 0;

        // count the num of rows in the string
        let num_rows = trim_newlines(s).split("\n").collect::<Vec<&str>>().len();

        let cleaned_board = trim_newlines(s);
        for line in cleaned_board.split("\n") {
            // parse a line
//             println!("-- Line -- ");
            qry_str.push_str("row(");
            char_counter = 0;  
            let mut sym_row: Vec<String> = Vec::new();

            for puyo_c in line.chars() {              
//                 println!("symbol: {}", puyo_c);
                // track which char in the str gets which $symbol
                match puyo_c {
                    'b' | 'g' | 'p' | 'r' | 'y' => {
                        // if the char already has a symbol,
                        // get the symbol and add it to the qry str
                        // but if the char doesn't have a symbol yet,
                        // add assign it a symbol and add it to the syms_to_char mapping
                        match char_to_sym.get(&puyo_c) {
                            Some(c) => {
                                qry_str.push_str(c);
                                sym_row.push(c.to_string()); // updating the symbol row
                            }
                            None => {
                                let new_symbol = format!("${}", color_index);
                                char_to_sym.insert(puyo_c, new_symbol.clone());
                                // also need to update the sym_to_char map
                                sym_to_char.insert(new_symbol.clone(), puyo_c);
                                qry_str.push_str(&new_symbol);
                                sym_row.push(new_symbol); // updating the symbol row
                                color_index += 1;                        
                            }
                        }
        //                 if char_to_sym.contains_key(&symbol) {
        //                     qry_str.push_str();
        //                 }
                    } // end of case where the char is a puyo char
                    ' ' => {
                        // if char is a space, need diff dummy vars for each space
                        let new_symbol = format!("${}", dummy_index);
                        qry_str.push_str(&new_symbol);
                        sym_row.push(new_symbol);
                        dummy_index += 1;
                    } // end of case where char is a space        
                    non_puyo_char => {
                        match char_to_sym.get(&non_puyo_char) {
                            Some(c) => {
                                qry_str.push_str(c);
                                sym_row.push(c.to_string()); // updating the symbol row
                            }
                            None => {
                                let new_symbol = format!("${}", dummy_index);
                                char_to_sym.insert(non_puyo_char, new_symbol.clone());
                                sym_to_char.insert(new_symbol.clone(), non_puyo_char);
                                qry_str.push_str(&new_symbol);
                                sym_row.push(new_symbol);
                                dummy_index += 1;
                            }
                        }
                    } // end of case where non-puyo/non-space char
                }
                // add a comma b/w each symbol
                if char_counter < 5 {
                    qry_str.push_str(", ");
                }
                char_counter += 1;

            } // end of for loop for each char
            qry_str.push_str(")");
            if row_counter < num_rows - 1 { // need to change this to check if this is the last row, not for 13 rows
                qry_str.push_str(",\n");
            } else {
                qry_str.push_str(")."); // needed for the "board(" part
            }
            // now that we're done w/ 1 row, add the symbol row to the symbol board
            sym_board.push(SymbolRow::IndivSymbols(sym_row));
            row_counter += 1;
        }

        // pad the qry str w/ vars for empty rows
        while row_counter < 13 {
            let row_symbol = format!("$13{}, ", row_counter+1);
            qry_str.insert_str(0, &row_symbol);
            sym_board.insert(0, SymbolRow::EntireRow(format!("$13{}", row_counter+1)));
            row_counter += 1;
        }

        // add the start of the qry ("board(")
        qry_str.insert_str(0, "board(");

        TextQryBoard { qry_str: qry_str, sym_to_char: sym_to_char, sym_board: sym_board }
    }
}

// func to check if already seen a color
// returns true if color hasn't been seen yet and adds the color to the hashset
// returns false ow 
pub fn is_new_color(colors_assigned: &mut HashSet<String>, color: &str) -> bool {
    if colors_assigned.contains(color) {
        false
    } else {
        colors_assigned.insert(color.to_owned());
        true
    }    
}

pub fn run_qry(tu: &mut logru::textual::TextualUniverse, qry: &TextQryBoard) -> Vec<NormalBoard> {
    let t_qry = tu.prepare_query(&qry.qry_str).unwrap();
    let t_solns = query_dfs(tu.inner(), &t_qry);
    let mut answers = HashMap::new();
    let mut nr_solns = Vec::new();
    const DUMMY_INDEX_START: usize = 20;
    
    'soln: for solution in t_solns {
        let mut soln_rows = Vec::new();
        let mut colors_assigned = HashSet::new();
        
        // create mapping fr symbols to puyo/answers
        for (index, var) in solution.into_iter().enumerate() {
            if let Some(term) = var {
                let symbol = format!("${}", index);
//                 rslt_str = rslt_str.replace(&symbol, &tu.pretty().term_to_string(&term));
                // check the sol'n to make sure that all the terms 
                // correspond to different colors
                // if have a sol'n that doesn't have unique colors, skip to next sol'n
                if (index < DUMMY_INDEX_START) && is_new_color(&mut colors_assigned, &tu.pretty().term_to_string(&term)) || (index >= DUMMY_INDEX_START) {
                    answers.insert(symbol, tu.pretty().term_to_string(&term));
                } else {
                    continue 'soln;
                }
                
            } 
//             println!("inner rslt: {}", rslt_str);
        }
        
        // use mapping fr symbols to puyo/answers to create a sol'n NormalBoard
        // fr the symbol board
        for row in &qry.sym_board {
            match row {
                SymbolRow::EntireRow(row_sym) => {
                    let mut row_rslt = match answers.get(row_sym) {
                        Some(ans) => ans.to_string(),
                        None => row_sym.to_string()
                    };
                    // make a NormalRow
                    // have to clean the rslt before can make a NormalRow
                    row_rslt = row_rslt.replace("row(", "");
                    row_rslt = row_rslt.replace(",", "");
                    row_rslt = row_rslt.replace(")", "");
                    row_rslt = row_rslt.replace(" ", "");
                    row_rslt = row_rslt.replace("l", " ");
//                     println!("row_rslt: {}", row_rslt);
                    // ignore blank rows
                    if row_rslt != "      " {
                        let nr = NormalRow::from_str(&row_rslt).unwrap_or(NormalRow::from_str("j j j ").unwrap());
                        soln_rows.push(nr);                        
                    }
//                     soln_rows.insert_str(0, &row_rslt);
                } // end of case where have an entire row as the sym row
                SymbolRow::IndivSymbols(ref v_row_syms) => {
                    let mut indiv_syms = String::new();
                    for indiv_sym in v_row_syms {
                        let indiv_ans = match answers.get(indiv_sym) {
                            Some(ans) => ans.to_string(),
                            None => indiv_sym.to_string()
                        };
                        indiv_syms.push_str(&indiv_ans);
                    }
                    indiv_syms = indiv_syms.replace("l", " ");
//                     println!("indiv_syms: {}", &indiv_syms);
                    if indiv_syms != "      " {
                        let nr = NormalRow::from_str(&indiv_syms).unwrap_or(NormalRow::from_str("j j j ").unwrap());
                        soln_rows.push(nr);
                    }
                }
            }
        } // end of for loop iterating over rows
        
        // now that we have the rows, make a NormalBoard fr them
        let nb = NormalBoard { rows: soln_rows };
        nr_solns.push(nb);
    } // end of for loop iterating over sol'ns
    nr_solns
}

// convenience func to load boards to a text universe
pub fn load_str_board(tu: &mut logru::textual::TextualUniverse, s: &str) {
//     let s_1 = String::from("
//     bgbyyy
//     bbgbbb
//     ggbygp
//     ");
    if NormalBoard::from_str(s).is_ok() {
        let b = NormalBoard::from_str(s).unwrap();
        tu.load_str(&b.to_text_db_board().board,).unwrap();
    } else {
        println!("Error loading board: {}", s);
    }
}

/// removes newlines fr the start and end of a str
/// to be used to create a board
fn trim_newlines(board: &str) -> &str {
    // start
    let mut cleaned_board = if board.starts_with("\n") {
        board.trim_start_matches("\n")
    } else {
        board
    };
    
    // end
    if cleaned_board.ends_with("\n") {
        cleaned_board.trim_end_matches("\n")
    } else {
        cleaned_board
    }
}

#[cfg(test)]
mod board_tests {
    use super::*;

    #[test]
    fn qry_empty_bottom_row() {
        let q_1 = String::from("\n   rr \n      \n");
        let qb_1 = TextQryBoard::from_str(&q_1);
        let expected_qry_str = "board($1313, $1312, $1311, $1310, $139, $138, $137, $136, $135, $134, $133, row($20, $21, $22, $0, $0, $23),\nrow($24, $25, $26, $27, $28, $29)).";
        assert_eq!(qb_1.qry_str, expected_qry_str);
    }
}
/*
    Serkr - An automated theorem prover. Copyright (C) 2015-2016 Mikko Aarnos.

    Serkr is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Serkr is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Serkr. If not, see <http://www.gnu.org/licenses/>.
*/

extern crate regex;


use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use crate::tptp_parser::ast::*;
//use tptp_parser::parser_grammar::parse_TPTP_file;

/// Checks if a character is a double quote. 
/// We only have this function because doing this the obvious way breaks syntax highlighting.
/// A bit brittle.
fn is_double_quote(c: char) -> bool {
    c as u8 == 34
}

/// Used for removing all comments from the file parsed in.
fn remove_comments(s: &str) -> String {
    // First remove all comment blocks.
    let comment_block_regex = regex::Regex::new(r"[/][*]([^*]*[*][*]*[^/*])*[^*]*[*][*]*[/]").expect("This should always work");
    let s2 = comment_block_regex.replace_all(s, regex::NoExpand(""));
    let mut s3 = "".to_owned();
    
    // Then remove all comment lines. 
    // This is a bit tricky due to the possibility of single-quoted and double-quoted strings.
    // Escaping is also annoying.
    // All in all this part most likely has some subtle bugs.
    for l in s2.lines() {
        let mut inside_single_quoted = false;
        let mut inside_double_quoted = false;
        let mut comment_start_location = None;
        let mut escaping_next = false;
        
        for (i, c) in l.chars().enumerate() {
            assert!(!inside_single_quoted || !inside_double_quoted);
            
            if (inside_single_quoted || inside_double_quoted) && c == '\\' {
                // Two slashes in a row is just a character, otherwise raise escape flag.
                escaping_next = !escaping_next;
                continue;
            } else if !inside_double_quoted && c == '\'' {
                if !escaping_next {
                    inside_single_quoted = !inside_single_quoted;
                } 
            } else if !inside_single_quoted && is_double_quote(c)  {
                if !escaping_next {
                    inside_double_quoted = !inside_double_quoted;
                } 
            } else if c == '%' && !inside_single_quoted && !inside_double_quoted{
                // This percentage sign wasn't inside a string, so it is a real comment.
                comment_start_location = Some(i);
                break;
            }
            
            escaping_next = false;
        }
        
        // Did we find a comment? If so remove it.
        if let Some(pos) = comment_start_location {
            s3.push_str(&l[0..pos]);
        } else {
            s3.push_str(l);
        }
    }
    
    s3
}

/// Remove all empty lines (i.e. containing only whitespace) from the file passed in.
/// Not sure if this is necessary.
fn remove_empty_lines(s: &str) -> String {
    let empty_line_regex = regex::Regex::new(r"[ ]*[\n]").expect("This should always work");
    empty_line_regex.replace_all(s, regex::NoExpand("")).into_owned()
}

/// Reads the file at the location given into a String.
fn read_file(s: &str) -> Result<String, String> {
    let path = Path::new(s);
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => return Err(format!("couldn't open {}: {}", display, &why)),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut f = String::new();
    match file.read_to_string(&mut f) {
        Err(why) => Err(format!("couldn't read {}: {}", display, &why)),
        Ok(_) => Ok(f),
    }
}

/// Reads the file at the location given into a String, and preprocesses it into a more suitable form for the parser.
fn read_and_preprocess_file(s: &str) -> Result<String, String> {
    let s2 = read_file(s)?;
    let s3 = remove_comments(&s2);
    Ok(remove_empty_lines(&s3))
}

/// Hacky way to see if an annotated formula has the same name as some string.
fn annotated_formula_names_match(af: &AnnotatedFormula, s: &str) -> bool {
    match *af { 
        AnnotatedFormula::Cnf(ref f) | 
        AnnotatedFormula::Fof(ref f) => f.0 == s,
    } 
}

/// Handles an include directive. Includes work pretty much like in C, just paste the file to where the include was.
fn handle_include(incl: Include) -> Result<Vec<AnnotatedFormula>, String> {
    let include_file = parse_tptp_file(&incl.0)?;
    if let Some(formulae) = incl.1 {
        Ok(include_file.into_iter().filter(|input| formulae.iter().any(|s| annotated_formula_names_match(input, s))).collect())
    } else {
        Ok(include_file)
    }
}

use crate::tptp_parser;
/// Parses a file in TPTP format to a vector of annotated formulae.
#[cfg_attr(feature="clippy", allow(use_debug))]
pub fn parse_tptp_file(s: &str) -> Result<Vec<AnnotatedFormula>, String> {
    let preprocessed_file = read_and_preprocess_file(s)?;
    let file_parser = tptp_parser::parser_grammar::TPTP_fileParser::new();
    let parsed_file = file_parser.parse(&preprocessed_file).map_err(|x| format!("{:?}", x))?;
    
    // Handle all includes.
    let mut formulas = Vec::<AnnotatedFormula>::new(); 
    for input in parsed_file {
        match input {
            TptpInput::AnnForm(f) => formulas.push(f),
            TptpInput::Incl(i) => formulas.append(&mut handle_include(i)?),
        }
    }
    
    Ok(formulas)
}

fn parse_cnf_annotated(s: &str) -> Result<(String, String, Formula), String> {
    let p = tptp_parser::parser_grammar::cnf_annotatedParser::new();
    let r = p.parse(s).map_err(|x| format!("{:?}", x));
    r
}

fn parse_single_quoted(s: &str) -> Result<String, String> {
    let p = tptp_parser::parser_grammar::single_quotedParser::new();
    let r = p.parse(s).map_err(|x| format!("{:?}", x));
    r
}

fn parse_include(s: &str) -> Result<(String, Option<Vec<String>>), String> {
    let p = tptp_parser::parser_grammar::includeParser::new();
    let r = p.parse(s).map_err(|x| format!("{:?}", x));
    r
}

fn parse_distinct_object(s: &str) -> Result<String, String> {
    let p = tptp_parser::parser_grammar::distinct_objectParser::new();
    let r = p.parse(s).map_err(|x| format!("{:?}", x));
    r
}

fn parse_dollar_word(s: &str) -> Result<String, String> {
    let p = tptp_parser::parser_grammar::dollar_wordParser::new();
    let r = p.parse(s).map_err(|x| format!("{:?}", x));
    r
}

#[cfg(test)]
mod test {
    
    use crate::tptp_parser::ast::*;   
    use super::parse_tptp_file;
    use super::parse_cnf_annotated;
    use super::parse_single_quoted;
    use super::parse_include;
    use super::parse_distinct_object;
    use super::parse_dollar_word;


    #[test]
    fn parser_test_0() {
        assert!(parse_tptp_file("examples/SET060-6.p").is_ok());
    }

    #[test]
    fn parser_test_1() {
        assert!(parse_tptp_file("test_problems/SYN000-1.p").is_ok());
    }
    
    #[test]
    fn parser_test_2() {
        assert!(parse_tptp_file("test_problems/SYN000+1.p").is_ok());
    }
    
    #[test]
    fn parser_test_3() {
        // Try to read a file which does not exist.
        assert!(parse_tptp_file("test_problems/does_not_exists.p").is_err());
    }
    
    
    #[test]
    fn parse_cnf_annotated_propositional() {
        let res = parse_cnf_annotated("cnf(propositional,axiom,( p0 | ~ q0 | r0 | ~ s0 )).");
        
        let p0 = Formula::Predicate("p0".to_owned(), Vec::new());
        let not_q0 = Formula::Not(Box::new(Formula::Predicate("q0".to_owned(), Vec::new())));
        let r0 = Formula::Predicate("r0".to_owned(), Vec::new());
        let not_s0 = Formula::Not(Box::new(Formula::Predicate("s0".to_owned(), Vec::new())));
        let fm = Formula::Or(Box::new(Formula::Or(Box::new(Formula::Or(Box::new(p0), Box::new(not_q0))), Box::new(r0))), Box::new(not_s0));
        
        assert_eq!(res.unwrap(), ("propositional".to_owned(), "axiom".to_owned(), fm));
    }
    
    #[test]
    fn parse_cnf_annotated_first_order() {
        let res = parse_cnf_annotated("cnf(first_order,axiom,( p(X) | ~ q(X,a) | r(X,f(Y),g(X,f(Y),Z)) | ~ s(f(f(f(b)))) )).");
        
        let x = Term::Variable("X".to_owned());
        let y = Term::Variable("Y".to_owned());
        let z = Term::Variable("Z".to_owned());
        let a = Term::Function("a".to_owned(), Vec::new());
        let b = Term::Function("b".to_owned(), Vec::new());
        let f_y = Term::Function("f".to_owned(), vec!(y.clone()));
        let g_x_f_y_z = Term::Function("g".to_owned(), vec!(x.clone(), f_y.clone(), z.clone()));
        let f_f_f_b = Term::Function("f".to_owned(), vec!(Term::Function("f".to_owned(), vec!(Term::Function("f".to_owned(), vec!(b))))));
        
        let p_x = Formula::Predicate("p".to_owned(), vec!(x.clone()));        
        let not_q_x_a = Formula::Not(Box::new(Formula::Predicate("q".to_owned(), vec!(x.clone(), a))));  
        let r_x_f_y_g_x_f_y_z = Formula::Predicate("r".to_owned(), vec!(x, f_y, g_x_f_y_z));  
        let not_s_f_f_f_b = Formula::Not(Box::new(Formula::Predicate("s".to_owned(), vec!(f_f_f_b))));  
        
        let fm = Formula::Or(Box::new(Formula::Or(Box::new(Formula::Or(Box::new(p_x), Box::new(not_q_x_a))), Box::new(r_x_f_y_g_x_f_y_z))), Box::new(not_s_f_f_f_b));
        
        assert_eq!(res.unwrap(), ("first_order".to_owned(), "axiom".to_owned(), fm));
    }
    
    #[test]
    fn parse_cnf_annotated_equality() {
        let res = parse_cnf_annotated("cnf(equality,axiom,( f(Y) = g(X,f(Y),Z) | f(f(f(b))) != a | X = f(Y) )).");
        
        let x = Term::Variable("X".to_owned());
        let y = Term::Variable("Y".to_owned());
        let z = Term::Variable("Z".to_owned());
        let a = Term::Function("a".to_owned(), Vec::new());
        let b = Term::Function("b".to_owned(), Vec::new());
        let f_y = Term::Function("f".to_owned(), vec!(y.clone()));
        let g_x_f_y_z = Term::Function("g".to_owned(), vec!(x.clone(), f_y.clone(), z.clone()));
        let f_f_f_b = Term::Function("f".to_owned(), vec!(Term::Function("f".to_owned(), vec!(Term::Function("f".to_owned(), vec!(b))))));
        
        let first_eq = Formula::Predicate("=".to_owned(), vec!(f_y.clone(), g_x_f_y_z));        
        let second_eq = Formula::Not(Box::new(Formula::Predicate("=".to_owned(), vec!(f_f_f_b, a))));  
        let third_eq = Formula::Predicate("=".to_owned(), vec!(x, f_y));
        
        let fm = Formula::Or(Box::new(Formula::Or(Box::new(first_eq), Box::new(second_eq))), Box::new(third_eq));
        
        assert_eq!(res.unwrap(), ("equality".to_owned(), "axiom".to_owned(), fm));
    }
    
    #[test]
    fn parse_cnf_annotated_true_false() {
        let res = parse_cnf_annotated("cnf(true_false,axiom,( $true | $false )).");
        
        let t = Formula::Predicate("$true".to_owned(), Vec::new());
        let f = Formula::Predicate("$false".to_owned(), Vec::new());
        let fm = Formula::Or(Box::new(t), Box::new(f));
        
        assert_eq!(res.unwrap(), ("true_false".to_owned(), "axiom".to_owned(), fm));
    }
    
    #[test]
    fn parse_cnf_annotated_single_quoted() {
        let res = parse_cnf_annotated("cnf(single_quoted,axiom,( 'A proposition' | 'A predicate'(Y) | p('A constant') | p('A function'(a)) | p('A \\'quoted \\ escape\\'') )).");
        
        let first = Formula::Predicate("A proposition".to_owned(), Vec::new());
        let second = Formula::Predicate("A predicate".to_owned(), vec!(Term::Variable("Y".to_owned())));
        let third = Formula::Predicate("p".to_owned(), vec!(Term::Function("A constant".to_owned(), Vec::new())));
        let fourth = Formula::Predicate("p".to_owned(), vec!(Term::Function("A function".to_owned(), vec!(Term::Function("a".to_owned(), Vec::new())))));
        let fifth = Formula::Predicate("p".to_owned(), vec!(Term::Function("A \\'quoted \\ escape\\'".to_owned(), Vec::new())));
        let fm = Formula::Or(Box::new(Formula::Or(Box::new(Formula::Or(Box::new(Formula::Or(Box::new(first), Box::new(second))), Box::new(third))), Box::new(fourth))), Box::new(fifth));
        
        assert_eq!(res.unwrap(), ("single_quoted".to_owned(), "axiom".to_owned(), fm));
    }
    
    #[test]
    fn parse_cnf_annotated_distinct_object() {
        let res = parse_cnf_annotated("cnf(distinct_object,axiom,( \"An Apple\" != \"A \\\"Microsoft \\ escape\\\"\" )).");
        
        let fst_p = Term::Function("An Apple".to_owned(), Vec::new());
        let snd_p = Term::Function("A \\\"Microsoft \\ escape\\\"".to_owned(), Vec::new());
        let fm = Formula::Not(Box::new(Formula::Predicate("=".to_owned(), vec!(fst_p, snd_p))));
        
        assert_eq!(res.unwrap(), ("distinct_object".to_owned(), "axiom".to_owned(), fm));
    }
    
    #[test]
    fn parse_cnf_annotated_integers() {
        let res = parse_cnf_annotated("cnf(integers,axiom,( p(12) | p(-12) )).").unwrap();
        
        let fst_p = Formula::Predicate("p".to_owned(), vec!(Term::Function("12".to_owned(), Vec::new())));
        let snd_p = Formula::Predicate("p".to_owned(), vec!(Term::Function("-12".to_owned(), Vec::new())));
        let fm = Formula::Or(Box::new(fst_p), Box::new(snd_p));
        
        assert_eq!(res, ("integers".to_owned(), "axiom".to_owned(), fm));
    }
    
    #[test]
    fn parse_cnf_annotated_rationals() {
        let res = parse_cnf_annotated("cnf(rationals ,axiom , ( p(123/456) | p(-123/456) | p(+123/456) )).").unwrap();
        
        let fst_p = Formula::Predicate("p".to_owned(), vec!(Term::Function("123/456".to_owned(), Vec::new())));
        let snd_p = Formula::Predicate("p".to_owned(), vec!(Term::Function("-123/456".to_owned(), Vec::new())));
        let trd_p = Formula::Predicate("p".to_owned(), vec!(Term::Function("+123/456".to_owned(), Vec::new())));
        let fm = Formula::Or(Box::new(Formula::Or(Box::new(fst_p), Box::new(snd_p))), Box::new(trd_p));
        
        assert_eq!(res, ("rationals".to_owned(), "axiom".to_owned(), fm));
    }
    
    #[test]
    fn parse_cnf_annotated_reals() {
        let res = parse_cnf_annotated("cnf(reals,axiom,
                                           ( p(123.456 )
                                           | p(-123.456 )
                                           | p(123.456E789 )
                                           | p(123.456e789 )
                                           | p(-123.456E789 )
                                           | p(123.456E-789 )
                                           | p(-123.456E-789 ) )).").unwrap();
        
        let p1 = Formula::Predicate("p".to_owned(), vec!(Term::Function("123.456".to_owned(), Vec::new())));
        let p2 = Formula::Predicate("p".to_owned(), vec!(Term::Function("-123.456".to_owned(), Vec::new())));
        let p3 = Formula::Predicate("p".to_owned(), vec!(Term::Function("123.456E789".to_owned(), Vec::new())));
        let p4 = Formula::Predicate("p".to_owned(), vec!(Term::Function("123.456e789".to_owned(), Vec::new())));
        let p5 = Formula::Predicate("p".to_owned(), vec!(Term::Function("-123.456E789".to_owned(), Vec::new())));
        let p6 = Formula::Predicate("p".to_owned(), vec!(Term::Function("123.456E-789".to_owned(), Vec::new())));
        let p7 = Formula::Predicate("p".to_owned(), vec!(Term::Function("-123.456E-789".to_owned(), Vec::new())));
        let fm = Formula::Or(Box::new(Formula::Or(Box::new(Formula::Or(Box::new(Formula::Or(Box::new(Formula::Or(Box::new(Formula::Or(Box::new(p1), Box::new(p2))), Box::new(p3))), Box::new(p4))), Box::new(p5))), Box::new(p6))), Box::new(p7));
        
        assert_eq!(res, ("reals".to_owned(), "axiom".to_owned(), fm));
    }
  
    #[test]
    fn parse_include_test() {
        assert_eq!(parse_include("include('Axioms/SYN000-0.ax').").unwrap(), ("Axioms/SYN000-0.ax".to_owned(), None));
        assert_eq!(parse_include("include('Axioms/SYN000+0.ax',[ia1,ia3]).").unwrap(), ("Axioms/SYN000+0.ax".to_owned(), Some(vec!("ia1".to_owned(), "ia3".to_owned()))));
    }

    #[test]
    fn parse_single_quoted_test() {
        assert_eq!(parse_single_quoted("'This is a single quoted string'").unwrap(), "This is a single quoted string");
        assert_eq!(parse_single_quoted("'A \\'quoted \\ escape\\''").unwrap(), "A \\'quoted \\ escape\\'");
        assert_eq!(parse_single_quoted("'The character \\' is quoted'").unwrap(), "The character \\' is quoted");
    }
    
    #[test]
    fn parse_distinct_object_1() {
        assert_eq!(parse_distinct_object("\"A \\\"Microsoft \\ escape\\\"\"").unwrap(), "A \\\"Microsoft \\ escape\\\"");
    }
    
    #[test]
    fn parse_distinct_object_2() {
        assert_eq!(parse_distinct_object(r#"" abc \""#).unwrap(), " abc \\");
    }
    
    #[test]
    fn dollar_word_test() {
        assert_eq!(parse_dollar_word("$aWord").unwrap(), "$aWord");
        assert!(parse_dollar_word("$ aWord").is_err());
    }
}

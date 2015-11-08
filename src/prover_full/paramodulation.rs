/*
    Serkr - An automated theorem prover. Copyright (C) 2015 Mikko Aarnos.

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

use std::collections::HashMap;
use prover_full::flatten_cnf::flatten_cnf;
use prover_full::term::Term;
use prover_full::literal::Literal;
use prover_full::clause::Clause;
use prover_full::tautology_deletion::trivial;
use prover_full::literal_deletion::simplify;
use prover_full::subsumption::subsumes_clause;
use prover_full::unification::mgu;
use parser::internal_parser::parse;
use cnf::naive_cnf::cnf;
use utils::formula::Formula;
use utils::stopwatch::Stopwatch;

fn overlaps(l: &Term, r: &Term, t: &Term, rfn: &Fn(HashMap<Term, Term>, Term) -> Clause, paramodulants: &mut Vec<Clause>) {
    if t.is_function() {
        if let Ok(theta) = mgu(l.clone(), t.clone()) {
            let new_cl = rfn(theta, r.clone());
            if !trivial(&new_cl) {
                paramodulants.push(new_cl);
            }
        }
        
    } else {
        // Paramodulating into variables is in general not necessary.
        // TODO: add a switch for doing that.
    }
}

fn overlaps_literal(eqn: &Literal, p: &Literal, rfn: &Fn(HashMap<Term, Term>, Literal) -> Clause, paramodulants: &mut Vec<Clause>) {
    let lhs_rfn = |theta, lhs| { rfn(theta, Literal::new(!p.is_positive(), lhs, p.get_rhs().clone())) };
    let rhs_rfn = |theta, rhs| { rfn(theta, Literal::new(!p.is_positive(), p.get_lhs().clone(), rhs)) };
    // s = t
    overlaps(eqn.get_lhs(), eqn.get_rhs(), p.get_lhs(), &lhs_rfn, paramodulants);
    overlaps(eqn.get_lhs(), eqn.get_rhs(), p.get_rhs(), &rhs_rfn, paramodulants);
    // t = s
    overlaps(eqn.get_rhs(), eqn.get_lhs(), p.get_lhs(), &lhs_rfn, paramodulants);
    overlaps(eqn.get_rhs(), eqn.get_lhs(), p.get_rhs(), &rhs_rfn, paramodulants);
}

/// Assumes that cl1 was renamed so that it can have no variables in common with anything else.
fn paramodulate_clauses(cl1: &Clause, cl2: &Clause, paramodulants: &mut Vec<Clause>) {
    for (i, l1) in cl1.iter().enumerate() {
        if l1.is_positive() {
            let mut c = cl1.clone();
            c.swap_remove(i);
            for (j, l2) in cl2.iter().enumerate() {
                let mut d = cl2.clone();
                d.swap_remove(j);
                let rfn = |theta, l| { let mut ret = c.clone(); 
                                       ret.add_literals(d.clone()); 
                                       ret.add_literal(l);
                                       ret.subst(&theta); 
                                       ret };
                overlaps_literal(l1, l2, &rfn, paramodulants);
            }
        }
    }
}

/// Rename a clause so that it contains no variables in common with any other clause we currently have.
fn rename_clause(cl: &mut Clause, var_cnt: &mut i64) {
    let mut var_map = HashMap::<i64, i64>::new();
    for l in cl.iter_mut() {
        l.rename_no_common(&mut var_map, var_cnt);
    }
}

/// Picks and removes the best clause from the unused clauses according to heuristics.
/// Currently just picks the shortest one.
fn pick_clause(unused: &mut Vec<Clause>) -> Clause {
    // TODO: use a priority queue instead of this
    let mut best_clause_index = 0;
    let mut best_clause_size = unused[0].size();
    
    for i in 1..unused.len() {
        if unused[i].size() < best_clause_size {
            best_clause_index = i;
            best_clause_size = unused[i].size();
        }
    }
    
    unused.swap_remove(best_clause_index)
}

fn paramodulation_loop(mut used: Vec<Clause>, mut unused: Vec<Clause>, mut var_cnt: i64) -> Result<bool, &'static str> {
    let mut sw = Stopwatch::new();
    let mut ms_count = 1000;
    sw.start();
    
    while !unused.is_empty() {
        if sw.elapsed_ms() > ms_count {
            println!("{} seconds have elapsed, used clauses = {}, unused clauses = {}", sw.elapsed_ms() / 1000, used.len(), unused.len());
            ms_count += 1000;
        }
        
        let mut chosen_clause = pick_clause(&mut unused);
        simplify(&mut chosen_clause);
        // If we derived a contradiction we are done.
        if chosen_clause.is_empty() {
            return Ok(true);
        }
        
        if !used.iter().any(|cl| subsumes_clause(cl, &chosen_clause)) {
            used.push(chosen_clause.clone());
            rename_clause(&mut chosen_clause, &mut var_cnt);
            
            let mut paramodulants = Vec::new();
            for cl in &used {
                paramodulate_clauses(&chosen_clause, cl, &mut paramodulants);
                paramodulate_clauses(cl, &chosen_clause, &mut paramodulants);
            }
            unused.append(&mut paramodulants);
        }
    }
    
    Err("No proof found.")
}

/// Attempts to prove the FOL formula passed in.
pub fn prove(s: &str) -> Result<bool, &'static str> {
    let cnf_f = cnf(Formula::Not(box parse(s).unwrap()));
    if cnf_f == Formula::False {
        Ok(true)
    } else if cnf_f == Formula::True {
        Ok(false)
    } else {
        let (flattened_cnf_f, renaming_info) = flatten_cnf(cnf_f);
        println!("{:?}", flattened_cnf_f.iter().filter(|cl| !trivial(cl)).collect::<Vec<_>>());
        paramodulation_loop(Vec::new(), flattened_cnf_f.into_iter().filter(|cl| !trivial(cl)).collect(), renaming_info.var_cnt)
    }
}

#[cfg(test)]
mod test {
    use super::prove;
    
    // Contains some pelletier problems negated so that we can make sure we don't prove a theorem not actually provable.
    
    #[test]
    fn pelletier_1() {
        let result = prove("((P ==> Q) <=> (~Q ==> ~P))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_1_negated() {
        let result = prove("~((P ==> Q) <=> (~Q ==> ~P))");
        assert!(result.is_err());
    }
    
    #[test]
    fn pelletier_2() {
        let result = prove("(~~P <=> P)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_3() {
        let result = prove("(~(P ==> Q) ==> (Q ==> P))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_4() {
        let result = prove("((~P ==> Q) <=> (~Q ==> P))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_5() {
        let result = prove("(((P \\/ Q) ==> (P \\/ R)) ==> (P \\/ (Q ==> R)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_6() {
        let result = prove("(P \\/ ~P)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_7() {
        let result = prove("(P \\/ ~~~P)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_8() {
        let result = prove("(((P ==> Q) ==> P) ==> P)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_8_negated() {
        let result = prove("~(((P ==> Q) ==> P) ==> P)");
        assert!(result.is_err());
    }
} 

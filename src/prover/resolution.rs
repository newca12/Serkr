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
use prover::unification::mgu;
use prover::literal::Literal;
use prover::clause::Clause;
use prover::trivial::trivial;
use prover::factoring::factor;
use prover::flatten_cnf::flatten_cnf;
use prover::subsumption::subsumes_clause;
use prover::duplicate_deletion::delete_duplicates;
use parser::internal_parser::parse;
use cnf::naive_cnf::cnf;
use utils::formula::Formula;
use utils::stopwatch::Stopwatch;

fn add_resolvents(cl1: &Clause, cl2: &Clause, p: Literal, used: &[Clause], unused: &mut Vec<Clause>) {
    let neg_p = p.negate();
    for x in cl2.iter() {
        if let Ok(theta) = mgu(neg_p.clone(), x.clone()) {
            assert!(mgu(p.clone(), x.negate()).unwrap() == theta);
            let mut cl1_done = Clause::new_from_vec(cl1.iter()
                                                       .cloned()
                                                       .filter(|l| *l != p)
                                                       .map(|mut l| { l.subst(&theta); l })
                                                       .collect());
            let cl2_done = Clause::new_from_vec(cl2.iter()
                                                   .cloned()
                                                   .filter(|l| *l != *x)
                                                   .map(|mut l| { l.subst(&theta); l })
                                                   .collect());                                   
            cl1_done.add_literals(cl2_done);
            delete_duplicates(&mut cl1_done);
            if !trivial(&cl1_done) && !used.iter().any(|cl| subsumes_clause(cl, &cl1_done)) && !unused.iter().any(|cl| subsumes_clause(cl, &cl1_done)) {
                unused.push(cl1_done);
            }
        }
    }
}

/// Assumes that cl1 was renamed so that it can have no variables in common with anything else.
fn resolve_clauses(cl1: &Clause, cl2: &Clause, used: &[Clause], unused: &mut Vec<Clause>) {
    // Positive resolution: one of the resolution clauses must be all-positive.
    if cl1.iter().all(|l| l.is_positive()) || cl2.iter().all(|l| l.is_positive()) {
        for p in cl1.iter().cloned() {
            add_resolvents(&cl1, &cl2, p, used, unused);
        }
    }
}

fn rename_clause(cl: &mut Clause, var_cnt: &mut i64) {
    let mut var_map = HashMap::<i64, i64>::new();
    for l in cl.iter_mut() {
        l.rename_no_common(&mut var_map, var_cnt);
    }
}

/// Picks and removes the best clause from the unused clauses according to heuristics.
/// Currently just picks the shortest one.
fn pick_clause(unused: &mut Vec<Clause>) -> Clause {
    // TODO: can be done better by using max.
    // TODO: can be done even better with a priority queue
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

fn resolution_loop(mut used: Vec<Clause>, mut unused: Vec<Clause>, mut var_cnt: i64) -> Result<bool, &'static str> {
    let mut sw = Stopwatch::new();
    let mut ms_count = 10000;
    sw.start();
    
    while !unused.is_empty() {
        if sw.elapsed_ms() > ms_count {
            println!("{} seconds have elapsed, used clauses = {}  and unused clauses = {}", sw.elapsed_ms() / 1000, used.len(), unused.len());
            ms_count += 10000;
        }
        
        let mut chosen_clause = pick_clause(&mut unused);
        // If we derived a contradiction we are done.
        if chosen_clause.is_empty() {
            return Ok(true);
        }
        // println!("Chosen clause: {:?}", chosen_clause);
        used.push(chosen_clause.clone());
        
        rename_clause(&mut chosen_clause, &mut var_cnt);
        for cl in &used {
            resolve_clauses(&chosen_clause, cl, &used, &mut unused);
        }
        factor(chosen_clause, &mut unused);
    }
    Err("No proof found.")
}

/// Attempts to parse and prove the string passed in.
pub fn resolution(s: &str) -> Result<bool, &'static str> {
    let cnf_f = cnf(Formula::Not(box parse(s).unwrap()));
    if cnf_f == Formula::False {
        Ok(true)
    } else if cnf_f == Formula::True {
        Ok(false)
    } else {
        let (flattened_cnf_f, renaming_info) = flatten_cnf(cnf_f);
        resolution_loop(Vec::new(), flattened_cnf_f.into_iter().filter(|cl| !trivial(cl)).collect(), renaming_info.var_cnt)
    }
}

#[cfg(test)]
mod test {
    use super::resolution;
    
    #[test]
    fn unprovable() {
        let result = resolution("(P ==> (Q ==> R))");
        assert!(result.is_err());
    }
    
    #[test]
    fn pelletier_1() {
        let result = resolution("((P ==> Q) <=> (~Q ==> ~P))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_2() {
        let result = resolution("(~~P <=> P)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_3() {
        let result = resolution("(~(P ==> Q) ==> (Q ==> P))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_4() {
        let result = resolution("((~P ==> Q) <=> (~Q ==> P))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_5() {
        let result = resolution("(((P \\/ Q) ==> (P \\/ R)) ==> (P \\/ (Q ==> R)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_6() {
        let result = resolution("(P \\/ ~P)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_7() {
        let result = resolution("(P \\/ ~~~P)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_8() {
        let result = resolution("(((P ==> Q) ==> P) ==> P)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_9() {
        let result = resolution("((((P \\/ Q) /\\ (~P \\/ Q)) /\\ (P \\/ ~Q)) ==> ~(~P \\/ ~Q))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_10() {
        let result = resolution("(((((Q ==> R) /\\ (R ==> (P /\\ Q))) /\\ (P ==> (Q \\/ R)))) ==> (P <=> Q))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_11() {
        let result = resolution("(P <=> P)");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_12() {
        let result = resolution("(((P <=> Q) <=> R) <=> (P <=> (Q <=> R)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_13() {
        let result = resolution("((P \\/ (Q /\\ R)) <=> ((P \\/ Q) /\\ (P \\/ R)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_14() {
        let result = resolution("((P <=> Q) <=> ((~P \\/ Q) /\\ (~Q \\/ P)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_15() {
        let result = resolution("((P ==> Q) <=> (~P \\/ Q))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_16() {
        let result = resolution("((P ==> Q) \\/ (Q ==> P))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_17() {
        let result = resolution("(((P /\\ (Q ==> R)) ==> S) <=>
                                  (((~P \\/ Q) \\/ S) /\\
                                  (~P \\/ (~R \\/ S))))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_18() {
        let result = resolution("exists y. forall x. (F(y) ==> F(x))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_19() {
        let result = resolution("exists x. forall y. forall z. ((P(y) ==> Q(z)) ==> (P(x) ==> Q(x)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_21() {
        let result = resolution("(((exists x. (P ==> F(x))) /\\ 
                                   (exists x. (F(x) ==> P))) 
                                    ==> (exists x. (P <=> F(x))))");
        assert!(result.is_ok());
    }

    #[test]
    fn pelletier_22() {
        let result = resolution("((forall x. (P <=> F(x))) ==> (P <=> forall x. F(x)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_23() {
        let result = resolution("((forall x. (P \\/ F(x))) <=> (P \\/ forall x. F(x)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_24() {
        let result = resolution("(((((~exists x. (S(x) /\\ Q(x))) /\\
                                     (forall x. (P(x) ==> (Q(x) \\/ R(x))))) /\\
                                    ((~exists x. P(x)) ==> exists x. Q(x))) /\\
                                     (forall x. ((Q(x) \\/ R(x)) ==> S(x))))
                                      ==> exists x. (P(x) /\\ R(x)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_25() {
        let result = resolution("(((((exists x. P(x)) /\\ 
                                   (forall x. (F(x) ==> (~G(x) /\\ R(x))))) /\\
                                   (forall x. (P(x) ==> (G(x) /\\ F(x))))) /\\
                                   ((forall x. (P(x) ==> Q(x))) \\/ exists x. (P(x) /\\ R(x))))
                                      ==> (exists x. (Q(x) /\\ P(x))))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_26() {
        let result = resolution("((((exists x. P(x)) <=> (exists x. Q(x))) /\\
                                    (forall x. forall y. ((P(x) /\\ Q(y)) ==> (R(x) <=> S(y)))))
                                 ==> ((forall x. (P(x) ==> R(x))) <=> (forall x. (Q(x) ==> S(x)))))");
        assert!(result.is_ok());
    }
    
   #[test]
   fn pelletier_30() {
        let result = resolution("(((forall x. ((F(x) \\/ G(x)) ==> ~H(x))) /\\ (forall x. ((G(x) ==> ~I(x)) ==> (F(x) /\\ H(x))))) ==> (forall x. I(x)))");
        assert!(result.is_ok());
    }

    #[test]
    fn pelletier_35() {
        let result = resolution("exists x. exists y. (P(x, y) ==> forall x. forall y. P(x, y))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_39() {
        let result = resolution("~exists x. forall y. (F(y, x) <=> ~F(y, y))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_40() {
        let result = resolution("((exists y. forall x. (F(x, y) <=> F(x, x))) ==>
                                  ~forall x. exists y. forall z. (F(x, y) <=> ~F(z, x)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_42() {
        let result = resolution("~exists y. forall x. (F(x, y) <=> ~exists z. (F(x, z) /\\ F(z, x)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn pelletier_44() {
        let result = resolution("(((forall x. (F(x) ==> (exists y. (G(y) /\\ H(x, y)) /\\ exists y. (G(y) /\\ ~H(x, y))))) /\\ 
                                    exists x. (J(x) /\\ forall y. (G(y) ==> H(x, y))))
                                    ==> exists x. (J(x) /\\ ~F(x)))");
        assert!(result.is_ok());
    }
    
    #[test]
    fn davis_putnam() {
        let result = resolution("exists x. exists y. forall z. ((F(x, y) ==> (F(y, z) /\\ F(z, z))) /\\ ((F(x, y) /\\ G(x, y)) ==> (G(x, z) /\\ G(z, z))))");
        assert!(result.is_ok());
    }
}
  
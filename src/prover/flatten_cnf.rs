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

use prover::term::Term as ProverTerm;
use prover::literal::Literal;
use prover::clause::Clause;
use cnf::ast::Term as CnfTerm;
use cnf::ast::Formula;

/// Turns a formula in CNF into a flat representation more suited for the prover.
/// Also converts the formula into pure equational logic.
/// We assume that the trivial cases of a formula being just True and False have been handled already.
pub fn flatten_cnf(f: Formula) -> Vec<Clause> {
    collect(f)
}

// TODO: clean this crap up.
fn collect(f: Formula) -> Vec<Clause> {
    match f {
        Formula::Predicate(s, args) => vec!(Clause::new(vec!(create_literal(false, s, args)))),
        Formula::Not(p) => match *p {
                               Formula::Predicate(ref s, ref args) =>  vec!(Clause::new(vec!(create_literal(true, s.clone(), args.clone())))),
                               _ => panic!("The CNF transformation failed due to some kind of a bug")
                           },
        Formula::Or(_, _) => vec!(collect_or(f)),
        Formula::And(p, q) => { let mut left = collect(*p); left.append(&mut collect(*q)); left }
        _ => panic!("The CNF transformation failed due to some kind of a bug")
    }
}

// TODO: clean this crap up.
fn collect_or(f: Formula) -> Clause {
    match f {
        Formula::Predicate(s, args) => Clause::new(vec!(create_literal(false, s, args))),
        Formula::Not(p) => match *p {
                               Formula::Predicate(ref s, ref args) =>  Clause::new(vec!(create_literal(true, s.clone(), args.clone()))),
                               _ => panic!("The CNF transformation failed due to some kind of a bug")
                           },
        Formula::Or(p, q) => { let mut left = collect_or(*p); left.add_literals(collect_or(*q)); left }
        _ => panic!("The CNF transformation failed due to some kind of a bug")
    }
}

fn create_literal(negated: bool, id: i64, args: Vec<CnfTerm>) -> Literal {
    if id == 0 {
        assert_eq!(args.len(), 2);
        Literal::new(negated, create_term(args[0].clone(), false), create_term(args[1].clone(), false))
    } else {
        Literal::new(negated, create_term(CnfTerm::Function(id, args), true), ProverTerm::new_truth())
    }
}

fn create_term(t: CnfTerm, special_fn: bool) -> ProverTerm {
    match t {
        CnfTerm::Variable(id) => ProverTerm::new_variable(id),
        CnfTerm::Function(id, args) => {
                let new_args = args.into_iter().map(|t2| create_term(t2, false)).collect();
                if special_fn {
                    ProverTerm::new_special_function(id, new_args)
                } else {
                    ProverTerm::new_function(id, new_args)
                }
            },
    }
}

#[cfg(test)]
mod test {
    
} 


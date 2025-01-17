// Serkr - An automated theorem prover. Copyright (C) 2015-2016 Mikko Aarnos.
//
// Serkr is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Serkr is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Serkr. If not, see <http://www.gnu.org/licenses/>.
//

use crate::cnf::ast::{Formula, Term};
use crate::cnf::renaming_info::RenamingInfo;

/// Renames variables so that different occurences of quantifiers bind different variables.
pub fn rename(f: Formula, ri: &mut RenamingInfo) -> Formula {
    match f {
        Formula::Not(p) => Formula::Not(Box::new(rename(*p, ri))),
        Formula::And(l) => Formula::And(l.into_iter().map(|x| rename(x, ri)).collect()),
        Formula::Or(l) => Formula::Or(l.into_iter().map(|x| rename(x, ri)).collect()),
        Formula::Forall(id, p) => rename_quantifier(id, *p, ri, true),
        Formula::Exists(id, p) => rename_quantifier(id, *p, ri, false),
        _ => f,
    }
}

fn rename_quantifier(
    id: i64,
    p: Formula,
    ri: &mut RenamingInfo,
    universal_quantifier: bool,
) -> Formula {
    let new_id = ri.create_new_variable_id();
    let renamed_p = rename(rename_variable(p, id, new_id), ri);

    if universal_quantifier {
        Formula::Forall(new_id, Box::new(renamed_p))
    } else {
        Formula::Exists(new_id, Box::new(renamed_p))
    }
}

/// Renames all occurrences of a single variable in a formula to another variable.
fn rename_variable(f: Formula, from: i64, to: i64) -> Formula {
    match f {
        Formula::Predicate(id, terms) => Formula::Predicate(
            id,
            terms
                .into_iter()
                .map(|t| rename_variable_in_term(t, from, to))
                .collect(),
        ),
        Formula::Not(p) => Formula::Not(Box::new(rename_variable(*p, from, to))),
        Formula::And(l) => Formula::And(
            l.into_iter()
                .map(|x| rename_variable(x, from, to))
                .collect(),
        ),
        Formula::Or(l) => Formula::Or(
            l.into_iter()
                .map(|x| rename_variable(x, from, to))
                .collect(),
        ),
        Formula::Forall(id, p) => Formula::Forall(
            if id == from { to } else { id },
            Box::new(rename_variable(*p, from, to)),
        ),
        Formula::Exists(id, p) => Formula::Exists(
            if id == from { to } else { id },
            Box::new(rename_variable(*p, from, to)),
        ),
        _ => f,
    }
}

/// Renames all occurrences of a single variable in a term to another variable.
fn rename_variable_in_term(t: Term, from: i64, to: i64) -> Term {
    match t {
        Term::Variable(id) => {
            if from == id {
                Term::Variable(to)
            } else {
                Term::Variable(id)
            }
        }
        Term::Function(id, args) => Term::Function(
            id,
            args.into_iter()
                .map(|t2| rename_variable_in_term(t2, from, to))
                .collect(),
        ),
    }
}

#[cfg(test)]
mod test {}

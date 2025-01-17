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

use crate::prover::data_structures::clause::Clause;
use crate::prover::inference::maximality::literal_maximal_in;
use crate::prover::ordering::term_ordering::TermOrdering;
use crate::prover::unification::full_unification::mgu;

/// Infers new clauses by (ordered) equality resolution.
/// Returns the amount of inferred clauses.
pub fn equality_resolution(
    term_ordering: &TermOrdering,
    cl: &Clause,
    generated: &mut Vec<Clause>,
) -> usize {
    let mut er_count = 0;

    for (i, l) in cl.iter().enumerate() {
        if l.is_negative() {
            if let Some(sigma) = mgu(l.get_lhs(), l.get_rhs()) {
                let mut new_cl = cl.clone();
                new_cl.subst(&sigma);
                let new_l = new_cl.swap_remove(i);

                assert_eq!(new_l.get_lhs(), new_l.get_rhs());
                assert_eq!(new_cl.size() + 1, cl.size());

                if literal_maximal_in(term_ordering, &new_cl, &new_l) {
                    generated.push(new_cl);
                    er_count += 1;
                }
            }
        }
    }

    er_count
}

#[cfg(test)]
mod test {}

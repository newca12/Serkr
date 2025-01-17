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
use std::cmp::Ordering;

/// Different ways to give weights to clauses.
/// Trying to order different variants in the enum results in a panic.
#[derive(PartialEq, Eq)]
pub enum ClauseWeight {
    /// Based on the size of the clause.
    /// Contains the ID and the symbol count of the clause.
    Size(u64, u64),
    /// Based on the age of the clause. Contains the ID of the clause.
    Age(u64),
}

impl ClauseWeight {
    /// Creates a new weight based on the symbol count of the clause.
    /// The variable 'f_value' is the value to give to function symbols.
    /// Then 'v_value' is just the value to give to variables.
    pub fn new_size_weight(cl: &Clause, f_value: u64, v_value: u64) -> ClauseWeight {
        ClauseWeight::Size(cl.get_id(), cl.symbol_count(f_value, v_value))
    }

    /// Creates a new weight based on the age of the clause.
    pub fn new_age_weight(clause: &Clause) -> ClauseWeight {
        ClauseWeight::Age(clause.get_id())
    }

    /// Get the ID of the clause a particular clause weight is associated with.
    pub fn get_id(&self) -> u64 {
        match *self {
            ClauseWeight::Size(id, _) | ClauseWeight::Age(id) => id,
        }
    }
}

impl PartialOrd for ClauseWeight {
    fn partial_cmp(&self, other: &ClauseWeight) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClauseWeight {
    fn cmp(&self, other: &ClauseWeight) -> Ordering {
        // Since BinaryHeap is a max heap, the comparison is inverted.
        match (self, other) {
            (&ClauseWeight::Size(_, weight1), &ClauseWeight::Size(_, weight2)) => {
                weight2.cmp(&weight1)
            }
            (&ClauseWeight::Age(id1), &ClauseWeight::Age(id2)) => id2.cmp(&id1),
            _ => panic!("faulty comparision"),
        }
    }
}

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

use crate::prover::data_structures::term::Term;
use crate::utils::hash_map::HashMap;
use std::collections::hash_map::IterMut;

/// Represents a substitution of from variables to terms.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Substitution {
    subst: HashMap<i64, Term>,
}

impl Substitution {
    /// Creates an empty substitution.
    pub fn new() -> Substitution {
        Substitution {
            subst: HashMap::default(),
        }
    }

    /// Used for getting the size of the substitution.
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.subst.len()
    }

    /// Inserts a new mapping from a variable to a term to the substitution.
    pub fn insert(&mut self, id: i64, t: Term) {
        assert!(id < 0);
        let res = self.subst.insert(id, t);
        assert!(res.is_none());
    }

    /// Get an (optional) reference to the term which is to be substituted for a given variable.
    pub fn get(&self, t: &Term) -> Option<&Term> {
        self.subst.get(&t.get_id())
    }

    /// Checks if the substitution is a variable renaming.
    pub fn is_variable_renaming(&self) -> bool {
        self.subst.iter().all(|(_, v)| v.is_variable())
    }

    /// Hack, remove.
    pub fn iter_mut(&mut self) -> IterMut<i64, Term> {
        self.subst.iter_mut()
    }
}

impl Default for Substitution {
    fn default() -> Substitution {
        Substitution::new()
    }
}

#[cfg(test)]
mod test {}

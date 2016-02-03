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

/// Contains functions for checking if a clause subsumes some other clause.
pub mod subsumption;

/// Contains functions for checking if a positive unit clause subsumes some ther clause.
pub mod equality_subsumption;

/// Contains functions for rewriting positive and negative literals.
pub mod rewriting;

/// Contains functions for performing positive and negative simplify-reflect simplifications.
pub mod simplify_reflect;

/// Contains functions for deleting unnecessary literals from clauses.
pub mod literal_deletion;

/// Contains functions for detecting tautologies.
pub mod tautology_deletion;
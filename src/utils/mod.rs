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

//! Contains all kinds of generally useful stuff (macros, timers etc.).

/// Contains some useful macros.
#[macro_use]
pub mod macros;

/// Contains a stopwatch-type timer for measuring time during program execution.
pub mod stopwatch;

/// Contains the Either type.
pub mod either;

/// Contains a `HashMap` more suitable for this program than the standard library one.
pub mod hash_map;

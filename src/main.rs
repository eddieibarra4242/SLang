/**
 *  SLang, shader language for SPIR-V
 *  Copyright (C) 2024  Eduardo Ibarra
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use clap::Parser;
use crate::scanner::Scanner;

mod scanner;

/// Recursive descent parser generator
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// Output file path
  #[arg(short, long)]
  output: Option<String>,

  /// input file path
  #[arg()]
  input: String,
}

fn main() {
  let cli_args = Args::parse();

  // Scan
  let mut scanner = Scanner::new(cli_args.input);
  let tokens = scanner.scan().expect("Failed to scan file!"); // fixme: print better error message and don't panic.

  for token in tokens {
    println!("{} {}", token.kind, token.value);
  }

  // Parse

  // Annotate

  // code gen
}

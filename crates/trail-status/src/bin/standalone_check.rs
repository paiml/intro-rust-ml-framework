//! Lesson 3.1.3: a program made by `apr compile` needs no model file.
//!
//! Moves the `.apr` file the program was compiled from out of the way, runs the
//! program with a question whose answer is known, then puts the file back
//! (even if the run fails: the move is undone when the guard is dropped).
//!
//! Provable contract: the model travels inside the program. With its `.apr`
//! file moved away, the compiled program still answers "What is 2+2?" with 4.
//!
//! Usage: standalone-check <compiled-program> <model.apr>

use std::path::{Path, PathBuf};
use std::process::Command;

/// Moves a file aside on creation and back on drop.
struct MovedAway {
    original: PathBuf,
    aside: PathBuf,
}

impl MovedAway {
    fn new(path: &Path) -> Self {
        let aside = path.with_extension("apr.away");
        std::fs::rename(path, &aside).unwrap_or_else(|e| panic!("move {} aside: {e}", path.display()));
        Self { original: path.to_path_buf(), aside }
    }
}

impl Drop for MovedAway {
    fn drop(&mut self) {
        let _ = std::fs::rename(&self.aside, &self.original);
    }
}

const QUESTION: &str = "What is 2+2? Answer with just the number.";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, program, model] = args.as_slice() else {
        eprintln!("usage: standalone-check <compiled-program> <model.apr>");
        std::process::exit(2);
    };
    let program = Path::new(program).canonicalize().expect("the compiled program");
    let model = Path::new(model);

    let answer = {
        let _guard = MovedAway::new(model);
        println!("moved {} away; it exists now: {}", model.display(), model.exists());
        assert!(!model.exists(), "the model file must be gone during the run");
        let out = Command::new(&program)
            .args(["-p", QUESTION, "-n", "16", "--no-gpu"])
            .output()
            .expect("run the compiled program");
        assert!(out.status.success(), "the compiled program failed");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    println!("asked: {QUESTION}");
    println!("answer: {answer}");
    println!("model file back in place: {}", model.exists());

    assert!(answer.contains('4'), "the compiled program must answer 4");
    assert!(model.exists(), "the model file must be restored");
    println!("contract: the model travels inside the program (answered 4 with its .apr moved away) OK");
}

use regex::Regex;

mod compiler;
mod runtime;


fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut compile = false;
    let mut infile = "hello.push";
    let mut outfile = "out.pushc";

    let push_file_regex  = Regex::new(".+\\.push$").unwrap();
    let pushc_file_regex = Regex::new(".+\\.pushc$").unwrap();

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        i += 1;

        match arg {
            "-c" => compile = true,
            "-o" => { outfile = args[i].as_str(); i += 1; },
            _ => {
                if infile.is_empty() {
                    infile = arg;
                } else {
                    todo!("unknown argument `{arg}`");
                }
            },
        }
    }

    if push_file_regex.is_match(infile) {
        let program = compiler::compile(&compiler::lex(
            std::fs::read_to_string(infile).unwrap().as_str()
        ));
        
        if compile {
            std::fs::write(outfile, &program).unwrap();
        }
        else {
            runtime::run(&program).unwrap();
        }
    }
    else if pushc_file_regex.is_match(infile) {
        let program = std::fs::read(infile).unwrap();
        
        runtime::run(&program).unwrap();
    }
}

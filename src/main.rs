use std::fs;

struct Token<'a> {
    word: &'a str,
    line_number: i32,
}

impl Token<'_> {
    fn print(&self) -> () {
        println!("{} {}", self.word, self.line_number);
    }
}

fn read_file(file_path: &str) -> String {
    let file_data = fs::read_to_string(file_path)
        .expect("Couldnt read the file, make sure the path is correct!");
    return file_data;
}

fn get_tokens(content: &String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    for (i, line) in content.lines().enumerate() {
        for word in line.split_whitespace() {
            tokens.push(Token {
                word,
                line_number: i as i32,
            });
        }
    }
    tokens
}

fn get_mutation_queries_from_tokens(tokens: Vec<Token>) -> Vec<Token> {
    let mut filtered_tokens: Vec<Token> = Vec::new();
    for token in tokens {
        match token.word {
            t if t.starts_with("use") && t.ends_with("Mutation()") => {
                println!("{}", token.word);
                filtered_tokens.push(token);
            }
            _ => {}
        }
    }
    filtered_tokens
}

fn main() {
    let file_data = read_file("graphql.ts");
    let tokens = get_tokens(&file_data);
    let mutation_tokens = get_mutation_queries_from_tokens(tokens);
    // for token in mutation_tokens {
    //     token.print();
    // }
}

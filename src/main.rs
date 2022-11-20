use std::{collections::HashMap, fs};

struct MutationToken {
    word: String,
    line_number: i32,
    mutation_variable: Option<String>,
}

impl MutationToken {
    fn print(&self) -> () {
        println!(
            "{} {} {:?}",
            self.word, self.line_number, self.mutation_variable
        );
    }
}

fn read_file(file_path: &str) -> String {
    let file_data = fs::read_to_string(file_path)
        .expect("Couldnt read the file, make sure the path is correct!");
    return file_data;
}

fn create_tokens(content: &String) -> (Vec<MutationToken>, HashMap<i32, String>) {
    let mut tokens: Vec<MutationToken> = Vec::new();
    let mut mutation_variables: Vec<String> = Vec::new();
    let mut content_by_line: HashMap<i32, String> = HashMap::new();
    for (i, line) in content.lines().enumerate() {
        content_by_line.insert(i as i32, String::from(line));
        for word in line.split_whitespace() {
            match word {
                t if t.starts_with("use") && t.ends_with("Mutation()") => {
                    tokens.push(MutationToken {
                        word: String::from(word),
                        line_number: i as i32,
                        mutation_variable: None,
                    });
                }
                t if t.ends_with("MutationVariables") => {
                    mutation_variables.push(String::from(word));
                }
                _ => {}
            }
        }
    }

    let mut updated_tokens: Vec<MutationToken> = Vec::new();

    for token in tokens {
        let related_mutation_variable = mutation_variables.iter().find(|&x| {
            *x.replace("MutationVariables", "")
                == token
                    .word
                    .clone()
                    .replace("use", "")
                    .replace("Mutation()", "")
        });
        match related_mutation_variable {
            Some(i) => updated_tokens.push(MutationToken {
                word: token.word,
                line_number: token.line_number,
                mutation_variable: Some(i.to_string()),
            }),
            None => {}
        }
    }

    (updated_tokens, content_by_line)
}

fn main() {
    let file_data = read_file("graphql.ts");
    let (tokens, map) = create_tokens(&file_data);

    for token in tokens {
        if token.mutation_variable.is_some() {
            // find main query
        }
    }
}

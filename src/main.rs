use std::{collections::HashMap, fmt::format, fs};

struct MutationToken {
    word: String,
    line_number: i32,
    mutation_variable: Option<String>,
    mutation_variable_line: Option<i32>,
}

impl MutationToken {
    fn print(&self) -> () {
        println!(
            "{} {} {:?} {:?}",
            self.word, self.line_number, self.mutation_variable, self.mutation_variable_line
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
                        mutation_variable_line: None,
                    });
                }
                t if t.ends_with("MutationVariables") => {
                    let content = format!("{}____{}", word, i);
                    mutation_variables.push(String::from(content));
                }
                _ => {}
            }
        }
    }

    let mut updated_tokens: Vec<MutationToken> = Vec::new();

    for token in tokens {
        let related_mutation_variable = mutation_variables.iter().find(|&x| {
            *x.replace("MutationVariables", "")
                .replace("____", "")
                .chars()
                .filter(|c| !c.is_digit(10))
                .collect::<String>()
                == token
                    .word
                    .clone()
                    .replace("use", "")
                    .replace("Mutation()", "")
        });

        match related_mutation_variable {
            Some(mutation_variable) => {
                let mut split_by_underscores = mutation_variable.split("____");
                let mutation_variable_itself = split_by_underscores.next().unwrap();
                let mutation_variable_line = split_by_underscores.next().unwrap();
                updated_tokens.push(MutationToken {
                    word: token.word,
                    line_number: token.line_number,
                    mutation_variable: Some(mutation_variable_itself.to_string()),
                    mutation_variable_line: Some(mutation_variable_line.parse::<i32>().unwrap()),
                });
            }
            None => {}
        }
    }

    (updated_tokens, content_by_line)
}

fn get_mutation_details(tokens: &Vec<MutationToken>, map: &HashMap<i32, String>) {
    for token in tokens {
        let mut mutation_variable_line = token.mutation_variable_line.unwrap().clone();
        let mut end = false;
        let mut start = false;

        let start_token = "Exact<{";
        let end_token = "}>;";

        let mut extracted_root_value = String::new();

        let mut iterations = 0;

        while end == false && iterations < 50 {
            let related_line = map.get(&mutation_variable_line).unwrap();
            if related_line.contains(start_token) {
                start = true;
            }
            if start == true {
                extracted_root_value += related_line;
            }
            if start == true && related_line.contains(end_token) {
                end = true;
            }
            iterations += 1;
            mutation_variable_line += 1;
        }

        println!("{:?}", extracted_root_value);

        // iterate through next lines i.e mutation_variable_line++++ until "}>;" is found
        // println!("{:?}", related_line);
    }
}
fn main() {
    let file_data = read_file("graphql.ts");
    let (tokens, map) = create_tokens(&file_data);

    get_mutation_details(&tokens, &map);
}

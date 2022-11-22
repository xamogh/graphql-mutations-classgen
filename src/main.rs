use std::{collections::HashMap, fmt::format, fs, hash::Hash};

struct MutationToken {
    word: String,
    line_number: i32,
    mutation_variable: Option<String>,
    mutation_variable_line: Option<i32>,
}

#[derive(Debug)]
struct GeneratedMutationToken {
    mutation_name: String,
    mutation_line: i32,
    mutation_kv_variables: (String, String),
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

fn create_tokens(
    content: &String,
) -> (
    Vec<MutationToken>,
    HashMap<i32, String>,
    HashMap<String, i32>,
) {
    let mut tokens: Vec<MutationToken> = Vec::new();
    let mut mutation_variables: Vec<String> = Vec::new();
    let mut content_by_line: HashMap<i32, String> = HashMap::new();
    let mut line_by_word_token: HashMap<String, i32> = HashMap::new();
    for (i, line) in content.lines().enumerate() {
        content_by_line.insert(i as i32, String::from(line));
        for word in line.split_whitespace() {
            if !(line_by_word_token.contains_key(word)) {
                line_by_word_token.insert(String::from(word), i as i32);
            }
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
                    .chars()
                    // skip "use"
                    .skip(3)
                    .collect::<String>()
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

    (updated_tokens, content_by_line, line_by_word_token)
}

fn extract_between_tokens_from_line(
    start_token: &str,
    end_token: &str,
    start_line: i32,
    map: &HashMap<i32, String>,
) -> String {
    let mut mutation_variable_line = start_line.clone();
    let mut end = false;
    let mut start = false;

    let mut extracted_root_value = String::new();

    let mut iterations = 0;

    while end == false && iterations < 500 {
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

    let start_bytes = extracted_root_value.find(start_token).unwrap_or(0);
    let end_bytes = extracted_root_value
        .find(end_token)
        .unwrap_or(extracted_root_value.len());

    let in_between = &extracted_root_value[(start_bytes)..end_bytes].replace(start_token, "");
    in_between.to_string()
}

fn get_mutation_details(
    tokens: &Vec<MutationToken>,
    map: &HashMap<i32, String>,
    r_map: &HashMap<String, i32>,
    collector: &mut Vec<GeneratedMutationToken>,
) {
    for token in tokens {
        fn dox(
            token: &MutationToken,
            in_between: &str,
            map: &HashMap<i32, String>,
            r_map: &HashMap<String, i32>,
            collector: &mut Vec<GeneratedMutationToken>,
        ) {
            for t in in_between.split(";") {
                let mut split = t.split(":");
                let mut kv_tuple = (split.next().unwrap_or(""), split.next().unwrap_or(""));
                kv_tuple.0 = kv_tuple.0.trim();
                kv_tuple.1 = kv_tuple.1.trim();

                if !kv_tuple.0.is_empty() && !kv_tuple.1.is_empty() {
                    if kv_tuple.1.contains("Scalar") {
                        // no need to go deep
                        // println!("{:?} {:?}", kv_tuple, token.word);
                        collector.push(GeneratedMutationToken {
                            mutation_name: token.word.clone(),
                            mutation_line: token.line_number,
                            mutation_kv_variables: (kv_tuple.0.to_string(), kv_tuple.1.to_string()),
                        })
                    } else {
                        // go deep infinite
                        let in_between = extract_between_tokens_from_line(
                            "{",
                            "};",
                            r_map.get(kv_tuple.1).unwrap().clone(),
                            map,
                        );

                        dox(token, &in_between, map, r_map, collector);
                        println!("{:?}\n", in_between);
                    }
                    // println!("{:?}", kv_tuple);
                }
            }
        }

        let in_between = extract_between_tokens_from_line(
            "Exact<{",
            "}>;",
            token.mutation_variable_line.unwrap().clone(),
            map,
        );

        dox(token, &in_between, map, r_map, collector);

        // println!("{:?} --------------- {:?}", in_between, token.word);

        // iterate through next lines i.e mutation_variable_line++++ until "}>;" is found
        // println!("{:?}", related_line);
    }
}
fn main() {
    let file_data = read_file("graphql.ts");

    let (tokens, map, r_map) = create_tokens(&file_data);

    let mut gen_collector: Vec<GeneratedMutationToken> = Vec::new();

    get_mutation_details(&tokens, &map, &r_map, &mut gen_collector);

    for item in gen_collector {
        // println!("{:?} \n", item);
    }
}

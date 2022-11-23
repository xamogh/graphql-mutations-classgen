use std::io::prelude::*;
use std::{collections::HashMap, fs};

struct MutationToken {
    word: String,
    line_number: i32,
    mutation_variable_line: Option<i32>,
}

#[derive(Debug)]
struct GeneratedMutationToken {
    mutation_name: String,
    mutation_line: i32,
    // kev, value, parent
    mutation_kv_variables: (String, String, String),
}

#[derive(Clone, Debug)]
struct AggregatedMutationTokens {
    mutation_name: String,
    mutation_line: i32,
    mutation_kv_variables: Vec<(String, String, String)>,
}

impl AggregatedMutationTokens {
    fn add_to_mutation_kv_variables(&mut self, v: (String, String, String)) -> () {
        self.mutation_kv_variables.push(v);
    }

    fn create_import_statement(&self) -> String {
        let root_mutation_name = self.mutation_name.replace("()", "").replace("use", "");
        let import_statement_core = format!(
            "import {{{}, {}Variables, {}Document}} from './graphql';",
            root_mutation_name,
            root_mutation_name,
            root_mutation_name.replace("Mutation", ""),
        );
        import_statement_core
    }

    fn create_mutation(&self) -> String {
        let mutation_start = format!("export function safe_{} {{", self.mutation_name);
        let mutation_end = format!("}};");
        let root_mutation_name = self.mutation_name.replace("()", "").replace("use", "");
        let urql_mutation = format!(
            "const [m1, m2] = Urql.useMutation<{}, {}Variables>({}Document);",
            root_mutation_name,
            root_mutation_name,
            root_mutation_name.replace("Mutation", "")
        );

        let mut keys = String::new();

        for (i, tuple) in self.mutation_kv_variables.iter().enumerate() {
            if i == 0 {
                keys += &format!("'{}'", tuple.0.replace("?", ""));
            } else {
                keys += &format!(",'{}'", tuple.0.replace("?", ""));
            }
        }

        let injected_fn_and_ret = format!(
            "const m3 = (args: {}Variables) => {{
                const r = [{}];
                checkAndRemoveDeep(r, args);
                    return m2(args);
            }};
            return [m1, m3] as Urql.UseMutationResponse<
            {},
            {}Variables>;",
            root_mutation_name, keys, root_mutation_name, root_mutation_name
        );
        let rt = format!(
            "{}\n{}\n{}\n{}",
            &mutation_start, &urql_mutation, &injected_fn_and_ret, &mutation_end
        );
        rt
    }
}

fn read_file(file_path: &str, msg: &str) -> String {
    println!("{}", file_path);
    let file_data = fs::read_to_string(file_path).expect(msg);
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
                    .skip(3)
                    .collect::<String>()
                    .replace("Mutation()", "")
        });

        match related_mutation_variable {
            Some(mutation_variable) => {
                let mut split_by_underscores = mutation_variable.split("____");
                split_by_underscores.next().unwrap();
                let mutation_variable_line = split_by_underscores.next().unwrap();
                updated_tokens.push(MutationToken {
                    word: token.word,
                    line_number: token.line_number,
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
            parent: String,
        ) {
            for t in in_between.split(";") {
                let mut split = t.split(":");
                let mut kv_tuple = (split.next().unwrap_or(""), split.next().unwrap_or(""));
                kv_tuple.0 = kv_tuple.0.trim();
                kv_tuple.1 = kv_tuple.1.trim();

                if !kv_tuple.0.is_empty() && !kv_tuple.1.is_empty() {
                    if kv_tuple.1.contains("Scalar") {
                        // no need to go deep
                        collector.push(GeneratedMutationToken {
                            mutation_name: token.word.clone(),
                            mutation_line: token.line_number,
                            mutation_kv_variables: (
                                kv_tuple.0.to_string(),
                                kv_tuple.1.to_string(),
                                parent.clone(),
                            ),
                        })
                    } else {
                        let in_between = extract_between_tokens_from_line(
                            "{",
                            "};",
                            r_map.get(kv_tuple.1).unwrap().clone(),
                            map,
                        );
                        // go deep infinite
                        collector.push(GeneratedMutationToken {
                            mutation_name: token.word.clone(),
                            mutation_line: token.line_number,
                            mutation_kv_variables: (
                                kv_tuple.0.to_string(),
                                kv_tuple.1.to_string(),
                                parent.clone(),
                            ),
                        });
                        let new_parent;

                        if parent.is_empty() {
                            new_parent = format!("{}", kv_tuple.0.to_string());
                        } else {
                            new_parent = format!("{}[{}]", parent, kv_tuple.0.to_string());
                        }

                        dox(token, &in_between, map, r_map, collector, new_parent);
                    }
                }
            }
        }

        let in_between = extract_between_tokens_from_line(
            "Exact<{",
            "}>;",
            token.mutation_variable_line.unwrap().clone(),
            map,
        );

        dox(token, &in_between, map, r_map, collector, "".to_string());
    }
}

fn aggregate_generated_mutation_tokens(
    collector: Vec<GeneratedMutationToken>,
) -> Vec<AggregatedMutationTokens> {
    let mut aggregated_tokens: Vec<AggregatedMutationTokens> = Vec::new();

    for token in collector {
        let idx = aggregated_tokens.iter().position(|r| {
            r.mutation_name == token.mutation_name && r.mutation_line == token.mutation_line
        });
        if idx.is_none() {
            aggregated_tokens.push(AggregatedMutationTokens {
                mutation_name: token.mutation_name,
                mutation_line: token.mutation_line,
                mutation_kv_variables: [token.mutation_kv_variables].to_vec(),
            });
        } else {
            let index = idx.unwrap();
            let element = &mut aggregated_tokens[index];
            element.add_to_mutation_kv_variables(token.mutation_kv_variables);
        }
    }

    return aggregated_tokens;
}

fn write_constants(f: &mut fs::File) -> () {
    let core_imports = format!("import * as Urql from 'urql';");
    let js_remove_and_warn_deep_fn = "
    function checkAndRemoveDeep(toCheckKeys: Array<string>, arg: any) {
      if (!arg || typeof arg !== 'object' || Object.keys(arg).length === 0) return;
      Object.keys(arg).forEach((k) => {
        if (!toCheckKeys.includes(k)) {
          console.error('unwanted mutation key detected please remove it', arg, k);
          if (process.env.NODE_ENV !== 'production') {
              window.alert(
                `unwanted mutation key detected please remove it\nunwanted_key= '${k}'\non_object = ${JSON.stringify(arg)}`
              );
          }
          delete arg[k];
        }
        checkAndRemoveDeep(toCheckKeys, arg[k]);
      });
    };";
    f.write(core_imports.as_bytes()).unwrap();
    f.write(js_remove_and_warn_deep_fn.as_bytes()).unwrap();
}

fn main() {
    const CONFIG_FILE_PATH: &str = "safe-urqlcodgen-mutations.conf";
    let config_file = read_file(CONFIG_FILE_PATH, "Could not find config file.\n Please add a config file named safe-urqlcodegen-mutations.conf to your root directory. \n generated_path={path_of_your_codegen_generated_file}, eg: generated_path=src/@generated");
    let mut root_path = String::from("");
    for line in config_file.lines() {
        let mut splitter = line.split("=");
        let k = splitter.next().unwrap_or("");
        let v = splitter.next().unwrap_or("");
        if k == "generated_path" {
            root_path = String::from(v);
        }
    }

    let file_data = read_file(&format!("{}/graphql.ts", root_path), "failed to read urqlcodegen file");

    let (tokens, map, r_map) = create_tokens(&file_data);

    let mut generated_tokens_collector: Vec<GeneratedMutationToken> = Vec::new();

    get_mutation_details(&tokens, &map, &r_map, &mut generated_tokens_collector);

    let aggregated_tokens = aggregate_generated_mutation_tokens(generated_tokens_collector);

    let output_path = format!("{}/gen.ts", root_path);

    let mut output =
        fs::File::create(output_path).expect("Something went wrong, couldnt create a output file");

    write_constants(&mut output);

    for token in aggregated_tokens {
        output
            .write(token.create_import_statement().as_bytes())
            .unwrap();
        output.write(token.create_mutation().as_bytes()).unwrap();
    }
}

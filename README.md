# safe-urql-urqlcodegen-mutations

#### Reads codegen generated graphql.ts file, then catches all the mutations, enhances them with excess property filters and warnings, and writes into a new generated file.

#### How to run?
1) Place the binary in your project root directory. 
2) Create a safe-urqlcodgen-mutations.conf file at your root directory
3) Inside the conf file add a line: generated_path={your_codegen_generated_path}
  Eg: generated_path=src/@generated
4) Run the binary
5) Mutations should be emitted in the same path

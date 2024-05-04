#[cfg(test)]
mod project_compiler_tests {
    use std::{
        collections::BTreeMap,
        path::PathBuf,
        process::{Command, Stdio},
    };

    use crate::{passes_exclude, passes_scope, read_remappings};
    use foundry_compilers::{utils, CompilerInput, Graph, Project, ProjectPathsConfig};

    #[test]
    fn test_grouping_files_to_compile() {
        let project_root_str = "/Users/tilakmadichetti/Documents/OpenSource/ccip/contracts";
        let scope = &Some(vec!["src/v0.8/ccip".to_string()]);
        let exclude = &Some(vec!["tests/".to_string()]);

        // let scope: &Option<_> = &None;
        // let exclude: &Option<_> = &None;

        let root = utils::canonicalize(project_root_str).unwrap();

        let compiler_input = CompilerInput::new(&root).unwrap();
        let solidity_files = compiler_input
            .into_iter()
            .filter(|c| c.language == *"Solidity")
            .collect::<Vec<_>>();
        let solidity_files = &solidity_files[0]; // No Yul Support as of now

        let paths = ProjectPathsConfig::builder().root(&root).build().unwrap();
        let project = Project::builder()
            .no_artifacts()
            .paths(paths)
            .ephemeral()
            .build()
            .unwrap();

        println!("Collecting sources in scope");
        let sources = solidity_files
            .sources
            .iter()
            .filter(|(solidity_file, _)| {
                passes_scope(
                    scope,
                    solidity_file.canonicalize().unwrap().as_path(),
                    &root.to_string_lossy().to_string(),
                )
            })
            .filter(|(solidity_file, _)| {
                passes_exclude(
                    exclude,
                    solidity_file.canonicalize().unwrap().as_path(),
                    &root.to_string_lossy().to_string(),
                )
            })
            .map(|(x, y)| (x.to_owned(), y.to_owned()))
            .collect::<BTreeMap<_, _>>();

        // println!("Sources: {:?}", sources.keys().cloned());

        println!("Resolving sources versions by graph ...");
        let graph = Graph::resolve_sources(&project.paths, sources).unwrap();
        let (versions, _) = graph.into_sources_by_version(project.offline).unwrap();

        let mut remappings = vec![];
        if let Some(custom_remappings) = read_remappings(&root) {
            remappings.extend(custom_remappings);
            remappings.dedup();
        }

        let sources_by_version = versions.get(&project).unwrap();
        for (solc, value) in sources_by_version {
            // let version = value.0;
            // let paths = value.1.keys().map(|x| x.display()).collect::<Vec<_>>();
            // println!("{} - \n{:?}\n\n", version, paths);
            println!("Compiling {} files with Solc {}", value.1.len(), value.0);
            let solc_bin = solc.solc.to_str().unwrap();
            println!("{solc_bin}");

            let files = value.1.keys().cloned().collect::<Vec<_>>();

            print_running_command(&remappings, &files);

            let command = Command::new(solc_bin)
                .args(remappings.clone())
                .arg("--ast-compact-json")
                .args(files)
                .current_dir(root.clone())
                .stdout(Stdio::piped())
                .output();
            if let Ok(command) = command {
                if !command.status.success() {
                    let msg = String::from_utf8(command.stderr).unwrap();
                    println!("stderr = {}", msg);
                    println!("cwd = {}", root.display());
                    panic!("Error running solc command");
                }
                let _stdout = String::from_utf8(command.stdout).unwrap();
                // println!("{}", _stdout);
            }
        }
    }

    fn print_running_command(remappings: &Vec<String>, files: &Vec<PathBuf>) {
        let mut command = String::new();
        command.push_str("solc --ast-compact-json ");
        for remap in remappings {
            command.push_str(&format!("{} ", remap));
        }
        for file in files {
            command.push_str(&format!("{} ", file.to_string_lossy().to_string()));
        }
        println!("{}", command);
    }
}

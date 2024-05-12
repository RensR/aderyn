use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    str::FromStr,
};

use foundry_compilers::{
    artifacts::Source, remappings::Remapping, CompilerInput, Project, ProjectPathsConfig,
};

use crate::{passes_exclude, passes_scope, passes_src, read_remappings};

/// CompilerInput is a module that allows us to locate all solidity files in a root
pub fn get_compiler_input(root: &Path) -> CompilerInput {
    let compiler_input = CompilerInput::new(root).unwrap();
    let solidity_files = compiler_input
        .into_iter()
        .filter(|c| c.language == *"Solidity")
        .collect::<Vec<_>>();
    let solidity_files = &solidity_files[0]; // No Yul Support as of now
    solidity_files.to_owned()
}

/// We retrieve the remappings in 2 styles. Both have their own use
pub fn get_remappings(root: &Path) -> (Vec<String>, Vec<Remapping>) {
    let mut remappings = vec![];
    if let Some(custom_remappings) = read_remappings(root) {
        remappings.extend(custom_remappings);
        remappings.dedup();
    }

    let foundry_compilers_remappings = remappings
        .iter()
        .filter_map(|x| Remapping::from_str(x).ok())
        .collect::<Vec<_>>();

    (remappings, foundry_compilers_remappings)
}

pub fn get_project(root: &Path, remappings: Vec<Remapping>) -> Project {
    let paths = ProjectPathsConfig::builder()
        .root(root)
        .remappings(remappings)
        .build()
        .unwrap();
    Project::builder()
        .no_artifacts()
        .paths(paths)
        .ephemeral()
        .build()
        .unwrap()
}

pub fn get_relevant_sources(
    root: &Path,
    solidity_files: CompilerInput,
    src: &Option<Vec<PathBuf>>,
    scope: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
) -> BTreeMap<PathBuf, Source> {
    solidity_files
        .sources
        .iter()
        .filter(|(solidity_file, _)| {
            passes_src(src, solidity_file.canonicalize().unwrap().as_path())
        })
        .filter(|(solidity_file, _)| {
            passes_scope(
                scope,
                solidity_file.canonicalize().unwrap().as_path(),
                root.to_string_lossy().as_ref(),
            )
        })
        .filter(|(solidity_file, _)| {
            passes_exclude(
                exclude,
                solidity_file.canonicalize().unwrap().as_path(),
                root.to_string_lossy().as_ref(),
            )
        })
        .map(|(x, y)| (x.to_owned(), y.to_owned()))
        .collect::<BTreeMap<_, _>>()
}

pub fn get_relevant_pathbufs(
    root: &Path,
    pathbufs: &[PathBuf],
    src: &Option<Vec<PathBuf>>,
    scope: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
) -> Vec<PathBuf> {
    pathbufs
        .iter()
        .filter(|solidity_file| passes_src(src, solidity_file.canonicalize().unwrap().as_path()))
        .filter(|solidity_file| {
            passes_scope(
                scope,
                solidity_file.canonicalize().unwrap().as_path(),
                root.to_string_lossy().as_ref(),
            )
        })
        .filter(|solidity_file| {
            passes_exclude(
                exclude,
                solidity_file.canonicalize().unwrap().as_path(),
                root.to_string_lossy().as_ref(),
            )
        })
        .map(|x| x.to_owned())
        .collect::<Vec<_>>()
}
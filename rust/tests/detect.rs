//! Auto-Search for all games: which game a found executable belongs to.

use std::path::Path;

use openase::detect::owners;

fn ids(path: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    let found = owners(Path::new(path));
    let sure = found
        .iter()
        .filter(|(_, s)| *s)
        .map(|(g, _)| g.id.as_str())
        .collect();
    let unsure = found
        .iter()
        .filter(|(_, s)| !*s)
        .map(|(g, _)| g.id.as_str())
        .collect();
    (sure, unsure)
}

#[test]
fn an_executable_of_one_game_is_sure() {
    let (sure, unsure) = ids("E:/Games/Quake III Arena/quake3.exe");
    assert_eq!(sure, ["q3"]);
    assert!(unsure.is_empty());
}

#[test]
fn a_shared_executable_goes_to_the_game_whose_folder_is_in_the_path() {
    let (sure, unsure) = ids("D:/Steam/steamapps/common/Counter-Strike Source/hl2.exe");
    assert_eq!(sure, ["css"]);
    assert!(unsure.contains(&"dods") && unsure.contains(&"gmod"));
    // Elsewhere nothing is sure: the user picks.
    let (sure, unsure) = ids("E:/Juegos/Source/hl2.exe");
    assert!(sure.is_empty());
    assert!(unsure.len() >= 4);
}

#[test]
fn file_names_match_case_insensitively_and_unknown_files_match_nothing() {
    assert_eq!(ids("E:/Games/Q3/QUAKE3.EXE").0, ["q3"]);
    assert_eq!(owners(Path::new("C:/Windows/notepad.exe")).len(), 0);
}

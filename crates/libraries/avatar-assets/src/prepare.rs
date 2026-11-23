use std::io;
use std::path::PathBuf;

use include_dir::Dir;

use crate::cast;
use crate::embedded::{self, write_directory, write_if_changed};
use crate::manifest::embedded_downloads;
use crate::platform::layout::DataDirectory;
use crate::resolve::Asset;

pub(crate) fn default_for(asset: Asset, data: &DataDirectory) -> io::Result<PathBuf> {
    match asset {
        Asset::RendererRoot => renderer(data),
        Asset::SeatPage => page(&embedded::SEAT_PAGE, data.join("seat-page")),
        Asset::WakePage => page(&embedded::WAKE_PAGE, data.join("wake-page")),
        Asset::CharacterVoices => cast_file(data, "character-voices.json", &cast::voices_json()?),
        Asset::ServantCharacters => cast_file(data, "servants.txt", embedded::SERVANT_CHARACTERS),
        Asset::VoiceDirectory => Ok(data.join("voices")),
        Asset::TurnModel | Asset::VoiceActivityModel | Asset::SpeakerModel => model(asset, data),
    }
}

pub(crate) fn renderer(data: &DataDirectory) -> io::Result<PathBuf> {
    let root = data.join("renderer");
    write_directory(&embedded::RENDERER, &root)?;
    write_renderer_data(&root)?;
    cast::link_models(&root.join("characters"))?;
    Ok(root)
}

fn write_renderer_data(root: &std::path::Path) -> io::Result<()> {
    let catalog = cast::catalog_json()?;
    write_if_changed(&root.join("characters.json"), catalog.as_bytes())?;
    write_if_changed(
        &root.join("portraits/jarvis.jpeg"),
        embedded::DEFAULT_PORTRAIT,
    )
}

fn page(source: &Dir<'_>, target: PathBuf) -> io::Result<PathBuf> {
    write_directory(source, &target)?;
    Ok(target)
}

fn cast_file(data: &DataDirectory, name: &str, contents: &str) -> io::Result<PathBuf> {
    let path = data.join("cast").join(name);
    write_if_changed(&path, contents.as_bytes())?;
    Ok(path)
}

fn download_name(asset: Asset) -> &'static str {
    match asset {
        Asset::TurnModel => "smart-turn",
        Asset::VoiceActivityModel => "silero-vad",
        _ => "speaker-embedding",
    }
}

fn model(asset: Asset, data: &DataDirectory) -> io::Result<PathBuf> {
    let path = downloaded(data, download_name(asset))?;
    path.is_file()
        .then_some(path.clone())
        .ok_or_else(|| missing(asset, &path))
}

fn missing(asset: Asset, path: &std::path::Path) -> io::Error {
    let message = format!(
        "{} not found at {}; set {} or run `avatar setup`",
        asset.label(),
        path.display(),
        asset.variable()
    );
    io::Error::new(io::ErrorKind::NotFound, message)
}

fn downloaded(data: &DataDirectory, name: &str) -> io::Result<PathBuf> {
    embedded_downloads()?
        .into_iter()
        .find(|download| download.name == name)
        .map(|download| data.join(&download.destination))
        .ok_or_else(|| io::Error::other(format!("no download named {name}")))
}

pub(crate) fn word_paths(data: &DataDirectory) -> String {
    let downloads = embedded_downloads().unwrap_or_default();
    let lists = downloads
        .iter()
        .filter(|download| download.destination.starts_with("words/"));
    let paths = lists.map(|download| data.join(&download.destination).display().to_string());
    paths.collect::<Vec<_>>().join(":")
}

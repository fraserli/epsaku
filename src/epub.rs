mod render;

use render::{render_node, RenderAttributes};

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use anyhow::{anyhow, bail, Context, Result};
use roxmltree::{Document, Node};
use zip::ZipArchive;

pub struct Epub {
    archive: ZipArchive<File>,
    manifest: HashMap<String, String>,
    spine: Vec<String>,
}

struct Container {
    pub package_path: String,
    pub base_path: String,
}

struct Package {
    pub manifest: HashMap<String, String>,
    pub spine: Vec<String>,
}

impl Epub {
    pub fn new(path: &str) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("unable to open '{path}'"))?;
        let mut archive = ZipArchive::new(file).with_context(|| format!("'{path}'"))?;

        let mimetype = read_archive(&mut archive, "mimetype")?;
        if mimetype.trim() != "application/epub+zip" {
            bail!("invalid mimetype: {mimetype}")
        }

        let container_xml = read_archive(&mut archive, "META-INF/container.xml")?;
        let Container {
            package_path,
            base_path,
        } = parse_container(&container_xml).context("failed to parse container")?;

        let package_xml = read_archive(&mut archive, &package_path)?;
        let Package { manifest, spine } =
            parse_package(&package_xml, &base_path).context("failed to parse package")?;

        Ok(Self {
            archive,
            manifest,
            spine,
        })
    }

    pub fn len(&self) -> usize {
        self.spine.len()
    }

    pub fn render(&mut self, index: usize) -> Result<String> {
        let id = &self.spine[index];
        let path = &self.manifest[id];
        let xml = read_archive(&mut self.archive, path)?;
        let doc = Document::parse(&xml)?;

        let text = render_node(doc.root(), RenderAttributes::default());

        Ok(text)
    }
}

fn parse_container(xml: &str) -> Result<Container> {
    let container = Document::parse(xml)?;

    let package_path =
        if let Some(rootfile_node) = find_node(container.root(), "container/rootfiles/rootfile") {
            rootfile_node
                .attribute("full-path")
                .ok_or_else(|| anyhow!("rootfile node missing full-path attribute"))?
                .to_string()
        } else {
            bail!("unable to find rootfile node");
        };

    let package_path_components: Vec<&str> = package_path.split('/').into_iter().collect();
    let (_, base_path_components) = package_path_components.split_last().unwrap_or((&"", &[]));
    let base_path = base_path_components.join("");

    Ok(Container {
        package_path,
        base_path,
    })
}

fn parse_package(xml: &str, base_path: &str) -> Result<Package> {
    let package = Document::parse(xml)?;

    let manifest_node = find_node(package.root(), "package/manifest")
        .ok_or_else(|| anyhow!("unable to find manifest node"))?;
    let manifest: HashMap<String, String> = manifest_node
        .children()
        .filter(|n| n.has_tag_name("item"))
        .map(|n| {
            (
                n.attribute("id").unwrap().to_owned(),
                if !base_path.is_empty() {
                    format!("{}/{}", base_path, n.attribute("href").unwrap())
                } else {
                    n.attribute("href").unwrap().to_owned()
                },
            )
        })
        .collect();

    let spine_node = find_node(package.root(), "package/spine")
        .ok_or_else(|| anyhow!("unable to find manifest node"))?;
    let spine: Vec<String> = spine_node
        .children()
        .filter(|n| {
            n.has_tag_name("itemref")
                && if let Some(linear) = n.attribute("linear") {
                    linear == "yes"
                } else {
                    true
                }
        })
        .map(|n| n.attribute("idref").unwrap().to_owned())
        .collect();

    Ok(Package { manifest, spine })
}

fn read_archive(archive: &mut ZipArchive<File>, path: &str) -> Result<String> {
    let mut buf = String::new();
    let mut archive_file = archive.by_name(path).with_context(|| path.to_string())?;
    archive_file.read_to_string(&mut buf)?;

    Ok(buf)
}

fn find_node<'a>(root: Node<'a, '_>, path: &str) -> Option<Node<'a, 'a>> {
    let mut node = root;
    for p in path.split('/') {
        match node.children().find(|n| n.has_tag_name(p)) {
            Some(n) => node = n,
            None => return None,
        }
    }
    Some(node)
}

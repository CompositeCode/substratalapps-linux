//! Compiles the Blueprint UI files and bundles them, the theme CSS and the
//! brand SVGs into one GResource (`/com/substratalapps/Hub/…`).

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

fn files(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut out: Vec<_> = fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|x| x == ext))
                .collect()
        })
        .unwrap_or_default();
    out.sort();
    out
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let res = manifest.join("resources");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ui_out = out.join("ui");
    fs::create_dir_all(&ui_out).unwrap();
    println!("cargo:rerun-if-changed=resources");

    // Blueprint → GtkBuilder XML.
    let blueprints = files(&res.join("ui"), "blp");
    if !blueprints.is_empty() {
        let status = Command::new("blueprint-compiler")
            .arg("batch-compile")
            .arg(&ui_out)
            .arg(res.join("ui"))
            .args(&blueprints)
            .status()
            .expect("blueprint-compiler is required (apt install blueprint-compiler)");
        assert!(status.success(), "blueprint-compiler failed");
    }

    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<gresources>\n  <gresource prefix=\"/com/substratalapps/Hub\">\n",
    );
    let mut add = |alias: String, file: PathBuf, compressed: bool| {
        let c = if compressed {
            " compressed=\"true\""
        } else {
            ""
        };
        writeln!(
            xml,
            "    <file alias=\"{alias}\"{c}>{}</file>",
            file.display()
        )
        .unwrap();
    };
    for bp in &blueprints {
        let name = bp.file_stem().unwrap().to_string_lossy().to_string();
        add(
            format!("ui/{name}.ui"),
            ui_out.join(format!("{name}.ui")),
            true,
        );
    }
    for mode in ["light", "dark"] {
        for css in files(&res.join("theme").join(mode), "css") {
            add(
                format!(
                    "theme/{mode}/{}",
                    css.file_name().unwrap().to_string_lossy()
                ),
                css,
                true,
            );
        }
    }
    for svg in files(&res.join("brand"), "svg") {
        add(
            format!("brand/{}", svg.file_name().unwrap().to_string_lossy()),
            svg,
            true,
        );
    }
    // Symbolic icons the spec names that current Adwaita no longer ships.
    // GTK adds <resource-base-path>/icons to the icon theme automatically.
    for svg in files(&res.join("icons/scalable/actions"), "svg") {
        add(
            format!(
                "icons/scalable/actions/{}",
                svg.file_name().unwrap().to_string_lossy()
            ),
            svg,
            true,
        );
    }
    add("style.css".into(), res.join("style.css"), true);
    xml.push_str("  </gresource>\n</gresources>\n");
    let xml_path = out.join("hub.gresource.xml");
    fs::write(&xml_path, xml).unwrap();
    glib_build_tools::compile_resources(
        &[out.as_path()],
        xml_path.to_str().unwrap(),
        "hub.gresource",
    );
}

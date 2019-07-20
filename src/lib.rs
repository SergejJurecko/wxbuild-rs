use cc::Build;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn is_modified(static_lib_path: &PathBuf, folder: &str) -> std::io::Result<bool> {
    let lib_meta = fs::metadata(static_lib_path)?;
    let lib_age = lib_meta.modified()?;
    if let Ok(lib_age) = lib_age.elapsed() {
        let dir_iter = fs::read_dir(folder)?;
        for entry in dir_iter {
            if let Ok(entry) = entry {
                if let Ok(md) = entry.metadata() {
                    if let Ok(file_age) = md.modified() {
                        if let Ok(file_age) = file_age.elapsed() {
                            if file_age < lib_age {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
    } else {
        return Ok(true);
    }
    Ok(false)
}

/// Build all cpp files in specified folder and correctly link with wxWidgets.
///
/// If add_start is true:
///
/// - It will create a cpp file containing wxIMPLEMENT_APP_NO_MAIN(appname)
/// - It will create wxffi.rs file you should include with: include!(concat!(env!("OUT_DIR"), "/wxffi.rs"))
/// - wxffi.rs will contain function start() that will run your wx gui. This function will not return while GUI is active.
/// - appname.h (all lowercase) must exist and have appname class declared.
pub fn build(folder: &str, add_start: bool, appname: &str) -> std::io::Result<()> {
    let target = env::var("TARGET").unwrap();
    let out_dir_s = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_s);
    let static_lib_path = out_dir.join("libwxrs.a");
    let wxcfg = env::var("WX_CONFIG").unwrap_or("wx-config".to_owned());
    let wxdir = env::var("WX_DIR").unwrap_or("".to_owned());

    if is_modified(&static_lib_path, folder).unwrap_or(true) {
        let mut cc = Build::new();
        for entry in fs::read_dir(folder)? {
            let entry = entry?;
            let path = entry.path();
            let extension = path.extension().and_then(OsStr::to_str).unwrap_or("");
            if extension == "cpp" {
                cc.file(&path);
            }
        }
        if let Ok(cxx) = Command::new(wxcfg.as_str()).args(&["--cxxflags"]).output() {
            let cxx = std::str::from_utf8(cxx.stdout.as_ref()).unwrap();
            for word in cxx.split_whitespace() {
                cc.flag(word);
            }
        } else if wxdir.len() > 0 && target.contains("msvc") {
            cc.define("__WXMSW__", "1");
            cc.define("_UNICODE", "1");
            cc.include(Path::new(&wxdir).join("include"));
            cc.include(Path::new(&wxdir).join("include").join("msvc"));
        // cc.include(Path::new(&wxdir).join("lib").join("vc_x64_lib").join("mswu"));
        } else {
            panic!("No WX_CONFIG or WX_DIR set");
        }
        cc.cpp(true);
        if target.contains("darwin") {
            cc.flag("-mmacosx-version-min=10.12");
            cc.flag("-std=c++11");
        } else if target.contains("msvc") {
            cc.flag("/EHsc");
        }
        if add_start {
            cc.include(folder);
            let start = out_dir.join("start.cpp");
            let mut file = fs::File::create(start.clone()).unwrap();
            use std::io::Write;
            let cpp = format!("#include <wx/wx.h>\n
                #include \"{}.h\"
                wxIMPLEMENT_APP_NO_MAIN({});
                void *g_rsdata = NULL;
                extern \"C\" {{ void wx_start(void* userdata) {{ char **argv = nullptr; int argc = 0; g_rsdata = userdata; wxEntry(argc, argv); }} }}", appname.to_ascii_lowercase(), appname);
            file.write(cpp.as_bytes()).unwrap();
            cc.file(start);

            let mut file = fs::File::create(&out_dir.join("wxffi.rs")).unwrap();
            file.write(
                br#"
                fn start("#,
            )
            .unwrap();
            file.write(format!("userdata: &mut {}", appname).as_bytes())
                .unwrap();
            file.write(
                br#") {
                    unsafe {
                        wx_start(userdata as *mut _ as _);
                    }
                }
                extern "C" {
                    fn wx_start( "#,
            )
            .unwrap();
            file.write(format!("userdata: *mut std::os::raw::c_void").as_bytes())
                .unwrap();
            file.write(
                br#" );
                }
            "#,
            )
            .unwrap();
            // #[link_name = "\u{1}_wx_start"]
        }

        cc.extra_warnings(false);
        cc.compile("libwxrs.a");
    }

    println!("cargo:rustc-link-search=native={}", out_dir_s);
    println!("cargo:rustc-link-lib=wxrs");

    if wxdir.len() > 0 && target.contains("msvc") {
        println!("cargo:rustc-link-search=native={}\\lib\\vc_x64_lib", wxdir);
        // println!(
        //     "cargo:rustc-link-lib=static={}",
        //     part.trim_start_matches("-l")
        // );
        let dir_iter = fs::read_dir(Path::new(&wxdir).join("lib").join("vc_x64_lib"))?;
        for entry in dir_iter {
            if let Ok(entry) = entry {
                let path = entry.path();
                let extension = path.extension().and_then(OsStr::to_str).unwrap_or("");
                let file_stem = path.file_stem().and_then(OsStr::to_str).unwrap_or("");
                if extension == "lib" {
                    // if cfg!(debug_assertions) {
                    //     if file_stem.starts_with("wxbase31ud") {
                    //         println!("cargo:rustc-link-lib=static={}", file_stem);
                    //     } else if file_stem.starts_with("wxbase31u") {
                    //         continue;
                    //     } else if file_stem.ends_with("d") {
                    //         println!("cargo:rustc-link-lib=static={}", file_stem);
                    //     }
                    // } else {

                    if file_stem.starts_with("wxmsw31ud") {
                        continue;
                    } else if file_stem.starts_with("wxbase31ud") {
                        continue;
                    } else if file_stem.starts_with("wxmsw31u") {
                        println!("cargo:rustc-link-lib=static={}", file_stem);
                    } else if file_stem.starts_with("wxbase31u") {
                        println!("cargo:rustc-link-lib=static={}", file_stem);
                    } else if !file_stem.ends_with("d") {
                        println!("cargo:rustc-link-lib=static={}", file_stem);
                    }
                    // }
                }
            }
        }
        return Ok(());
    }

    let libs = Command::new(wxcfg.as_str())
        .args(&["--libs"])
        .output()
        .expect("failed to execute wx-config");
    let libs = std::str::from_utf8(libs.stdout.as_ref()).unwrap();
    let mut framework: bool = false;
    for part in libs.split_whitespace() {
        if part.starts_with("-L") {
            println!(
                "cargo:rustc-link-search=native={}",
                part.trim_start_matches("-L")
            );
        } else if part.starts_with("-l") {
            let static_pth = format!("/usr/local/lib/lib{}.a", part.trim_start_matches("-l"));
            if fs::metadata(static_pth).is_ok() {
                println!("cargo:rustc-link-search=native=/usr/local/lib/");
                println!(
                    "cargo:rustc-link-lib=static={}",
                    part.trim_start_matches("-l")
                );
            } else {
                println!("cargo:rustc-link-lib={}", part.trim_start_matches("-l"));
            }
        } else if part == "-framework" {
            framework = true;
        } else {
            if framework {
                println!("cargo:rustc-link-lib=framework={}", part);
                framework = false;
            } else if part.ends_with(".a") {
                let path = PathBuf::from(part.to_string());
                println!(
                    "cargo:rustc-link-search=native={}",
                    path.parent().unwrap().to_str().unwrap()
                );
                println!(
                    "cargo:rustc-link-lib={}",
                    path.file_stem()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .trim_start_matches("lib")
                );
            }
        }
    }
    println!("cargo:rustc-link-lib=c++");

    Ok(())
}

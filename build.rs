fn main() {
    println!("cargo:rerun-if-changed=Compositor/Rendering/AdjustPixels.c");
    println!("cargo:rerun-if-changed=Compositor/Rendering/AdjustPixels.h");
    println!("cargo:rerun-if-changed=Compositor/Rendering/BrushPixels.c");
    println!("cargo:rerun-if-changed=Compositor/Rendering/BrushPixels.h");
    println!("cargo:rerun-if-changed=Compositor/Rendering/ContentFill.c");
    println!("cargo:rerun-if-changed=Compositor/Rendering/ContentFill.h");
    println!("cargo:rerun-if-changed=Compositor/Rendering/DitherPixels.c");
    println!("cargo:rerun-if-changed=Compositor/Rendering/DitherPixels.h");
    println!("cargo:rerun-if-changed=Compositor/Rendering/HealPixels.c");
    println!("cargo:rerun-if-changed=Compositor/Rendering/HealPixels.h");
    println!("cargo:rerun-if-changed=Compositor/Rendering/LensPixels.c");
    println!("cargo:rerun-if-changed=Compositor/Rendering/LensPixels.h");
    println!("cargo:rerun-if-changed=Compositor/Rendering/LevelsPixels.c");
    println!("cargo:rerun-if-changed=Compositor/Rendering/LevelsPixels.h");
    println!("cargo:rerun-if-changed=Compositor/Rendering/NoisePixels.c");
    println!("cargo:rerun-if-changed=Compositor/Rendering/NoisePixels.h");
    println!("cargo:rerun-if-changed=Compositor/Rendering/WandPixels.c");
    println!("cargo:rerun-if-changed=Compositor/Rendering/WandPixels.h");

    let mut build = cc::Build::new();
    build
        .include("Compositor/Rendering")
        .file("Compositor/Rendering/AdjustPixels.c")
        .file("Compositor/Rendering/BrushPixels.c")
        .file("Compositor/Rendering/ContentFill.c")
        .file("Compositor/Rendering/DitherPixels.c")
        .file("Compositor/Rendering/HealPixels.c")
        .file("Compositor/Rendering/LensPixels.c")
        .file("Compositor/Rendering/LevelsPixels.c")
        .file("Compositor/Rendering/NoisePixels.c")
        .file("Compositor/Rendering/WandPixels.c")
        .define("_DEFAULT_SOURCE", None)
        .define("_GNU_SOURCE", None)
        .define("_USE_MATH_DEFINES", None)
        .flag_if_supported("-std=gnu99")
        .flag_if_supported("-Wno-unused-parameter")
        .flag_if_supported("-Wno-misleading-indentation")
        .flag_if_supported("-fopenmp");

    build.compile("ckernels");

    println!("cargo:rustc-link-lib=m");
    println!("cargo:rustc-link-lib=gomp");
}

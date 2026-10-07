fn main() {
    println!("cargo:rerun-if-changed=c_kernels/AdjustPixels.c");
    println!("cargo:rerun-if-changed=c_kernels/AdjustPixels.h");
    println!("cargo:rerun-if-changed=c_kernels/BrushPixels.c");
    println!("cargo:rerun-if-changed=c_kernels/BrushPixels.h");
    println!("cargo:rerun-if-changed=c_kernels/ContentFill.c");
    println!("cargo:rerun-if-changed=c_kernels/ContentFill.h");
    println!("cargo:rerun-if-changed=c_kernels/DitherPixels.c");
    println!("cargo:rerun-if-changed=c_kernels/DitherPixels.h");
    println!("cargo:rerun-if-changed=c_kernels/HealPixels.c");
    println!("cargo:rerun-if-changed=c_kernels/HealPixels.h");
    println!("cargo:rerun-if-changed=c_kernels/LensPixels.c");
    println!("cargo:rerun-if-changed=c_kernels/LensPixels.h");
    println!("cargo:rerun-if-changed=c_kernels/LevelsPixels.c");
    println!("cargo:rerun-if-changed=c_kernels/LevelsPixels.h");
    println!("cargo:rerun-if-changed=c_kernels/NoisePixels.c");
    println!("cargo:rerun-if-changed=c_kernels/NoisePixels.h");
    println!("cargo:rerun-if-changed=c_kernels/WandPixels.c");
    println!("cargo:rerun-if-changed=c_kernels/WandPixels.h");

    let mut build = cc::Build::new();
    build
        .include("c_kernels")
        .file("c_kernels/AdjustPixels.c")
        .file("c_kernels/BrushPixels.c")
        .file("c_kernels/ContentFill.c")
        .file("c_kernels/DitherPixels.c")
        .file("c_kernels/HealPixels.c")
        .file("c_kernels/LensPixels.c")
        .file("c_kernels/LevelsPixels.c")
        .file("c_kernels/NoisePixels.c")
        .file("c_kernels/WandPixels.c")
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

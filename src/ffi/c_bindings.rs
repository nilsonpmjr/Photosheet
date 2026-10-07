use libc::{c_double, c_float, c_int, size_t};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DitherParams {
    pub style: c_int,
    pub levels: c_int,
    pub diffusion: c_float,
    pub density: c_float,
    pub contrast: c_float,
    pub cell: c_int,
    pub angle: c_float,
    pub light_on_dark: c_int,
    pub original_colors: c_int,
    pub dark: [u8; 3],
    pub light: [u8; 3],
    pub glyph_width: c_int,
    pub glyph_height: c_int,
    pub glyphs: *const u8,
    pub glyph_coverage: *const c_float,
    pub glyph_count: c_int,
    pub dots: c_float,
    pub wobble: c_float,
}

extern "C" {
    // DitherPixels
    pub fn dither_apply(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        params: *const DitherParams,
    ) -> c_int;

    pub fn dither_dots(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        block: c_int,
        gap: *const u8,
    );

    pub fn dither_glow(
        rgba: *mut u8,
        glow: *const u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        amount: c_float,
    );

    // HealPixels
    pub fn heal_coverage_bounds(
        gray: *const u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        bounds: *mut i64,
    );

    pub fn spot_heal(
        rgba: *mut u8,
        coverage: *const u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        opacity: c_float,
        mode: c_int,
        seed: u32,
    ) -> c_int;

    // LensPixels
    pub fn lens_distort(
        source: *const u8,
        destination: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        k: c_double,
    );

    // LevelsPixels
    pub fn levels_apply(pixels: *mut u8, count: size_t, tables: *const c_float);

    pub fn levels_histogram(
        pixels: *const u8,
        coverage: *const u8,
        count: size_t,
        bins: *mut c_double,
    );

    pub fn cube_apply(pixels: *mut u8, count: size_t, cube: *const c_float, dimension: c_int);

    // NoisePixels
    pub fn noise_add(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        amount: c_float,
        gaussian: c_int,
        monochromatic: c_int,
        seed: u32,
    );

    pub fn noise_add_at(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        amount: c_float,
        gaussian: c_int,
        monochromatic: c_int,
        seed: u32,
        origin_x: i64,
        origin_y: i64,
    );

    // WandPixels
    pub fn wand_mask(
        rgba: *const u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        seed_x: size_t,
        seed_y: size_t,
        radius: size_t,
        tolerance: c_int,
        contiguous: c_int,
        mask: *mut u8,
    ) -> i64;

    pub fn color_range_mask(
        rgba: *const u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        include: *const u8,
        include_count: c_int,
        exclude: *const u8,
        exclude_count: c_int,
        fuzziness: c_int,
        invert: c_int,
        mask: *mut u8,
    ) -> i64;

    pub fn wand_trace(
        mask: *const u8,
        width: size_t,
        height: size_t,
        points: *mut *mut i32,
        point_count: *mut size_t,
        loops: *mut *mut i32,
        loop_count: *mut size_t,
    ) -> c_int;

    // BrushPixels
    pub fn brush_alpha_bounds(
        bytes: *const u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        bounds: *mut size_t,
    );

    pub fn layer_extract_alpha(
        rgba: *const u8,
        rgba_stride: size_t,
        gray: *mut u8,
        gray_stride: size_t,
        width: size_t,
        height: size_t,
    );

    pub fn layer_unpremultiply_opaque(rgba: *mut u8, stride: size_t, width: size_t, height: size_t);

    pub fn layer_restore_alpha(
        rgba: *mut u8,
        stride: size_t,
        alpha: *const u8,
        alpha_stride: size_t,
        width: size_t,
        height: size_t,
    );

    // ContentFill
    pub fn content_fill(
        rgba: *mut u8,
        stride: size_t,
        mask: *const u8,
        mask_stride: size_t,
        width: c_int,
        height: c_int,
    ) -> c_int;

    // AdjustPixels
    pub fn adjust_color_balance(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        shadows: *const c_float,
        midtones: *const c_float,
        highlights: *const c_float,
        preserve_luminosity: c_int,
    );

    pub fn rgba_clamp_premultiplied(rgba: *mut u8, count: size_t);

    pub fn adjust_camera_raw(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        red_gain: c_double,
        green_gain: c_double,
        blue_gain: c_double,
        exposure: c_double,
        contrast: c_double,
        highlights: c_double,
        shadows: c_double,
        whites: c_double,
        blacks: c_double,
        vibrance: c_double,
        saturation: c_double,
        clipping: c_int,
    );

    pub fn adjust_camera_raw_clip_overlay(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        shadows: c_int,
        highlights: c_int,
    );

    pub fn adjust_camera_raw_curve_color(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        tone_lut: *const c_float,
        red_lut: *const c_float,
        green_lut: *const c_float,
        blue_lut: *const c_float,
        refine_saturation: c_double,
        mixer: *const c_float,
        point_count: c_int,
        points: *const c_float,
        grade: *const c_float,
        blending: c_double,
        balance: c_double,
        visualize: c_int,
    );

    pub fn adjust_camera_raw_effects(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        texture: c_double,
        clarity: c_double,
        dehaze: c_double,
        glow: c_double,
        glow_style: c_int,
        glow_range: c_double,
        glow_spread: c_double,
        glow_warmth: c_double,
        vignette_amount: c_double,
        vignette_midpoint: c_double,
        vignette_roundness: c_double,
        vignette_feather: c_double,
        vignette_highlights: c_double,
        vignette_style: c_int,
        scale: c_double,
    );

    pub fn adjust_colored_vignette(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        frame_x: c_double,
        frame_y: c_double,
        frame_width: c_double,
        frame_height: c_double,
        fills_clear: c_int,
        amount: c_double,
        midpoint: c_double,
        roundness: c_double,
        feather: c_double,
        highlights: c_double,
        red: c_double,
        green: c_double,
        blue: c_double,
    );

    pub fn adjust_tonal_contrast(
        rgba: *mut u8,
        blurred: *const u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        blurred_stride: size_t,
        amount: c_double,
        shadows: c_double,
        midtones: c_double,
        highlights: c_double,
    );

    pub fn adjust_camera_raw_detail(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        sharpen_amount: c_double,
        sharpen_radius: c_double,
        sharpen_detail: c_double,
        sharpen_masking: c_double,
        noise_luminance: c_double,
        noise_luminance_detail: c_double,
        noise_luminance_contrast: c_double,
        noise_color: c_double,
        noise_color_detail: c_double,
        noise_color_smoothness: c_double,
        scale: c_double,
    );

    pub fn adjust_camera_raw_sharpen_mask_overlay(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        sharpen_radius: c_double,
        sharpen_detail: c_double,
        sharpen_masking: c_double,
        scale: c_double,
    );

    pub fn adjust_camera_raw_optics(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        remove_chromatic: c_int,
        lens_profile: c_int,
        profile_distortion: c_double,
        profile_vignetting: c_double,
        distortion_k: c_double,
        purple_amount: c_double,
        purple_hue_low: c_double,
        purple_hue_high: c_double,
        green_amount: c_double,
        green_hue_low: c_double,
        green_hue_high: c_double,
        vignette_amount: c_double,
        vignette_midpoint: c_double,
        scale: c_double,
    );

    pub fn adjust_camera_raw_calibration(
        rgba: *mut u8,
        width: size_t,
        height: size_t,
        stride: size_t,
        shadow_tint: c_double,
        red_hue: c_double,
        red_saturation: c_double,
        green_hue: c_double,
        green_saturation: c_double,
        blue_hue: c_double,
        blue_saturation: c_double,
        process_version: c_int,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wand_mask_basic() {
        let width = 4;
        let height = 4;
        let stride = width * 4;
        // 4x4 image, all red with full alpha
        let mut image = vec![0u8; width * height * 4];
        for i in (0..image.len()).step_by(4) {
            image[i] = 255; // R
            image[i + 1] = 0; // G
            image[i + 2] = 0; // B
            image[i + 3] = 255; // A
        }

        let mut mask = vec![0u8; width * height];
        let matched = unsafe {
            wand_mask(
                image.as_ptr(),
                width,
                height,
                stride,
                0,  // seed_x
                0,  // seed_y
                0,  // radius
                10, // tolerance
                1,  // contiguous
                mask.as_mut_ptr(),
            )
        };

        assert_eq!(matched, 16);
        assert!(mask.iter().all(|&val| val == 255));
    }

    #[test]
    fn test_noise_add() {
        let width = 8;
        let height = 8;
        let stride = width * 4;
        let mut image = vec![128u8; width * height * 4];
        unsafe {
            noise_add(image.as_mut_ptr(), width, height, stride, 20.0, 0, 1, 1337);
        }
        // Alpha must remain intact (128)
        for i in (3..image.len()).step_by(4) {
            assert_eq!(image[i], 128);
        }
    }

    #[test]
    fn test_lens_distort_identity() {
        let width = 4;
        let height = 4;
        let stride = width * 4;
        let mut source = vec![0u8; width * height * 4];
        for i in 0..source.len() {
            source[i] = (i % 256) as u8;
        }
        let mut dest = vec![0u8; width * height * 4];

        unsafe {
            lens_distort(
                source.as_ptr(),
                dest.as_mut_ptr(),
                width,
                height,
                stride,
                0.0, // k = 0 copies source exactly
            );
        }

        assert_eq!(source, dest);
    }

    #[test]
    fn test_dither_apply_basic() {
        let width = 4;
        let height = 4;
        let stride = width * 4;
        let mut image = vec![128u8; width * height * 4];
        let params = DitherParams {
            style: 0, // DITHER_ATKINSON
            levels: 2,
            diffusion: 1.0,
            density: 0.0,
            contrast: 0.0,
            cell: 2,
            angle: 0.0,
            light_on_dark: 0,
            original_colors: 0,
            dark: [0, 0, 0],
            light: [255, 255, 255],
            glyph_width: 0,
            glyph_height: 0,
            glyphs: std::ptr::null(),
            glyph_coverage: std::ptr::null(),
            glyph_count: 0,
            dots: 0.0,
            wobble: 0.0,
        };

        let res = unsafe { dither_apply(image.as_mut_ptr(), width, height, stride, &params) };
        assert_eq!(res, 1);
    }
}

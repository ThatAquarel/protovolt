use u8g2_fonts::{FontRenderer, fonts};

pub struct Fonts {
    pub icons_1x: FontRenderer,
    pub icons_2x: FontRenderer,
    pub info_small: FontRenderer,
    pub info_navbar: FontRenderer,
    pub info_large: FontRenderer,
    pub readout_small: FontRenderer,
    pub readout_large: FontRenderer,
}

impl Default for Fonts {
    fn default() -> Self {
        Self {
            icons_1x: FontRenderer::new::<fonts::u8g2_font_open_iconic_arrow_1x_t>(),
            icons_2x: FontRenderer::new::<fonts::u8g2_font_open_iconic_all_2x_t>(),
            info_small: FontRenderer::new::<fonts::u8g2_font_helvB08_tf>(),
            info_navbar: FontRenderer::new::<fonts::u8g2_font_profont11_tr>(),
            info_large: FontRenderer::new::<fonts::u8g2_font_helvR14_tr>(),
            readout_small: FontRenderer::new::<fonts::u8g2_font_logisoso16_tr>(),
            readout_large: FontRenderer::new::<fonts::u8g2_font_logisoso32_tn>(),
        }
    }
}

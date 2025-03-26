use std::sync::LazyLock;
use std::{iter, slice};

use casefold::ascii::CaseFoldMap;

use super::rgb::RgbColor;

pub type NamedColorIter = iter::Copied<slice::Iter<'static, (&'static str, RgbColor)>>;

impl RgbColor {
    /// Finds a color by its name in the standard list of [148 CSS colors]. Case-insensitive.
    ///
    /// [148 CSS colors]: https://www.w3.org/wiki/CSS/Properties/color/keywords
    pub fn named(name: &str) -> Option<RgbColor> {
        static LOOKUP: LazyLock<CaseFoldMap<&str, RgbColor>> = LazyLock::new(|| {
            NAMED_COLORS
                .iter()
                .map(|&(key, val)| (key.into(), val))
                .collect()
        });

        if name.starts_with('#') {
            return name.parse().ok();
        }
        LOOKUP.get(name).copied()
    }

    /// Iterates through colors in the standard list of [148 CSS colors].
    ///
    /// [148 CSS colors]: https://www.w3.org/wiki/CSS/Properties/color/keywords
    pub fn iter_named() -> NamedColorIter {
        NAMED_COLORS.iter().copied()
    }
}

const NAMED_COLORS: &[(&str, RgbColor)] = &[
    ("aliceblue", RgbColor::hex(0xF0F8FF)),
    ("antiquewhite", RgbColor::hex(0xFAEBD7)),
    ("aqua", RgbColor::hex(0x00FFFF)),
    ("aquamarine", RgbColor::hex(0x7FFFD4)),
    ("azure", RgbColor::hex(0xF0FFFF)),
    ("beige", RgbColor::hex(0xF5F5DC)),
    ("bisque", RgbColor::hex(0xFFE4C4)),
    ("black", RgbColor::hex(0x000000)),
    ("blanchedalmond", RgbColor::hex(0xFFEBCD)),
    ("blue", RgbColor::hex(0x0000FF)),
    ("blueviolet", RgbColor::hex(0x8A2BE2)),
    ("brown", RgbColor::hex(0xA52A2A)),
    ("burlywood", RgbColor::hex(0xDEB887)),
    ("cadetblue", RgbColor::hex(0x5F9EA0)),
    ("chartreuse", RgbColor::hex(0x7FFF00)),
    ("chocolate", RgbColor::hex(0xD2691E)),
    ("coral", RgbColor::hex(0xFF7F50)),
    ("cornflowerblue", RgbColor::hex(0x6495ED)),
    ("cornsilk", RgbColor::hex(0xFFF8DC)),
    ("crimson", RgbColor::hex(0xDC143C)),
    ("cyan", RgbColor::hex(0x00FFFF)),
    ("darkblue", RgbColor::hex(0x00008B)),
    ("darkcyan", RgbColor::hex(0x008B8B)),
    ("darkgoldenrod", RgbColor::hex(0xB8860B)),
    ("darkgray", RgbColor::hex(0xA9A9A9)),
    ("darkgrey", RgbColor::hex(0xA9A9A9)),
    ("darkgreen", RgbColor::hex(0x006400)),
    ("darkkhaki", RgbColor::hex(0xBDB76B)),
    ("darkmagenta", RgbColor::hex(0x8B008B)),
    ("darkolivegreen", RgbColor::hex(0x556B2F)),
    ("darkorange", RgbColor::hex(0xFF8C00)),
    ("darkorchid", RgbColor::hex(0x9932CC)),
    ("darkred", RgbColor::hex(0x8B0000)),
    ("darksalmon", RgbColor::hex(0xE9967A)),
    ("darkseagreen", RgbColor::hex(0x8FBC8F)),
    ("darkslateblue", RgbColor::hex(0x483D8B)),
    ("darkslategray", RgbColor::hex(0x2F4F4F)),
    ("darkslategrey", RgbColor::hex(0x2F4F4F)),
    ("darkturquoise", RgbColor::hex(0x00CED1)),
    ("darkviolet", RgbColor::hex(0x9400D3)),
    ("deeppink", RgbColor::hex(0xFF1493)),
    ("deepskyblue", RgbColor::hex(0x00BFFF)),
    ("dimgray", RgbColor::hex(0x696969)),
    ("dimgrey", RgbColor::hex(0x696969)),
    ("dodgerblue", RgbColor::hex(0x1E90FF)),
    ("firebrick", RgbColor::hex(0xB22222)),
    ("floralwhite", RgbColor::hex(0xFFFAF0)),
    ("forestgreen", RgbColor::hex(0x228B22)),
    ("fuchsia", RgbColor::hex(0xFF00FF)),
    ("gainsboro", RgbColor::hex(0xDCDCDC)),
    ("ghostwhite", RgbColor::hex(0xF8F8FF)),
    ("gold", RgbColor::hex(0xFFD700)),
    ("goldenrod", RgbColor::hex(0xDAA520)),
    ("gray", RgbColor::hex(0x808080)),
    ("grey", RgbColor::hex(0x808080)),
    ("green", RgbColor::hex(0x008000)),
    ("greenyellow", RgbColor::hex(0xADFF2F)),
    ("honeydew", RgbColor::hex(0xF0FFF0)),
    ("hotpink", RgbColor::hex(0xFF69B4)),
    ("indianred", RgbColor::hex(0xCD5C5C)),
    ("indigo", RgbColor::hex(0x4B0082)),
    ("ivory", RgbColor::hex(0xFFFFF0)),
    ("khaki", RgbColor::hex(0xF0E68C)),
    ("lavender", RgbColor::hex(0xE6E6FA)),
    ("lavenderblush", RgbColor::hex(0xFFF0F5)),
    ("lawngreen", RgbColor::hex(0x7CFC00)),
    ("lemonchiffon", RgbColor::hex(0xFFFACD)),
    ("lightblue", RgbColor::hex(0xADD8E6)),
    ("lightcoral", RgbColor::hex(0xF08080)),
    ("lightcyan", RgbColor::hex(0xE0FFFF)),
    ("lightgoldenrodyellow", RgbColor::hex(0xFAFAD2)),
    ("lightgreen", RgbColor::hex(0x90EE90)),
    ("lightgrey", RgbColor::hex(0xD3D3D3)),
    ("lightgray", RgbColor::hex(0xD3D3D3)),
    ("lightpink", RgbColor::hex(0xFFB6C1)),
    ("lightsalmon", RgbColor::hex(0xFFA07A)),
    ("lightseagreen", RgbColor::hex(0x20B2AA)),
    ("lightskyblue", RgbColor::hex(0x87CEFA)),
    ("lightslategray", RgbColor::hex(0x778899)),
    ("lightslategrey", RgbColor::hex(0x778899)),
    ("lightsteelblue", RgbColor::hex(0xB0C4DE)),
    ("lightyellow", RgbColor::hex(0xFFFFE0)),
    ("lime", RgbColor::hex(0x00FF00)),
    ("limegreen", RgbColor::hex(0x32CD32)),
    ("linen", RgbColor::hex(0xFAF0E6)),
    ("magenta", RgbColor::hex(0xFF00FF)),
    ("maroon", RgbColor::hex(0x800000)),
    ("mediumaquamarine", RgbColor::hex(0x66CDAA)),
    ("mediumblue", RgbColor::hex(0x0000CD)),
    ("mediumorchid", RgbColor::hex(0xBA55D3)),
    ("mediumpurple", RgbColor::hex(0x9370DB)),
    ("mediumseagreen", RgbColor::hex(0x3CB371)),
    ("mediumslateblue", RgbColor::hex(0x7B68EE)),
    ("mediumspringgreen", RgbColor::hex(0x00FA9A)),
    ("mediumturquoise", RgbColor::hex(0x48D1CC)),
    ("mediumvioletred", RgbColor::hex(0xC71585)),
    ("midnightblue", RgbColor::hex(0x191970)),
    ("mintcream", RgbColor::hex(0xF5FFFA)),
    ("mistyrose", RgbColor::hex(0xFFE4E1)),
    ("moccasin", RgbColor::hex(0xFFE4B5)),
    ("navajowhite", RgbColor::hex(0xFFDEAD)),
    ("navy", RgbColor::hex(0x000080)),
    ("oldlace", RgbColor::hex(0xFDF5E6)),
    ("olive", RgbColor::hex(0x808000)),
    ("olivedrab", RgbColor::hex(0x6B8E23)),
    ("orange", RgbColor::hex(0xFFA500)),
    ("orangered", RgbColor::hex(0xFF4500)),
    ("orchid", RgbColor::hex(0xDA70D6)),
    ("palegoldenrod", RgbColor::hex(0xEEE8AA)),
    ("palegreen", RgbColor::hex(0x98FB98)),
    ("paleturquoise", RgbColor::hex(0xAFEEEE)),
    ("palevioletred", RgbColor::hex(0xDB7093)),
    ("papayawhip", RgbColor::hex(0xFFEFD5)),
    ("peachpuff", RgbColor::hex(0xFFDAB9)),
    ("peru", RgbColor::hex(0xCD853F)),
    ("pink", RgbColor::hex(0xFFC0CB)),
    ("plum", RgbColor::hex(0xDDA0DD)),
    ("powderblue", RgbColor::hex(0xB0E0E6)),
    ("purple", RgbColor::hex(0x800080)),
    ("rebeccapurple", RgbColor::hex(0x663399)),
    ("red", RgbColor::hex(0xFF0000)),
    ("rosybrown", RgbColor::hex(0xBC8F8F)),
    ("royalblue", RgbColor::hex(0x4169E1)),
    ("saddlebrown", RgbColor::hex(0x8B4513)),
    ("salmon", RgbColor::hex(0xFA8072)),
    ("sandybrown", RgbColor::hex(0xF4A460)),
    ("seagreen", RgbColor::hex(0x2E8B57)),
    ("seashell", RgbColor::hex(0xFFF5EE)),
    ("sienna", RgbColor::hex(0xA0522D)),
    ("silver", RgbColor::hex(0xC0C0C0)),
    ("skyblue", RgbColor::hex(0x87CEEB)),
    ("slateblue", RgbColor::hex(0x6A5ACD)),
    ("slategray", RgbColor::hex(0x708090)),
    ("slategrey", RgbColor::hex(0x708090)),
    ("snow", RgbColor::hex(0xFFFAFA)),
    ("springgreen", RgbColor::hex(0x00FF7F)),
    ("steelblue", RgbColor::hex(0x4682B4)),
    ("tan", RgbColor::hex(0xD2B48C)),
    ("teal", RgbColor::hex(0x008080)),
    ("thistle", RgbColor::hex(0xD8BFD8)),
    ("tomato", RgbColor::hex(0xFF6347)),
    ("turquoise", RgbColor::hex(0x40E0D0)),
    ("violet", RgbColor::hex(0xEE82EE)),
    ("wheat", RgbColor::hex(0xF5DEB3)),
    ("white", RgbColor::hex(0xFFFFFF)),
    ("whitesmoke", RgbColor::hex(0xF5F5F5)),
    ("yellow", RgbColor::hex(0xFFFF00)),
    ("yellowgreen", RgbColor::hex(0x9ACD32)),
];

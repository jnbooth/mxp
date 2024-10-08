use std::collections::hash_map;
use std::iter;

use casefold::ascii::CaseFold;

use crate::lookup::Lookup;

use super::rgb::RgbColor;

pub type NamedColorIter = iter::Map<
    hash_map::Iter<'static, CaseFold<&'static str>, RgbColor>,
    fn((&CaseFold<&'static str>, &'static RgbColor)) -> (&'static str, RgbColor),
>;

impl RgbColor {
    /// Finds a color by its name in the standard list of [148 CSS colors]. Case-insensitive.
    ///
    /// [148 CSS colors]: https://www.w3.org/wiki/CSS/Properties/color/keywords
    pub fn named(name: &str) -> Option<RgbColor> {
        if name.starts_with('#') {
            name.parse().ok()
        } else {
            NAMED_COLORS.get(name).copied()
        }
    }

    /// Iterates through colors in the standard list of [148 CSS colors].
    ///
    /// [148 CSS colors]: https://www.w3.org/wiki/CSS/Properties/color/keywords
    pub fn iter_named() -> NamedColorIter {
        NAMED_COLORS.entries().map(|(k, v)| (k.unfold(), *v))
    }
}

static NAMED_COLORS: Lookup<RgbColor> = Lookup::new(|| {
    let hex = RgbColor::hex;
    vec![
        ("aliceblue", hex(0xF0F8FF)),
        ("antiquewhite", hex(0xFAEBD7)),
        ("aqua", hex(0x00FFFF)),
        ("aquamarine", hex(0x7FFFD4)),
        ("azure", hex(0xF0FFFF)),
        ("beige", hex(0xF5F5DC)),
        ("bisque", hex(0xFFE4C4)),
        ("black", hex(0x000000)),
        ("blanchedalmond", hex(0xFFEBCD)),
        ("blue", hex(0x0000FF)),
        ("blueviolet", hex(0x8A2BE2)),
        ("brown", hex(0xA52A2A)),
        ("burlywood", hex(0xDEB887)),
        ("cadetblue", hex(0x5F9EA0)),
        ("chartreuse", hex(0x7FFF00)),
        ("chocolate", hex(0xD2691E)),
        ("coral", hex(0xFF7F50)),
        ("cornflowerblue", hex(0x6495ED)),
        ("cornsilk", hex(0xFFF8DC)),
        ("crimson", hex(0xDC143C)),
        ("cyan", hex(0x00FFFF)),
        ("darkblue", hex(0x00008B)),
        ("darkcyan", hex(0x008B8B)),
        ("darkgoldenrod", hex(0xB8860B)),
        ("darkgray", hex(0xA9A9A9)),
        ("darkgrey", hex(0xA9A9A9)),
        ("darkgreen", hex(0x006400)),
        ("darkkhaki", hex(0xBDB76B)),
        ("darkmagenta", hex(0x8B008B)),
        ("darkolivegreen", hex(0x556B2F)),
        ("darkorange", hex(0xFF8C00)),
        ("darkorchid", hex(0x9932CC)),
        ("darkred", hex(0x8B0000)),
        ("darksalmon", hex(0xE9967A)),
        ("darkseagreen", hex(0x8FBC8F)),
        ("darkslateblue", hex(0x483D8B)),
        ("darkslategray", hex(0x2F4F4F)),
        ("darkslategrey", hex(0x2F4F4F)),
        ("darkturquoise", hex(0x00CED1)),
        ("darkviolet", hex(0x9400D3)),
        ("deeppink", hex(0xFF1493)),
        ("deepskyblue", hex(0x00BFFF)),
        ("dimgray", hex(0x696969)),
        ("dimgrey", hex(0x696969)),
        ("dodgerblue", hex(0x1E90FF)),
        ("firebrick", hex(0xB22222)),
        ("floralwhite", hex(0xFFFAF0)),
        ("forestgreen", hex(0x228B22)),
        ("fuchsia", hex(0xFF00FF)),
        ("gainsboro", hex(0xDCDCDC)),
        ("ghostwhite", hex(0xF8F8FF)),
        ("gold", hex(0xFFD700)),
        ("goldenrod", hex(0xDAA520)),
        ("gray", hex(0x808080)),
        ("grey", hex(0x808080)),
        ("green", hex(0x008000)),
        ("greenyellow", hex(0xADFF2F)),
        ("honeydew", hex(0xF0FFF0)),
        ("hotpink", hex(0xFF69B4)),
        ("indianred", hex(0xCD5C5C)),
        ("indigo", hex(0x4B0082)),
        ("ivory", hex(0xFFFFF0)),
        ("khaki", hex(0xF0E68C)),
        ("lavender", hex(0xE6E6FA)),
        ("lavenderblush", hex(0xFFF0F5)),
        ("lawngreen", hex(0x7CFC00)),
        ("lemonchiffon", hex(0xFFFACD)),
        ("lightblue", hex(0xADD8E6)),
        ("lightcoral", hex(0xF08080)),
        ("lightcyan", hex(0xE0FFFF)),
        ("lightgoldenrodyellow", hex(0xFAFAD2)),
        ("lightgreen", hex(0x90EE90)),
        ("lightgrey", hex(0xD3D3D3)),
        ("lightgray", hex(0xD3D3D3)),
        ("lightpink", hex(0xFFB6C1)),
        ("lightsalmon", hex(0xFFA07A)),
        ("lightseagreen", hex(0x20B2AA)),
        ("lightskyblue", hex(0x87CEFA)),
        ("lightslategray", hex(0x778899)),
        ("lightslategrey", hex(0x778899)),
        ("lightsteelblue", hex(0xB0C4DE)),
        ("lightyellow", hex(0xFFFFE0)),
        ("lime", hex(0x00FF00)),
        ("limegreen", hex(0x32CD32)),
        ("linen", hex(0xFAF0E6)),
        ("magenta", hex(0xFF00FF)),
        ("maroon", hex(0x800000)),
        ("mediumaquamarine", hex(0x66CDAA)),
        ("mediumblue", hex(0x0000CD)),
        ("mediumorchid", hex(0xBA55D3)),
        ("mediumpurple", hex(0x9370DB)),
        ("mediumseagreen", hex(0x3CB371)),
        ("mediumslateblue", hex(0x7B68EE)),
        ("mediumspringgreen", hex(0x00FA9A)),
        ("mediumturquoise", hex(0x48D1CC)),
        ("mediumvioletred", hex(0xC71585)),
        ("midnightblue", hex(0x191970)),
        ("mintcream", hex(0xF5FFFA)),
        ("mistyrose", hex(0xFFE4E1)),
        ("moccasin", hex(0xFFE4B5)),
        ("navajowhite", hex(0xFFDEAD)),
        ("navy", hex(0x000080)),
        ("oldlace", hex(0xFDF5E6)),
        ("olive", hex(0x808000)),
        ("olivedrab", hex(0x6B8E23)),
        ("orange", hex(0xFFA500)),
        ("orangered", hex(0xFF4500)),
        ("orchid", hex(0xDA70D6)),
        ("palegoldenrod", hex(0xEEE8AA)),
        ("palegreen", hex(0x98FB98)),
        ("paleturquoise", hex(0xAFEEEE)),
        ("palevioletred", hex(0xDB7093)),
        ("papayawhip", hex(0xFFEFD5)),
        ("peachpuff", hex(0xFFDAB9)),
        ("peru", hex(0xCD853F)),
        ("pink", hex(0xFFC0CB)),
        ("plum", hex(0xDDA0DD)),
        ("powderblue", hex(0xB0E0E6)),
        ("purple", hex(0x800080)),
        ("rebeccapurple", hex(0x663399)),
        ("red", hex(0xFF0000)),
        ("rosybrown", hex(0xBC8F8F)),
        ("royalblue", hex(0x4169E1)),
        ("saddlebrown", hex(0x8B4513)),
        ("salmon", hex(0xFA8072)),
        ("sandybrown", hex(0xF4A460)),
        ("seagreen", hex(0x2E8B57)),
        ("seashell", hex(0xFFF5EE)),
        ("sienna", hex(0xA0522D)),
        ("silver", hex(0xC0C0C0)),
        ("skyblue", hex(0x87CEEB)),
        ("slateblue", hex(0x6A5ACD)),
        ("slategray", hex(0x708090)),
        ("slategrey", hex(0x708090)),
        ("snow", hex(0xFFFAFA)),
        ("springgreen", hex(0x00FF7F)),
        ("steelblue", hex(0x4682B4)),
        ("tan", hex(0xD2B48C)),
        ("teal", hex(0x008080)),
        ("thistle", hex(0xD8BFD8)),
        ("tomato", hex(0xFF6347)),
        ("turquoise", hex(0x40E0D0)),
        ("violet", hex(0xEE82EE)),
        ("wheat", hex(0xF5DEB3)),
        ("white", hex(0xFFFFFF)),
        ("whitesmoke", hex(0xF5F5F5)),
        ("yellow", hex(0xFFFF00)),
        ("yellowgreen", hex(0x9ACD32)),
    ]
});

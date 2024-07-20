//
//  MudOutput.swift
//  SmushClient
//
//  Created by Joshua Booth on 7/19/24.
//

import Foundation
import SwiftUI

public enum OutputColor {
    case ansi(Int)
    case baseline
    case hex(Color)
    
    func color(_ colors: AnsiColors) -> Color? {
        switch self {
        case .ansi(let code):
            return colors[code]
        case .hex(let color):
            return color
        case .baseline:
            return Optional.none
        }
    }
}

extension OutputColor {
    init(ansi: UInt8) {
        self = .ansi(Int(ansi))
    }
    
    init(hex: UInt32) {
        self = .hex(Color.init(
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255
        ));
    }
    
    init(_ color: MudColor) {
        switch color {
        case .Ansi(let code):
            self.init(ansi: code)
        case .Hex(let hex):
            self.init(hex: hex)
        }
    }
    
    init(foreground: MudColor) {
        switch foreground {
        case .Ansi(7):
            self = .baseline
        case .Hex(0xFFFFFF):
            self = .baseline
        default: self.init(foreground)
        }
    }
    
    init(background: MudColor) {
        switch background {
        case .Ansi(0):
            self = .baseline
        case .Hex(0x000000):
            self = .baseline
        default: self.init(background)
        }
    }
}

func isBlack(_ color: MudColor) -> Bool {
    switch color {
    case .Ansi(let code):
        return code == 0
    case .Hex(let hex):
        return hex == 0x000000
    }
}

func isWhite(_ color: MudColor) -> Bool {
    switch color {
    case .Ansi(let code):
        return code == 7
    case .Hex(let hex):
        return hex == 0xFFFFFF
    }
}

public struct TextFragment: Identifiable {
    public let id: UUID;
    let text: String;
    let foreground: OutputColor;
    let background: OutputColor;
    let bold: Bool;
    let italic: Bool;
    let strikethrough: Bool;
    let underline: Bool;
    
    
    init(_ fragment: RustTextFragment) {
        id = UUID()
        text = String(bytes: fragment.text(), encoding: .utf8)!
        foreground = OutputColor(foreground: fragment.foreground())
        background = OutputColor(background: fragment.background())
        bold = fragment.is_bold()
        italic = fragment.is_italic()
        strikethrough = fragment.is_strikeout()
        underline = fragment.is_underline()
    }
    
    func render(_ colors: AnsiColors) -> Text {
        let text = Text(text).bold(bold).italic(italic).strikethrough(strikethrough).underline(underline);
        if let fgColor = foreground.color(colors) {
            return text.foregroundStyle(fgColor);
        } else {
            return text;
        }
    }
}


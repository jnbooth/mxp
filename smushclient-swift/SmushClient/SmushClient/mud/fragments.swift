//
//  MudOutput.swift
//  SmushClient
//
//  Created by Joshua Booth on 7/19/24.
//

import Foundation
import SwiftUI

enum RenderError: Error {
    case InvalidUtf8
}

func rgb(_ hex: UInt32) -> Color {
    Color.init(
        red: Double((hex >> 16) & 0xFF) / 255,
        green: Double((hex >> 8) & 0xFF) / 255,
        blue: Double(hex & 0xFF) / 255
    )
}

public func renderText(_ fragment: RustTextFragment, _ ansiColors: AnsiColors) -> AttributedString {
    guard let text = String(bytes: fragment.text(), encoding: .utf8) else {
        return AttributedString("�")
    }
    var string = AttributedString(text)
    if let link = fragment.link() {
        let action = getAction(link.action, text)
        string.link = serializeActionUrl(link.sendto, action)
        string.underlineStyle = .single
        string.toolTip = action
    }
    let invert = fragment.is_inverse()
    let foreground = invert ? fragment.background() : fragment.foreground()
    let background = invert ? fragment.foreground() : fragment.background()
    switch foreground {
    case .Ansi(7):
        break
    case .Ansi(let code):
        string.foregroundColor = ansiColors[Int(code)]
    case .Hex(0xFFFFFF):
        break
    case .Hex(let hex):
        string.foregroundColor = rgb(hex)
    }
    switch background {
    case .Ansi(0):
        break
    case .Ansi(let code):
        string.backgroundColor = ansiColors[Int(code)]
    case .Hex(0x000000):
        break
    case .Hex(let hex):
        string.backgroundColor = rgb(hex)
    }
    if fragment.is_strikeout() {
        string.strikethroughStyle = .single
    }
    if fragment.is_underline() {
        string.underlineStyle = .single
    }
    var font = Font.system(.body, weight: fragment.is_bold() ? .bold : .medium)
    if fragment.is_italic() {
        font = font.italic()
    }
    string.font = font
    return string
}


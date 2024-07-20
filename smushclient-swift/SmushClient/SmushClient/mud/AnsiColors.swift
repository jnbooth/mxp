//
//  ansi.swift
//  SmushClient
//
//  Created by Joshua Booth on 7/19/24.
//

import Foundation
import SwiftUI

public typealias AnsiColors = [Color]

public func defaultAnsiColors() -> AnsiColors {
    return [
        Color(red: 0, green: 0, blue: 0), // black
        Color(red: 0.5, green: 0, blue: 0), // red
        Color(red: 0, green: 0.5, blue: 0), // green
        Color(red: 0.5, green: 0.5, blue: 0), // yellow
        Color(red: 0, green: 0, blue: 0.5), // blue
        Color(red: 0.5, green: 0, blue: 0.5), // purple
        Color(red: 0, green: 0.5, blue: 0.5), // cyan
        Color(red: 0.75, green: 0.75, blue: 0.75), // white
        Color(red: 0.5, green: 0.5, blue: 0.5), // bright black
        Color(red: 1, green: 0, blue: 0), // bright red
        Color(red: 0, green: 1, blue: 0), // bright green
        Color(red: 1, green: 1, blue: 0), //bright yellow
        Color(red: 0, green: 0, blue: 1), // bright blue
        Color(red: 1, green: 0, blue: 1), // bright purple
        Color(red: 0, green: 1, blue: 1), // bright cyan
        Color(red: 1, green: 1, blue: 1), // bright white
    ]
}



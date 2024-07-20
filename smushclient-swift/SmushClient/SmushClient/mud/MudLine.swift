//
//  File.swift
//  SmushClient
//
//  Created by Joshua Booth on 7/19/24.
//

import Foundation
import SwiftUI

enum LineSource {
    case client
    case server
}

struct MudLine: Identifiable {
    let id: UUID = UUID();
    public let text: AttributedString;
    public let source: LineSource;
    
    init(_ text: AttributedString) {
        self.text = text
        self.source = .server
    }
}

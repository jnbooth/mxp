//
//  SmushClientApp.swift
//  SmushClient
//
//  Created by Joshua Booth on 7/17/24.
//

import SwiftUI

struct VisualEffect: NSViewRepresentable {
    func makeNSView(context: Self.Context) -> NSView {
        let view = NSVisualEffectView()
        view.material = .underWindowBackground
        view.state = .active
        return view
    }
  func updateNSView(_ nsView: NSView, context: Context) { }
}

@main
struct SmushClientApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
                .background(VisualEffect())
                .handlesExternalEvents(preferring: [actionScheme], allowing: [actionScheme])
        }
    }
}
